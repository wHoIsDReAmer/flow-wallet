use async_trait::async_trait;

use crate::wallet::Signer;

// TODO: implments LocalSigner
pub struct LocalSigner;

#[async_trait]
impl Signer for LocalSigner {
    async fn sign(&self, _message: &[u8]) -> Result<Vec<u8>, ()> {
        todo!()
    }
    fn public_key(&self) -> Vec<u8> {
        todo!()
    }
}
