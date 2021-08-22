use std::collections::VecDeque;

use crate::Packet;
use naia_socket_shared::Ref;
use std::error::Error;
use web_sys::RtcDataChannel;

/// Handles sending messages to the Server for a given Client Socket
#[derive(Clone, Debug)]
pub struct PacketSender {
    data_channel: RtcDataChannel,
    dropped_outgoing_messages: Ref<VecDeque<Packet>>,
}

impl PacketSender {
    /// Create a new PacketSender, if supplied with the RtcDataChannel and a
    /// reference to a list of dropped messages
    pub fn new(
        data_channel: RtcDataChannel,
        dropped_outgoing_messages: Ref<VecDeque<Packet>>,
    ) -> Self {
        PacketSender {
            data_channel,
            dropped_outgoing_messages,
        }
    }

    /// Send a Packet to the Server
    pub fn send(&mut self, packet: Packet) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Err(_) = self.data_channel.send_with_u8_array(&packet.payload()) {
            self.dropped_outgoing_messages
                .borrow_mut()
                .push_back(packet);
        }
        Ok(())
    }
}
