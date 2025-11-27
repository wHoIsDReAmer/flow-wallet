pub mod chain;
pub mod crypto;
pub mod key_source;
pub mod signer;

use crate::wallet::chain::{Chain, ChainError};
use async_trait::async_trait;

#[async_trait]
pub trait Signer: Send + Sync {
    async fn sign(&self, message: &[u8]) -> Result<Vec<u8>, ()>;
    fn public_key(&self) -> Vec<u8>;
}

#[async_trait]
impl Signer for Box<dyn Signer> {
    async fn sign(&self, message: &[u8]) -> Result<Vec<u8>, ()> {
        (**self).sign(message).await
    }
    fn public_key(&self) -> Vec<u8> {
        (**self).public_key()
    }
}

pub struct Wallet<C: Chain, T: Signer> {
    pub signer: T,
    pub chain: C,
}

impl<C: Chain, T: Signer> Wallet<C, T> {
    pub fn new(signer: T, chain: C) -> Self {
        Self { signer, chain }
    }

    /// Derive the on-chain address for this wallet using the chain rules.
    pub fn address(&self) -> Result<String, ChainError> {
        let pk = self.signer.public_key();
        self.chain.address_from_pubkey(&pk)
    }

    /// Send coins to a destination address.
    /// Orchestrates the flow: create (async) -> prepare (sync) -> sign (async) -> finalize (sync) -> broadcast (async).
    /// Send coins to a destination address.
    /// Orchestrates the flow: create (async) -> prepare (sync) -> sign (async) -> finalize (sync) -> broadcast (async).
    pub async fn send_coins(
        &self,
        provider: &dyn crate::node::Provider,
        to: &str,
        amount: u64,
    ) -> Result<String, crate::WalletError> {
        let from = self.address()?;

        // 1. Create raw transaction (Async, Network)
        let raw_tx = provider.create_transaction(&from, to, amount).await?;

        // 2. Prepare transaction for signing (Sync, Chain Logic)
        let bytes_to_sign = self.chain.prepare_transaction(&raw_tx)?;

        // 3. Sign the bytes (Async, Signer/MPC)
        let mut signatures = Vec::new();
        for bytes in bytes_to_sign {
            let signature = self
                .signer
                .sign(&bytes)
                .await
                .map_err(|_| crate::WalletError::SigningFailed)?;
            signatures.push(signature);
        }

        // 4. Finalize transaction (Sync, Chain Logic)
        let pubkey = self.signer.public_key();
        let signed_tx = self
            .chain
            .finalize_transaction(&raw_tx, &signatures, &pubkey)?;

        // 5. Broadcast transaction (Async, Network)
        let tx_hash = provider.broadcast_transaction(&signed_tx).await?;

        Ok(tx_hash)
    }
}

#[cfg(test)]
mod tests {
    use k256::ecdsa::{Signature, VerifyingKey, signature::DigestVerifier};
    use sha2::{Digest, Sha256};

    use crate::wallet::chain::TRON;
    use crate::wallet::signer::local::LocalSigner;
    use crate::wallet::{Signer, Wallet};

    #[tokio::test]
    async fn test_sign() {
        // 0x01... is a valid small scalar on secp256k1 for testing.
        let secret = [1u8; 32];
        let signer = LocalSigner::from_bytes(secret).expect("valid test key");
        let foo_wallet = Wallet::new(signer, TRON);

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

    #[tokio::test]
    async fn test_tron_address_derivation() {
        let secret = [1u8; 32];
        let signer = LocalSigner::from_bytes(secret).expect("valid key");
        let wallet = Wallet::new(signer, TRON);

        let addr = wallet.address().expect("address");
        assert_eq!(addr, "TCNkawTmcQgYSU8nP8cHswT1QPjharxJr7");
    }
}
