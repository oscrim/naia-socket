use crossbeam::channel::Sender;

use super::packet::Packet;

/// Used to send packets to the Server Socket
#[derive(Debug, Clone)]
pub struct PacketSender {
    channel_sender: Sender<Packet>,
}

impl PacketSender {
    /// Creates a new PacketSender
    pub fn new(channel_sender: Sender<Packet>) -> Self {
        PacketSender { channel_sender }
    }

    /// Sends a packet to the Server Socket
    pub fn send(&self, packet: Packet) {
        self.channel_sender.send(packet).unwrap(); //TODO: handle result..
    }
}
