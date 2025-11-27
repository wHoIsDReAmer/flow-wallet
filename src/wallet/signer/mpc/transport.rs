use async_trait::async_trait;
use thiserror::Error;

pub type PartyId = u16;

#[derive(Debug, Error)]
pub enum TransportError {
    #[error("send failed: {0}")]
    SendError(String),
    #[error("receive failed: {0}")]
    ReceiveError(String),
}

/// Abstract transport for MPC communication.
#[async_trait]
pub trait MpcTransport: Send + Sync {
    /// Send data to a specific party.
    async fn send(&self, to: PartyId, data: &[u8]) -> Result<(), TransportError>;

    /// Receive data from any party.
    /// Returns (sender_id, data).
    async fn receive(&self) -> Result<(PartyId, Vec<u8>), TransportError>;

    /// Get the ID of this party.
    fn my_party_id(&self) -> PartyId;
}
