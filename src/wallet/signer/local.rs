use async_trait::async_trait;
use k256::ecdsa::{SigningKey, VerifyingKey};

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
    async fn sign(&self, digest: &[u8]) -> Result<Vec<u8>, ()> {
        // secp256k1 signs a fixed 32-byte digest. Hashing the payload is the chain's
        // responsibility (see `Chain::prepare_transaction`), so we sign as-is and
        // reject anything that isn't a 32-byte prehash.
        if digest.len() != 32 {
            return Err(());
        }

        // k256 normalizes to low-S. Return the canonical, chain-neutral form
        // r(32) || s(32) || v(1), where `v` is the raw recovery id (0/1).
        let (signature, recovery_id) = self
            .signing_key
            .sign_prehash_recoverable(digest)
            .map_err(|_| ())?;

        let mut out = Vec::with_capacity(65);
        out.extend_from_slice(&signature.to_bytes());
        out.push(recovery_id.to_byte());
        Ok(out)
    }

    fn public_key(&self) -> Vec<u8> {
        self.compressed_public_key()
            .to_encoded_point(true)
            .as_bytes()
            .to_vec()
    }
}
