// src/paymaster.rs
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::Arc;

use anyhow::Result;
use ethers::prelude::*;
use ethers::signers::{LocalWallet, Signer};
use ethers::utils::keccak256;
use tracing::{debug, error, info};

use crate::error::PaymasterError;
use crate::types::{PaymasterAndData, PaymasterResponse, UserOperation, ValidationResult};

pub struct Paymaster {
    wallet: LocalWallet,
    client: Arc<Provider<Http>>,
    pub paymaster_address: Address,
    chain_id: u64,
    // Configuration parameters
    valid_duration: u64, // The validity time window in seconds
    gas_price_buffer: u64, // Buffer percentage for gas price
}

impl Paymaster {
    pub async fn new(
        private_key: String,
        chain_id: u64,
        eth_rpc_url: String,
    ) -> Result<Self> {
        // Create the wallet from private key
        let wallet = private_key.parse::<LocalWallet>()?.with_chain_id(chain_id);
        
        // Create Ethereum client
        let provider = Provider::<Http>::try_from(eth_rpc_url)?;
        let client = Arc::new(provider);
        
        // Get the paymaster address from the wallet
        let paymaster_address = wallet.address();
        
        info!("Initialized paymaster with address: {}", paymaster_address);
        
        Ok(Self {
            wallet,
            client,
            paymaster_address,
            chain_id,
            valid_duration: 3600, // Default 1 hour validity
            gas_price_buffer: 10,  // Default 10% buffer
        })
    }
    
    // Sign a user operation to sponsor it
    pub async fn sign_user_operation(&self, user_op: &UserOperation) -> Result<PaymasterResponse, PaymasterError> {
        // 1. Validate the user operation
        self.validate_user_operation(user_op).await?;
        
        // 2. Calculate the gas cost and check if we can afford it
        let max_cost = self.calculate_max_cost(user_op)?;
        
        // 3. Check if the paymaster has enough funds
        self.check_paymaster_balance(max_cost).await?;
        
        // 4. Create time-range for paymaster validity
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| PaymasterError::InvalidParameters(e.to_string()))?
            .as_secs();
            
        let valid_until = now + self.valid_duration;
        let valid_after = now;
        
        // 5. Create the paymaster data
        let paymaster_data = PaymasterAndData {
            paymaster: self.paymaster_address,
            valid_until,
            valid_after,
            signature: Bytes::default(), // Will be replaced with the actual signature
        };
        
        // 6. Hash and sign the paymaster data
        let signature = self.sign_paymaster_data(user_op, valid_until, valid_after).await?;
        
        // 7. Encode the paymaster data with the signature
        let paymaster_and_data = self.encode_paymaster_data(valid_until, valid_after, signature)?;
        
