use thiserror::Error;

pub mod tvm;
pub mod utxo;

pub use tvm::{TRON, TvmChain, tvm_address_from_pubkey};
pub use utxo::{LITECOIN, UtxoChain, utxo_address_from_pubkey};

/// Blockchain-specific address derivation contract.
pub trait Chain: Send + Sync {
    fn id(&self) -> &'static str;
    fn address_from_pubkey(&self, pubkey_sec1: &[u8]) -> Result<String, ChainError>;
    fn prepare_transaction(&self, raw_tx: &str) -> Result<Vec<Vec<u8>>, ChainError>;
    fn finalize_transaction(
        &self,
        raw_tx: &str,
        signatures: &[Vec<u8>],
        pubkey: &[u8],
    ) -> Result<String, ChainError>;
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ChainError {
    #[error("invalid public key")]
    InvalidPublicKey,
    #[error("other error: {0}")]
    Other(String),
    #[error("derivation failed: {0}")]
    Derivation(String),
}
