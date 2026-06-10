use async_trait::async_trait;
use bip32::XPub;
use k256::ecdsa::VerifyingKey;
use std::str::FromStr;

use super::{KeySource, KeySourceError};
use crate::wallet::Signer;

/// A signer that can only provide public keys but cannot sign.
/// Used for watch-only wallets.
pub struct WatchOnlySigner {
    public_key: VerifyingKey,
}

impl WatchOnlySigner {
    pub fn new(public_key: VerifyingKey) -> Self {
        Self { public_key }
    }
}

#[async_trait]
impl Signer for WatchOnlySigner {
    async fn sign(&self, _digest: &[u8]) -> Result<Vec<u8>, ()> {
        // Watch-only wallets cannot sign.
        Err(())
    }

    fn public_key(&self) -> Vec<u8> {
        self.public_key.to_encoded_point(true).as_bytes().to_vec()
    }
}

/// Key source based on an Extended Public Key (xpub).
/// Can derive child public keys but cannot derive private keys.
pub struct XPubKeySource {
    xpub: XPub,
}

impl XPubKeySource {
    /// Create a new source from an xpub string.
    pub fn new(xpub_str: &str) -> Result<Self, KeySourceError> {
        let xpub = XPub::from_str(xpub_str)
            .map_err(|e| KeySourceError::Derivation(format!("Invalid xpub: {}", e)))?;
        Ok(Self { xpub })
    }
}

#[async_trait]
impl KeySource for XPubKeySource {
    async fn derive_signer(&self, path: &str) -> Result<Box<dyn Signer>, KeySourceError> {
        let derivation_path: bip32::DerivationPath = path
            .parse()
            .map_err(|e| KeySourceError::Derivation(format!("Invalid path: {}", e)))?;

        // XPub derives non-hardened children only, one index at a time (no
        // `derive_from_path`), so walk the path component-by-component.
        let mut current_xpub = self.xpub.clone();
        for child_index in derivation_path {
            current_xpub = current_xpub
                .derive_child(child_index)
                .map_err(|e| KeySourceError::Derivation(format!("Derivation failed: {}", e)))?;
        }

        Ok(Box::new(WatchOnlySigner::new(*current_xpub.public_key())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_xpub_derivation() {
        let valid_xpub = "xpub661MyMwAqRbcFtXgS5sYJABqqG9YLmC4Q1Rdap9gSE8NqtwybGhePY2gZ29ESFjqJoCu1Rupje8YtGqsefD265TMg7usUDFdp6W1EGMcet8";

        let source = XPubKeySource::new(valid_xpub).expect("create source");

        // Derive m/0/0 (relative to the xpub)
        let signer = source.derive_signer("m/0/0").await.expect("derive");

        assert_eq!(signer.public_key().len(), 33);

        // Ensure signing fails
        let res = signer.sign(b"test").await;
        assert!(res.is_err());
    }
}