        Ok(PaymasterResponse {
            paymaster_and_data,
        })
    }
    
    // Validate the user operation
    async fn validate_user_operation(&self, user_op: &UserOperation) -> Result<(), PaymasterError> {
        // Basic validation checks
        if user_op.max_fee_per_gas.is_zero() || user_op.max_priority_fee_per_gas.is_zero() {
            return Err(PaymasterError::InvalidUserOperation("Gas price cannot be zero".to_string()));
        }
        
        // Add more validation as needed
        // ...
        
        Ok(())
    }
    
    // Calculate the maximum cost of the operation
    fn calculate_max_cost(&self, user_op: &UserOperation) -> Result<U256, PaymasterError> {
        // Calculate gas limit: callGasLimit + verificationGasLimit + preVerificationGas
        let total_gas = user_op.call_gas_limit
            .checked_add(user_op.verification_gas_limit)
            .and_then(|sum| sum.checked_add(user_op.pre_verification_gas))
            .ok_or_else(|| PaymasterError::InvalidUserOperation("Gas limit overflow".to_string()))?;
            
        // Apply buffer to gas price
        let buffered_gas_price = user_op.max_fee_per_gas
            .checked_mul(U256::from(100 + self.gas_price_buffer))
            .and_then(|product| product.checked_div(U256::from(100)))
            .ok_or_else(|| PaymasterError::InvalidUserOperation("Gas price calculation error".to_string()))?;
            
        // Calculate max cost
        let max_cost = total_gas
            .checked_mul(buffered_gas_price)
            .ok_or_else(|| PaymasterError::InvalidUserOperation("Max cost calculation overflow".to_string()))?;
            
        Ok(max_cost)
    }
    
    // Check if the paymaster has enough balance
    async fn check_paymaster_balance(&self, max_cost: U256) -> Result<(), PaymasterError> {
        let balance = self.client.get_balance(self.paymaster_address, None)
            .await
            .map_err(|e| PaymasterError::EthereumProviderError(e.to_string()))?;
            
        if balance <= max_cost {
            return Err(PaymasterError::InsufficientFunds);
        }
        
        Ok(())
    }
    
    // Hash and sign the paymaster data
    async fn sign_paymaster_data(
        &self,
        user_op: &UserOperation,
        valid_until: u64,
        valid_after: u64,
    ) -> Result<Bytes, PaymasterError> {
        // Calculate user operation hash according to ERC-4337 spec
        let user_op_hash = self.hash_user_operation(user_op);
        
        // Prepare the message to sign: paymaster + validUntil + validAfter + userOpHash
        let mut message = vec![];
        message.extend_from_slice(&self.paymaster_address.as_bytes());
        message.extend_from_slice(&valid_until.to_be_bytes());
        message.extend_from_slice(&valid_after.to_be_bytes());
        message.extend_from_slice(&user_op_hash.as_bytes());
        
        // Hash the message
        let message_hash = keccak256(&message);
        
        // Sign the hash
        let signature = self.wallet.sign_message(message_hash)
            .await
            .map_err(|e| PaymasterError::SignatureVerificationFailed)?;
            
        // Convert to bytes
        let signature_bytes = Bytes::from(signature.to_vec());
        
        Ok(signature_bytes)
    }
    
    // Encode paymaster data according to ERC-4337 spec
    fn encode_paymaster_data(
        &self,
        valid_until: u64,
        valid_after: u64,
        signature: Bytes,
    ) -> Result<Bytes, PaymasterError> {
        // Encode: paymaster address (20 bytes) + validUntil (32 bytes) + validAfter (32 bytes) + signature
        let mut data = vec![];
        
        // Add paymaster address
        data.extend_from_slice(self.paymaster_address.as_bytes());
        
        // Add valid until (32 bytes)
        let mut valid_until_bytes = [0u8; 32];
        let valid_until_be = valid_until.to_be_bytes();
        valid_until_bytes[32 - valid_until_be.len()..].copy_from_slice(&valid_until_be);
        data.extend_from_slice(&valid_until_bytes);
        
        // Add valid after (32 bytes)
        let mut valid_after_bytes = [0u8; 32];
        let valid_after_be = valid_after.to_be_bytes();
        valid_after_bytes[32 - valid_after_be.len()..].copy_from_slice(&valid_after_be);
        data.extend_from_slice(&valid_after_bytes);
        
        // Add signature
        data.extend_from_slice(&signature);
        
        Ok(Bytes::from(data))
    }
    
    // Calculate the hash of a user operation according to ERC-4337 spec
    fn hash_user_operation(&self, user_op: &UserOperation) -> H256 {
        // Pack the user operation
        let mut data = vec![];
        
        // Pack sender
        data.extend_from_slice(user_op.sender.as_bytes());
        
        // Pack nonce (32 bytes)
        let nonce_bytes = ethers::utils::rlp::encode(&user_op.nonce);
        data.extend_from_slice(&nonce_bytes);
        
        // Pack initCode hash
        data.extend_from_slice(&keccak256(&user_op.init_code));
        
        // Pack callData hash
        data.extend_from_slice(&keccak256(&user_op.call_data));
        
        // Pack callGasLimit
        let call_gas_limit_bytes = ethers::utils::rlp::encode(&user_op.call_gas_limit);
        data.extend_from_slice(&call_gas_limit_bytes);
        
        // Pack verificationGasLimit
        let verification_gas_limit_bytes = ethers::utils::rlp::encode(&user_op.verification_gas_limit);
        data.extend_from_slice(&verification_gas_limit_bytes);
        
        // Pack preVerificationGas
        let pre_verification_gas_bytes = ethers::utils::rlp::encode(&user_op.pre_verification_gas);
        data.extend_from_slice(&pre_verification_gas_bytes);
        
        // Pack maxFeePerGas
        let max_fee_per_gas_bytes = ethers::utils::rlp::encode(&user_op.max_fee_per_gas);
        data.extend_from_slice(&max_fee_per_gas_bytes);
        
        // Pack maxPriorityFeePerGas
        let max_priority_fee_per_gas_bytes = ethers::utils::rlp::encode(&user_op.max_priority_fee_per_gas);
        data.extend_from_slice(&max_priority_fee_per_gas_bytes);
        
        // First hash
        let hash = keccak256(&data);
        
        // Include chain ID and entrypoint address in the hash
        let mut chain_hash_data = vec![];
        chain_hash_data.extend_from_slice(&hash);
        chain_hash_data.extend_from_slice(&ethers::utils::rlp::encode(&U256::from(self.chain_id)));
        chain_hash_data.extend_from_slice(self.paymaster_address.as_bytes());
        
        // Final hash
        let final_hash = keccak256(&chain_hash_data);
        H256::from_slice(&final_hash)
    }
    
}