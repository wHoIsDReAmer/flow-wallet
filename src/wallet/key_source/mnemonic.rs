use async_trait::async_trait;
use bip32::XPrv;
use bip39::Mnemonic;
use rand::RngCore;
use std::str::FromStr;

use super::{KeySource, KeySourceError};
use crate::wallet::Signer;
use crate::wallet::crypto::memory::SecureBuffer;
use crate::wallet::signer::local::LocalSigner;

/// Local HD Wallet key source based on BIP-39 mnemonic.
pub struct MnemonicKeySource {
    seed: SecureBuffer,
    phrase: SecureBuffer,
}

impl MnemonicKeySource {
    /// Create a new source from a BIP-39 mnemonic phrase.
    pub fn new(phrase: &str, passphrase: Option<&str>) -> Result<Self, KeySourceError> {
        let mnemonic = Mnemonic::from_str(phrase)
            .map_err(|e| KeySourceError::InvalidMnemonic(e.to_string()))?;
        let seed = mnemonic.to_seed(passphrase.unwrap_or(""));
        Ok(Self {
            seed: SecureBuffer::new(seed.to_vec()),
            phrase: SecureBuffer::from(phrase),
        })
    }

    /// Generate a new random mnemonic (12 words).
    pub fn random(passphrase: Option<&str>) -> Self {
        let mut entropy = [0u8; 16]; // 128 bits = 12 words
        rand::rng().fill_bytes(&mut entropy);

        let mnemonic = Mnemonic::from_entropy(&entropy).expect("valid entropy");
        let phrase = mnemonic.to_string();
        let seed = mnemonic.to_seed(passphrase.unwrap_or(""));
        Self {
            seed: SecureBuffer::new(seed.to_vec()),
            phrase: SecureBuffer::from(phrase),
        }
    }

    /// Get the mnemonic phrase.
    pub fn phrase(&self) -> &str {
        self.phrase.as_str().unwrap_or("")
    }
}

#[async_trait]
impl KeySource for MnemonicKeySource {
    async fn derive_signer(&self, path: &str) -> Result<Box<dyn Signer>, KeySourceError> {
        let xprv = XPrv::derive_from_path(&self.seed, &path.parse().unwrap())
            .map_err(|e| KeySourceError::Derivation(e.to_string()))?;

        let secret_key_bytes = xprv.private_key().to_bytes();
        let signer = LocalSigner::from_slice(&secret_key_bytes)
            .map_err(|e| KeySourceError::Derivation(e.to_string()))?;

        Ok(Box::new(signer))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mnemonic_derivation() {
        // Generate a random valid mnemonic
        let source = MnemonicKeySource::random(None);

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

    #[tokio::test]
    async fn test_passphrase_derivation() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

        // Source 1: No passphrase
        let source1 = MnemonicKeySource::new(phrase, None).expect("valid");
        let signer1 = source1
            .derive_signer("m/44'/0'/0'/0/0")
            .await
            .expect("derive");

        // Source 2: With passphrase
        let source2 = MnemonicKeySource::new(phrase, Some("secret")).expect("valid");
        let signer2 = source2
            .derive_signer("m/44'/0'/0'/0/0")
            .await
            .expect("derive");

        // Keys should be different
        assert_ne!(signer1.public_key(), signer2.public_key());
    }
}
