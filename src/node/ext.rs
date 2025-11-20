use async_trait::async_trait;

use crate::node::{Transaction, errors::NodeError};

#[async_trait]
pub trait Provider: Send + Sync {
    /// Get transactions for a specific address
    async fn get_transactions(&self, address: &str) -> Result<Vec<Transaction>, NodeError>;

    /// Get the latest block number
    async fn get_block_number(&self) -> Result<u64, NodeError>;
}
