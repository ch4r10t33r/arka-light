// src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PaymasterError {
    #[error("Invalid UserOperation: {0}")]
    InvalidUserOperation(String),
    
    #[error("Insufficient funds for sponsoring transaction")]
    InsufficientFunds,
    
    #[error("Signature verification failed")]
    SignatureVerificationFailed,
    
    #[error("Transaction reverted: {0}")]
    TransactionReverted(String),
    
    #[error("Ethereum provider error: {0}")]
    EthereumProviderError(String),
    
    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),
    
    #[error("Unsupported operation")]
    UnsupportedOperation,
}