use crossbeam::channel::Receiver;

use super::{error::NaiaServerSocketError, packet::Packet};

/// Used to receive packets from the Server Socket
#[derive(Debug, Clone)]
pub struct PacketReceiver {
    channel_receiver: Receiver<Result<Packet, NaiaServerSocketError>>,
}

impl PacketReceiver {
    /// Creates a new PacketReceiver
    pub fn new(channel_receiver: Receiver<Result<Packet, NaiaServerSocketError>>) -> Self {
        PacketReceiver { channel_receiver }
    }
    /// Receives a packet from the Server Socket
    pub fn receive(&self) -> Result<Option<Packet>, NaiaServerSocketError> {
        match self.channel_receiver.try_recv() {
            Ok(result) => match result {
                Ok(packet) => return Ok(Some(packet)),
                Err(_) => return Ok(None),
            },
            Err(_) => {
                return Ok(None);
            }
        }
    }
}
