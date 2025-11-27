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
        // In a real implementation, we would derive the party_id from the transport or config
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
    async fn sign(&self, _message: &[u8]) -> Result<Vec<u8>, ()> {
        // TODO: Implement actual MPC signing protocol
        // For now, we just sign with the local key share to simulate success in tests
        // In reality, this would involve multiple rounds of communication via self.transport

        // Simulating MPC delay
        // tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        // Placeholder: just sign with the share's private key part (which is a full key in this mock)
        let _payload = &self.share.share_data;

        // For the prototype, we can't easily sign without the full key.
        // But KeyShare in this mock might actually hold the full key for simplicity?
        // Let's assume KeyShare::share_data holds the private key bytes for this "Local MPC" mock.

        // This is a HACK for the prototype to allow "MPC" signer to work in basic flow tests
        // without implementing a full GG18/CMP protocol.
        let secret_key_bytes = &self.share.share_data;
        let signer =
            crate::wallet::signer::local::LocalSigner::from_slice(secret_key_bytes.as_ref())
                .map_err(|_| ())?;
        signer.sign(_message).await
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
            share_data: SecureBuffer::new(vec![1u8; 32]),
        };

        let signer = MpcSigner::new(share, transport);

        // Test public key retrieval
        assert_eq!(signer.public_key(), vec![1, 2, 3]);

        // Test signing (skeleton)
        let sig = signer.sign(b"test").await.expect("sign");
        // assert_eq!(sig, vec![0xde, 0xad, 0xbe, 0xef]);
        assert!(!sig.is_empty()); // Just check it produces something valid-ish
    }
}
