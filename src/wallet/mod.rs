pub mod chain;
pub mod crypto;
pub mod key_source;
pub mod signer;

use crate::wallet::chain::{Chain, ChainError};
use async_trait::async_trait;

#[async_trait]
pub trait Signer: Send + Sync {
    /// Sign a 32-byte prehash `digest` and return a canonical, chain-neutral
    /// recoverable signature: `r(32) || s(32) || v(1)`, where `v` is the raw
    /// secp256k1 recovery id (0/1) and `s` is low-S normalized.
    ///
    /// Hashing the transaction payload into a digest is the responsibility of
    /// [`Chain::prepare_transaction`], not the signer — this keeps signers chain
    /// agnostic (Tron hashes with SHA256, UTXO uses the prehash from the node,
    /// Ethereum would use keccak256). Each [`Chain`] then re-encodes the result as
    /// needed (Tron uses it verbatim; UTXO drops `v` and converts r||s to DER).
    async fn sign(&self, digest: &[u8]) -> Result<Vec<u8>, ()>;
    fn public_key(&self) -> Vec<u8>;
}

#[async_trait]
impl Signer for Box<dyn Signer> {
    async fn sign(&self, digest: &[u8]) -> Result<Vec<u8>, ()> {
        (**self).sign(digest).await
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
    pub async fn send_coins(
        &self,
        provider: &dyn crate::rpc::Provider,
        to: &str,
        amount: u64,
    ) -> Result<String, crate::WalletError> {
        let from = self.address()?;

        // 1. Create raw transaction (Async, Network)
        let raw_tx = provider.create_transaction(&from, to, amount).await?;

        // 2. Prepare transaction for signing (Sync, Chain Logic).
        // Returns the 32-byte digest(s) to sign; the chain owns the hashing rules.
        let digests = self.chain.prepare_transaction(&raw_tx)?;

        // 3. Sign each digest (Async, Signer/MPC)
        let mut signatures = Vec::new();
        for digest in digests {
            let signature = self
                .signer
                .sign(&digest)
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
    use k256::ecdsa::{
        RecoveryId, Signature, VerifyingKey, signature::hazmat::PrehashVerifier,
    };
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

        // The signer signs a 32-byte prehash digest; hashing is the caller's job.
        let digest = Sha256::digest(b"foobar");
        let sig_bytes = foo_wallet.signer.sign(&digest).await.expect("signs");

        // Canonical recoverable form: r(32) || s(32) || v(1).
        assert_eq!(sig_bytes.len(), 65, "signature must be r||s||v (65 bytes)");

        // Verify signature using the public key the wallet exposes.
        let vk_bytes = foo_wallet.signer.public_key();
        let verifying_key = VerifyingKey::from_sec1_bytes(&vk_bytes).expect("valid pk");
        let sig = Signature::from_slice(&sig_bytes[..64]).expect("r||s sig");
        verifying_key
            .verify_prehash(&digest, &sig)
            .expect("signature should verify");

        // The recovery id must recover the same public key — this is what makes the
        // signature valid for Tron, which has no separate pubkey field.
        let recovery_id = RecoveryId::from_byte(sig_bytes[64]).expect("valid recovery id");
        let recovered = VerifyingKey::recover_from_prehash(&digest, &sig, recovery_id)
            .expect("recover pubkey");
        assert_eq!(recovered, verifying_key, "recovery id must match the signer");
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
