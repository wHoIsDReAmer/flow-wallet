use k256::ecdsa::VerifyingKey;
use sha2::{Digest as _, Sha256};
use sha3::Keccak256;

use super::{Chain, ChainError};

/// Tron mainnet chain implementation.
pub struct TronMainnet;

impl Chain for TronMainnet {
    fn id(&self) -> &'static str {
        "tron-mainnet"
    }

    fn address_from_pubkey(&self, pubkey_sec1: &[u8]) -> Result<String, ChainError> {
        tron_address_from_pubkey(pubkey_sec1)
    }
}

/// Derive Tron base58check address from a compressed SEC1 public key.
pub fn tron_address_from_pubkey(pubkey_sec1: &[u8]) -> Result<String, ChainError> {
    let verifying_key =
        VerifyingKey::from_sec1_bytes(pubkey_sec1).map_err(|_| ChainError::InvalidPublicKey)?;

    // Uncompressed SEC1: 0x04 || X(32) || Y(32)
    let encoded = verifying_key.to_encoded_point(false);
    let bytes = encoded.as_bytes();
    if bytes.len() != 65 || bytes[0] != 0x04 {
        return Err(ChainError::Derivation(
            "unexpected uncompressed key format".into(),
        ));
    }

    let keccak = Keccak256::digest(&bytes[1..]);
    let last20 = &keccak[keccak.len() - 20..];

    // Tron base58check: prefix 0x41 (mainnet) + 20-byte payload, double SHA256 checksum (first 4 bytes)
    let mut payload = [0u8; 21];
    payload[0] = 0x41;
    payload[1..].copy_from_slice(last20);

    let checksum_full = Sha256::digest(Sha256::digest(payload));
    let checksum = &checksum_full[..4];

    let mut address_bytes = Vec::with_capacity(25);
    address_bytes.extend_from_slice(&payload);
    address_bytes.extend_from_slice(checksum);

    Ok(bs58::encode(address_bytes).into_string())
}

#[cfg(test)]
mod tests {
    use super::{Chain, TronMainnet, tron_address_from_pubkey};
    use crate::wallet::Signer;
    use crate::wallet::signer::local::LocalSigner;

    #[test]
    fn tron_address_matches_known_vector() {
        // Deterministic secret for test.
        let sk = [1u8; 32];
        let signer = LocalSigner::from_bytes(sk).expect("key");
        let pk = signer.public_key();

        let addr = tron_address_from_pubkey(&pk).expect("addr");
        // Precomputed once via same algorithm; guards against regressions.
        assert_eq!(addr, "TCNkawTmcQgYSU8nP8cHswT1QPjharxJr7");

        // Via chain object
        let chain = TronMainnet;
        let addr2 = chain.address_from_pubkey(&pk).unwrap();
        assert_eq!(addr, addr2);
    }
}
