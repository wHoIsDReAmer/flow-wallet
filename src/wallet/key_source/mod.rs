use async_trait::async_trait;
use thiserror::Error;

use crate::wallet::Signer;

pub mod mnemonic;
pub mod mpc;

pub use mnemonic::MnemonicKeySource;
pub use mpc::MpcKeySource;

#[derive(Debug, Error)]
pub enum KeySourceError {
    #[error("invalid mnemonic: {0}")]
    InvalidMnemonic(String),
    #[error("derivation failed: {0}")]
    Derivation(String),
}

/// Abstract source of keys.
/// Can be a local mnemonic, a hardware wallet, or an MPC share.
#[async_trait]
pub trait KeySource: Send + Sync {
    /// Derive a signer for a specific path.
    /// For local mnemonics, this derives the private key.
    /// For MPC, this might prepare a session for that path.
    async fn derive_signer(&self, path: &str) -> Result<Box<dyn Signer>, KeySourceError>;
}
