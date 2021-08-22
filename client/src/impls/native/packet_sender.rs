use std::net::{SocketAddr, UdpSocket};

use crate::Packet;
use naia_socket_shared::Ref;
use std::error::Error;

/// Handles sending messages to the Server for a given Client Socket
#[derive(Clone, Debug)]
pub struct PacketSender {
    address: SocketAddr,
    socket: Ref<UdpSocket>,
}

impl PacketSender {
    /// Create a new PacketSender, if supplied with the Server's address & a
    /// reference back to the parent Socket
    pub fn new(address: SocketAddr, socket: Ref<UdpSocket>) -> Self {
        PacketSender { address, socket }
    }

    /// Send a Packet to the Server
    pub fn send(&mut self, packet: Packet) -> Result<(), Box<dyn Error + Send + Sync>> {
        //send it
        if let Err(err) = self
            .socket
            .borrow()
            .send_to(&packet.payload(), self.address)
        {
            return Err(Box::new(err));
        } else {
            return Ok(());
        }
    }
}
