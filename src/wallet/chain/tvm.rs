use k256::ecdsa::VerifyingKey;

use crate::wallet::crypto::hash::{double_sha256, keccak256};

use super::{Chain, ChainError};

/// Generic TVM-based chain implementation (e.g. Tron, Tron Testnet).
pub struct TvmChain {
    pub name: &'static str,
    pub address_prefix: u8,
}

impl Chain for TvmChain {
    fn id(&self) -> &'static str {
        self.name
    }

    fn address_from_pubkey(&self, pubkey_sec1: &[u8]) -> Result<String, ChainError> {
        tvm_address_from_pubkey(pubkey_sec1, self.address_prefix)
    }
}

/// Tron Mainnet configuration.
pub const TRON: TvmChain = TvmChain {
    name: "tron",
    address_prefix: 0x41,
};

/// Derive TVM base58check address from a compressed SEC1 public key.
pub fn tvm_address_from_pubkey(pubkey_sec1: &[u8], prefix: u8) -> Result<String, ChainError> {
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

    let keccak = keccak256(&bytes[1..]);
    let last20 = &keccak[keccak.len() - 20..];

    // Tron base58check: prefix (e.g. 0x41) + 20-byte payload, double SHA256 checksum (first 4 bytes)
    let mut payload = [0u8; 21];
    payload[0] = prefix;
    payload[1..].copy_from_slice(last20);

    let checksum_full = double_sha256(&payload);
    let checksum = &checksum_full[..4];

    let mut address_bytes = Vec::with_capacity(25);
    address_bytes.extend_from_slice(&payload);
    address_bytes.extend_from_slice(checksum);

    Ok(bs58::encode(address_bytes).into_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wallet::Signer;
    use crate::wallet::signer::local::LocalSigner;

    #[test]
    fn tron_address_matches_known_vector() {
        // Deterministic secret for test.
        let sk = [1u8; 32];
        let signer = LocalSigner::from_bytes(sk).expect("key");
        let pk = signer.public_key();

        // Tron prefix 0x41
        let addr = tvm_address_from_pubkey(&pk, 0x41).expect("addr");
        // Precomputed once via same algorithm; guards against regressions.
        assert_eq!(addr, "TCNkawTmcQgYSU8nP8cHswT1QPjharxJr7");

        // Via chain object
        let chain = TRON;
        let addr2 = chain.address_from_pubkey(&pk).unwrap();
        assert_eq!(addr, addr2);
    }
}
