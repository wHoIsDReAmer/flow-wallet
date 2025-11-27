use async_trait::async_trait;
use std::sync::Arc;

use crate::wallet::Signer;
use crate::wallet::signer::mpc::signer::{KeyShare, MpcSigner};
use crate::wallet::signer::mpc::transport::MpcTransport;

use super::{KeySource, KeySourceError};

/// MPC-based key source.
pub struct MpcKeySource {
    share: KeyShare,
    transport: Arc<dyn MpcTransport>,
}

impl MpcKeySource {
    pub fn new(share: KeyShare, transport: Arc<dyn MpcTransport>) -> Self {
        Self { share, transport }
    }
}

#[async_trait]
impl KeySource for MpcKeySource {
    async fn derive_signer(&self, _path: &str) -> Result<Box<dyn Signer>, KeySourceError> {
        // TODO:
        // In a real MPC, derivation might involve communication or just using the share for that path.
        // For this skeleton, we assume the share is already for the target key.
        // We clone the share data for the new signer instance.
        let signer_share = KeyShare {
            public_key: self.share.public_key.clone(),
            share_data: self.share.share_data.clone(),
        };

        Ok(Box::new(MpcSigner::new(
            signer_share,
            self.transport.clone(),
        )))
    }
}
