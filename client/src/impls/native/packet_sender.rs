use std::{
    net::{SocketAddr, UdpSocket},
    sync::{Arc, Mutex},
};

use crate::Packet;

/// Handles sending messages to the Server for a given Client Socket
#[derive(Clone)]
pub struct PacketSender {
    address: SocketAddr,
    socket: Arc<Mutex<UdpSocket>>,
}

impl PacketSender {
    /// Create a new PacketSender, if supplied with the Server's address & a
    /// reference back to the parent Socket
    pub fn new(address: SocketAddr, socket: Arc<Mutex<UdpSocket>>) -> Self {
        PacketSender { address, socket }
    }

    /// Send a Packet to the Server
    pub fn send(&mut self, packet: Packet) {
        //send it
        if let Err(_) = self
            .socket
            .as_ref()
            .lock()
            .unwrap()
            .send_to(&packet.payload(), self.address)
        {
            //TODO: handle this error
        }
    }
}
