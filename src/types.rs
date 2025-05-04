// src/types.rs
use ethers::types::{Address, Bytes, H256, U256};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserOperation {
    pub sender: Address,
    pub nonce: U256,
    pub init_code: Bytes,
    pub call_data: Bytes,
    pub call_gas_limit: U256,
    pub verification_gas_limit: U256,
    pub pre_verification_gas: U256,
    pub max_fee_per_gas: U256,
    pub max_priority_fee_per_gas: U256,
    pub paymaster_and_data: Bytes,
    pub signature: Bytes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymasterAndData {
    pub paymaster: Address,
    pub valid_until: u64,
    pub valid_after: u64,
    pub signature: Bytes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymasterResponse {
    pub paymaster_and_data: Bytes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub reason: Option<String>,
}