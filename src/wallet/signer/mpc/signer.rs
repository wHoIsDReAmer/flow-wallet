use async_trait::async_trait;
use std::sync::Arc;

use super::transport::{MpcTransport, PartyId};
use crate::wallet::Signer;
use crate::wallet::crypto::memory::SecureBuffer;

/// Placeholder for MPC key share data.
/// In a real implementation, this would contain the mathematical share.
pub struct KeyShare {
    pub public_key: Vec<u8>,
    pub share_data: SecureBuffer,
}

/// Signer that uses Multi-Party Computation to generate signatures.
pub struct MpcSigner {
    share: KeyShare,
    _transport: Arc<dyn MpcTransport>,
    _party_id: PartyId,
}

impl MpcSigner {
    pub fn new(share: KeyShare, transport: Arc<dyn MpcTransport>) -> Self {
        // Hardcoded for the mock; a real impl derives this from transport/config.
        let party_id = 1;
        Self {
            share,
            _transport: transport,
            _party_id: party_id,
        }
    }
}

#[async_trait]
impl Signer for MpcSigner {
    async fn sign(&self, digest: &[u8]) -> Result<Vec<u8>, ()> {
        // MOCK: a real signer runs a threshold protocol (e.g. GG18/CMP) over
        // `self.transport`. This prototype assumes `share_data` holds the full key
        // and signs locally. TODO: replace with the actual MPC protocol.
        let signer =
            crate::wallet::signer::local::LocalSigner::from_slice(self.share.share_data.as_ref())
                .map_err(|_| ())?;
        signer.sign(digest).await
    }

    fn public_key(&self) -> Vec<u8> {
        self.share.public_key.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wallet::signer::mpc::transport::{MpcTransport, TransportError};
    use std::sync::Mutex;

    struct MockTransport {
        id: PartyId,
        sent_messages: Arc<Mutex<Vec<(PartyId, Vec<u8>)>>>,
    }

    #[async_trait]
    impl MpcTransport for MockTransport {
        async fn send(&self, to: PartyId, data: &[u8]) -> Result<(), TransportError> {
            self.sent_messages.lock().unwrap().push((to, data.to_vec()));
            Ok(())
        }

        async fn receive(&self) -> Result<(PartyId, Vec<u8>), TransportError> {
            // Dummy receive
            Ok((0, vec![]))
        }

        fn my_party_id(&self) -> PartyId {
            self.id
        }
    }

    #[tokio::test]
    async fn test_mpc_signer_creation() {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let transport = Arc::new(MockTransport {
            id: 1,
            sent_messages: sent.clone(),
        });

        let share = KeyShare {
            public_key: vec![1, 2, 3],
            share_data: SecureBuffer::new(vec![1u8; 32]),
        };

        let signer = MpcSigner::new(share, transport);

        // Test public key retrieval
        assert_eq!(signer.public_key(), vec![1, 2, 3]);

        // Test signing (skeleton). The signer expects a 32-byte prehash digest.
        let digest = [0x42u8; 32];
        let sig = signer.sign(&digest).await.expect("sign");
        assert_eq!(sig.len(), 65); // canonical r||s||v
    }
}
