pub mod errors;
pub mod network;
pub mod utils;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::node::errors::NodeError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub hash: String,
    pub from: String,
    pub to: String,
    pub value: String, // Using String for precision (BigInt)
    pub block_number: u64,
    pub timestamp: u64,
    pub status: String, // "SUCCESS", "FAILED"
}

#[async_trait]
pub trait Provider: Send + Sync {
    /// Get transactions for a specific address
    async fn get_transactions(&self, address: &str) -> Result<Vec<Transaction>, NodeError>;

    /// Get the latest block number
    async fn get_block_number(&self) -> Result<u64, NodeError>;

    /// Get the balance of an address
    async fn get_balance(&self, address: &str) -> Result<String, NodeError>;
}
