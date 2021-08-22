use std::collections::VecDeque;
use std::error::Error;

use log::info;

use web_sys::RtcDataChannel;

use naia_socket_shared::Ref;

use crate::Packet;

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
        self.resend_dropped_messages();

        if let Err(_) = self.data_channel.send_with_u8_array(&packet.payload()) {
            self.dropped_outgoing_messages
                .borrow_mut()
                .push_back(packet);
        }
        Ok(())
    }

    fn resend_dropped_messages(&mut self) {
        if !self.dropped_outgoing_messages.borrow().is_empty() {
            if let Some(dropped_packets) = {
                let mut dom = self.dropped_outgoing_messages.borrow_mut();
                let dropped_packets: Vec<Packet> = dom.drain(..).collect::<Vec<Packet>>();
                Some(dropped_packets)
            } {
                for dropped_packet in dropped_packets {
                    self.send(dropped_packet).unwrap_or_else(|err| {
                        info!("Can't send dropped packet. Original Error: {:?}", err)
                    });
                }
            }
        }
    }
}
