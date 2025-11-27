use thiserror::Error;

use crate::node::NodeError;
use crate::wallet::chain::ChainError;
use crate::wallet::key_source::KeySourceError;

#[derive(Debug, Error)]
pub enum WalletError {
    #[error("Chain error: {0}")]
    Chain(#[from] ChainError),

    #[error("Key source error: {0}")]
    KeySource(#[from] KeySourceError),

    #[error("Node error: {0}")]
    Node(#[from] NodeError),

    #[error("Signing failed")]
    SigningFailed,
}
