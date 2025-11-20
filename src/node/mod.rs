pub mod errors;
pub mod ext;
pub mod tron;

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
