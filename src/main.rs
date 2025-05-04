// src/main.rs
use std::net::SocketAddr;
use std::sync::Arc;

use clap::Parser;
use dotenv::dotenv;
use jsonrpsee::server::{ServerBuilder, ServerHandle};
use jsonrpsee::RpcModule;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod error;
mod paymaster;
mod rpc;
mod types;

use crate::paymaster::Paymaster;
use crate::rpc::PaymasterRpcImpl;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, default_value = "127.0.0.1:8545")]
    rpc_server_addr: String,
    
    #[clap(short, long)]
    private_key: String,
    
    #[clap(short, long)]
    chain_id: u64,
    
    #[clap(short, long)]
    eth_rpc_url: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    
    // Initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    
    // Parse command line arguments
    let args = Args::parse();
    
    // Create the paymaster service
    let paymaster = Paymaster::new(
        args.private_key,
        args.chain_id,
        args.eth_rpc_url,
    ).await?;
    
    // Create the JSON-RPC server
    let server_addr: SocketAddr = args.rpc_server_addr.parse()?;
    let paymaster_rpc = PaymasterRpcImpl::new(Arc::new(paymaster));
    
    info!("Starting ERC-4337 Paymaster RPC server on {}", server_addr);
    
    // Start the JSON-RPC server
    let server_handle = start_server(server_addr, paymaster_rpc).await?;
    
    // Keep the server running until Ctrl+C is pressed
    tokio::signal::ctrl_c().await?;
    server_handle.stop()?;
    info!("Server stopped");
    
    Ok(())
}

async fn start_server(
    server_addr: SocketAddr,
    paymaster_rpc: PaymasterRpcImpl
) -> anyhow::Result<ServerHandle> {
    let server = ServerBuilder::default()
        .build(server_addr)
        .await?;
    
    let mut module = RpcModule::new(paymaster_rpc);
    rpc::register_methods(&mut module)?;
    let server_handle = server.start(module);
    
    Ok(server_handle)
}