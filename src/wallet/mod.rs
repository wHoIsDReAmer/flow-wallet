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

#[cfg(test)]
mod tests {
    use async_trait::async_trait;

    use crate::wallet::{Signer, Wallet};

    struct NoopSigner {}

    #[async_trait]
    impl Signer for NoopSigner {
        async fn sign(&self, _message: &[u8]) -> Result<Vec<u8>, ()> {
            todo!()
        }

        fn public_key(&self) -> Vec<u8> {
            todo!()
        }
    }

    #[tokio::test]
    async fn test_sign() {
        let signer = NoopSigner {};
        let foo_wallet = Wallet {
            signer,
            chain: "FOO20".into(),
        };

        let message = b"foobar";
        foo_wallet.signer.sign(message).await.ok();
    }
}
