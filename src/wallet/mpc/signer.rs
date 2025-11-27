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
    transport: Arc<dyn MpcTransport>,
    party_id: PartyId,
}

impl MpcSigner {
    pub fn new(share: KeyShare, transport: Arc<dyn MpcTransport>) -> Self {
        let party_id = transport.my_party_id();
        Self {
            share,
            transport,
            party_id,
        }
    }
}

#[async_trait]
impl Signer for MpcSigner {
    async fn sign(&self, message: &[u8]) -> Result<Vec<u8>, ()> {
        // TODO: Implement actual MPC signing protocol (e.g. GG18).
        // For now, we simulate a protocol round.

        // 1. Broadcast "I am ready to sign"
        // In a real protocol, this would be the first round message.
        let payload = b"init_sign";
        // We don't know other parties here yet, assuming broadcast or specific logic.
        // For this skeleton, we just return a dummy signature.

        Ok(vec![0xde, 0xad, 0xbe, 0xef])
    }

    fn public_key(&self) -> Vec<u8> {
        self.share.public_key.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wallet::mpc::transport::{MpcTransport, TransportError};
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
            share_data: SecureBuffer::new(vec![4, 5, 6]),
        };

        let signer = MpcSigner::new(share, transport);

        // Test public key retrieval
        assert_eq!(signer.public_key(), vec![1, 2, 3]);

        // Test signing (skeleton)
        let sig = signer.sign(b"test").await.expect("sign");
        assert_eq!(sig, vec![0xde, 0xad, 0xbe, 0xef]);
    }
}
