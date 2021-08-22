use async_trait::async_trait;

use super::packet::Packet;
use crate::error::NaiaServerSocketError;

/// Defines the functionality of the inner async Naia Server Socket
#[async_trait]
pub trait AsyncServerSocketTrait: Send + Sync {
    /// Receive a new packet from the socket, or a tick event
    async fn receive(&mut self) -> Result<Packet, NaiaServerSocketError>;
    /// Gets a MessageSender you can use to send messages through the Server
    /// Socket
    fn get_sender(&self) -> futures_channel::mpsc::Sender<Packet>;
}
