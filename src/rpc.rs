// src/rpc.rs
use std::sync::Arc;

use jsonrpsee::core::{async_trait, RpcResult};
use jsonrpsee::proc_macros::rpc;
use jsonrpsee::RpcModule;
use serde_json::json;
use tracing::{debug, error, info};

use crate::error::PaymasterError;
use crate::paymaster::Paymaster;
use crate::types::{PaymasterResponse, UserOperation, ValidationResult};

// Define the RPC interface
#[rpc(server, namespace = "pm")]
pub trait PaymasterRpc {
    /// Requests the paymaster to sponsor a user operation
    #[method(name = "sponsorUserOperation")]
    async fn sponsor(&self, user_op: UserOperation) -> RpcResult<PaymasterResponse>;
}

pub struct PaymasterRpcImpl {
    paymaster: Arc<Paymaster>,
}

impl PaymasterRpcImpl {
    pub fn new(paymaster: Arc<Paymaster>) -> Self {
        Self { paymaster }
    }
}

#[async_trait]
impl PaymasterRpcServer for PaymasterRpcImpl {
    async fn sponsor(&self, user_op: UserOperation) -> RpcResult<PaymasterResponse> {
        debug!("Received sponsor request for sender: {}", user_op.sender);
        
        match self.paymaster.sign_user_operation(&user_op).await {
            Ok(response) => {
                info!("Successfully sponsored operation for {}", user_op.sender);
                Ok(response)
            }
            Err(e) => {
                error!("Failed to sponsor operation: {}", e);
                Err(jsonrpsee::types::error::ErrorObject::owned(
                    -32000,
                    format!("Paymaster error: {}", e),
                    None::<()>,
                ))
            }
        }
    }
}

pub fn register_methods(module: &mut RpcModule<PaymasterRpcImpl>) -> anyhow::Result<()> {
    module.register_async_method("pm_sponsorUserOperation", |params, context| async move {
        let user_op = params.parse::<UserOperation>()?;
        context.sponsor(user_op).await
    })?;
    
    Ok(())
}