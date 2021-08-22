use std::fmt::Debug;

use super::{error::NaiaClientSocketError, packet::Packet};

/// Used to receive packets from the Client Socket
pub trait PacketReceiverTrait: Debug {
    /// Receives a packet from the Client Socket
    fn receive(&mut self) -> Result<Option<Packet>, NaiaClientSocketError>;
}
