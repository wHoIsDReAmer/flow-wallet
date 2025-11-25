pub mod signer;
pub mod transaction;

use async_trait::async_trait;

#[async_trait]
pub trait Signer: Send + Sync {
    async fn sign(&self, message: &[u8]) -> Result<Vec<u8>, ()>;
    fn public_key(&self) -> Vec<u8>;
}

pub struct Wallet<T: Signer> {
    pub signer: T,
    pub chain: String,
}

impl<T: Signer> Wallet<T> {
    pub fn new(signer: T, chain: String) -> Self {
        Self { signer, chain }
    }
}

#[cfg(test)]
mod tests {
    use k256::ecdsa::{Signature, VerifyingKey, signature::DigestVerifier};
    use sha2::{Digest, Sha256};

    use crate::wallet::signer::local::LocalSigner;
    use crate::wallet::{Signer, Wallet};

    #[tokio::test]
    async fn test_sign() {
        // 0x01... is a valid small scalar on secp256k1 for testing.
        let secret = [1u8; 32];
        let signer = LocalSigner::from_bytes(secret).expect("valid test key");
        let foo_wallet = Wallet::new(signer, "FOO20".into());

        let message = b"foobar";
        let sig_bytes = foo_wallet.signer.sign(message).await.expect("signs");

        // Verify signature using the public key the wallet exposes.
        let vk_bytes = foo_wallet.signer.public_key();
        let verifying_key = VerifyingKey::from_sec1_bytes(&vk_bytes).expect("valid pk");
        let sig = Signature::from_der(&sig_bytes).expect("der sig");
        let digest = Sha256::new().chain_update(message);
        verifying_key
            .verify_digest(digest, &sig)
            .expect("signature should verify");
    }

    #[tokio::test]
    async fn test_public_key_format() {
        let secret = [2u8; 32];
        let signer = LocalSigner::from_bytes(secret).expect("valid test key");

        let pk = signer.public_key();
        assert_eq!(
            pk.len(),
            33,
            "compressed SEC1 public key should be 33 bytes"
        );

        VerifyingKey::from_sec1_bytes(&pk).expect("public key must parse");
    }
}
