use sha2::{Digest, Sha256};
use sha3::Keccak256;

use super::ripemd160::ripemd160;

pub fn sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

pub fn double_sha256(data: &[u8]) -> [u8; 32] {
    sha256(&sha256(data))
}

pub fn keccak256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    hasher.update(data);
    hasher.finalize().into()
}

pub fn hash160(data: &[u8]) -> [u8; 20] {
    ripemd160(&sha256(data))
}

#[cfg(test)]
mod tests {
    use super::{double_sha256, hash160, keccak256, sha256};

    fn to_hex(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }

    #[test]
    fn sha256_known_vector() {
        let out = sha256(b"abc");
        assert_eq!(
            to_hex(&out),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn double_sha256_known_vector() {
        let out = double_sha256(b"abc");
        assert_eq!(
            to_hex(&out),
            "4f8b42c22dd3729b519ba6f68d2da7cc5b2d606d05daed5ad5128cc03e6c6358"
        );
    }

    #[test]
    fn keccak256_known_vector() {
        let out = keccak256(b"abc");
        assert_eq!(
            to_hex(&out),
            "4e03657aea45a94fc7d47ba826c8d667c0d1e6e33a64a036ec44f58fa12d6c45"
        );
    }

    #[test]
    fn hash160_known_vector() {
        let out = hash160(b"abc");
        assert_eq!(to_hex(&out), "bb1be98c142444d7a56aa3981c3942a978e4dc33");
    }
}
