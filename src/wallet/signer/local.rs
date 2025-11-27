use async_trait::async_trait;
use k256::ecdsa::{Signature, SigningKey, VerifyingKey, signature::DigestSigner};
use sha2::{Digest, Sha256};

use crate::wallet::Signer;

/// Local software signer backed by an in-memory secp256k1 private key.
pub struct LocalSigner {
    signing_key: SigningKey,
}

impl LocalSigner {
    /// Create a signer from a 32-byte secp256k1 secret scalar.
    pub fn from_bytes(secret_key: [u8; 32]) -> Result<Self, k256::ecdsa::Error> {
        Self::from_slice(&secret_key)
    }

    /// Create a signer from a secret scalar slice.
    pub fn from_slice(secret_key: &[u8]) -> Result<Self, k256::ecdsa::Error> {
        let signing_key = SigningKey::from_bytes(secret_key.into())?;
        Ok(Self { signing_key })
    }

    /// Return the compressed public key (33 bytes, SEC1).
    fn compressed_public_key(&self) -> VerifyingKey {
        self.signing_key.verifying_key().to_owned()
    }
}

#[async_trait]
impl Signer for LocalSigner {
    async fn sign(&self, message: &[u8]) -> Result<Vec<u8>, ()> {
        // Hash the message to 32 bytes; required size for secp256k1 signing.
        let digest = Sha256::new().chain_update(message);
        let signature: Signature = self.signing_key.sign_digest(digest);
        Ok(signature.to_der().as_bytes().to_vec())
    }

    fn public_key(&self) -> Vec<u8> {
        self.compressed_public_key()
            .to_encoded_point(true)
            .as_bytes()
            .to_vec()
    }
}
