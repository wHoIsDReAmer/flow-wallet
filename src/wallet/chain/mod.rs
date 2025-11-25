use thiserror::Error;

pub mod tron;

pub use tron::{TronMainnet, tron_address_from_pubkey};

/// Blockchain-specific address derivation contract.
pub trait Chain: Send + Sync {
    fn id(&self) -> &'static str;
    fn address_from_pubkey(&self, pubkey_sec1: &[u8]) -> Result<String, ChainError>;
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ChainError {
    #[error("invalid public key bytes")]
    InvalidPublicKey,
    #[error("derivation failed: {0}")]
    Derivation(String),
}
