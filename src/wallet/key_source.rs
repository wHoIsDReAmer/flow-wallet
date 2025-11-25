use async_trait::async_trait;
use bip32::XPrv;
use bip39::Mnemonic;
use rand::RngCore;
use std::str::FromStr;
use std::sync::Arc;
use thiserror::Error;

use crate::wallet::Signer;
use crate::wallet::mpc::signer::{KeyShare, MpcSigner};
use crate::wallet::mpc::transport::MpcTransport;
use crate::wallet::signer::local::LocalSigner;

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

/// Local HD Wallet key source based on BIP-39 mnemonic.
pub struct MnemonicKeySource {
    seed: [u8; 64],
    phrase: String,
}

impl MnemonicKeySource {
    /// Create a new source from a BIP-39 mnemonic phrase.
    pub fn new(phrase: &str) -> Result<Self, KeySourceError> {
        let mnemonic = Mnemonic::from_str(phrase)
            .map_err(|e| KeySourceError::InvalidMnemonic(e.to_string()))?;
        let seed = mnemonic.to_seed(""); // TODO: Support passphrase
        Ok(Self {
            seed,
            phrase: phrase.to_string(),
        })
    }

    /// Generate a new random mnemonic (12 words).
    pub fn random() -> Self {
        let mut entropy = [0u8; 16]; // 128 bits = 12 words
        rand::rng().fill_bytes(&mut entropy);

        let mnemonic = Mnemonic::from_entropy(&entropy).expect("valid entropy");
        let phrase = mnemonic.to_string();
        let seed = mnemonic.to_seed("");
        Self { seed, phrase }
    }

    /// Get the mnemonic phrase.
    pub fn phrase(&self) -> &str {
        &self.phrase
    }
}

#[async_trait]
impl KeySource for MnemonicKeySource {
    async fn derive_signer(&self, path: &str) -> Result<Box<dyn Signer>, KeySourceError> {
        let xprv = XPrv::derive_from_path(self.seed, &path.parse().unwrap())
            .map_err(|e| KeySourceError::Derivation(e.to_string()))?;

        let secret_key_bytes = xprv.private_key().to_bytes();
        let signer = LocalSigner::from_bytes(secret_key_bytes.into())
            .map_err(|e| KeySourceError::Derivation(e.to_string()))?;

        Ok(Box::new(signer))
    }
}

/// MPC-based key source.
pub struct MpcKeySource {
    share: KeyShare,
    transport: Arc<dyn MpcTransport>,
}

impl MpcKeySource {
    pub fn new(share: KeyShare, transport: Arc<dyn MpcTransport>) -> Self {
        Self { share, transport }
    }
}

#[async_trait]
impl KeySource for MpcKeySource {
    async fn derive_signer(&self, _path: &str) -> Result<Box<dyn Signer>, KeySourceError> {
        // In a real MPC, derivation might involve communication or just using the share for that path.
        // For this skeleton, we assume the share is already for the target key.
        // We clone the share data for the new signer instance.
        let signer_share = KeyShare {
            public_key: self.share.public_key.clone(),
            share_data: self.share.share_data.clone(),
        };

        Ok(Box::new(MpcSigner::new(
            signer_share,
            self.transport.clone(),
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mnemonic_derivation() {
        // Generate a random valid mnemonic
        let source = MnemonicKeySource::random();

        // Check retrieval
        assert!(!source.phrase().is_empty());
        println!("{}", source.phrase());

        // BIP-44 path for Bitcoin: m/44'/0'/0'/0/0
        // We'll use a simple path for testing.
        let path = "m/44'/0'/0'/0/0";
        let signer = source.derive_signer(path).await.expect("derivation");

        // Check if public key matches expected
        let pk = signer.public_key();
        assert_eq!(pk.len(), 33);
    }
}
