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
    async fn sign(&self, _message: &[u8]) -> Result<Vec<u8>, ()> {
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
        // Parse the path. Note: XPub can only derive non-hardened children.
        // Path should be relative to the xpub's depth if possible, or we assume the xpub is the root
        // and we are deriving children.
        // For simplicity, let's assume the path string is a standard BIP-32 path.
        // However, `bip32` crate's `derive_child` works on `DerivationPath`.

        // We need to handle the "m/" prefix or relative paths.
        // If the xpub is already at "m/44'/0'/0'", then deriving "0/0" gives the first address.

        let derivation_path: bip32::DerivationPath = path
            .parse()
            .map_err(|e| KeySourceError::Derivation(format!("Invalid path: {}", e)))?;

        // Wait, `derive_child` only takes one index. `derive_from_path` is for XPrv usually.
        // For XPub, we need to iterate over the path components.
        // Also, XPub cannot derive hardened indices.

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
        // Known vector
        // Mnemonic: abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about
        // Path: m/44'/0'/0'
        // XPub: xpub6C5UHIn72E6z1j5p... (This is just a placeholder, need a real one)

        // Let's use a real xpub generated from the mnemonic above for m/44'/0'/0'
        // xpub6DUSaRj8C1...

        // For this test, let's use a known valid xpub.
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
