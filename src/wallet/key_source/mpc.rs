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
        // SKELETON: assumes the share already targets the requested key (no path
        // derivation). TODO: derive per-path, which may require party communication.
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
