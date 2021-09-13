extern crate log;

use std::{collections::VecDeque, net::SocketAddr};

use naia_socket_shared::{Ref, SocketConfig};

use crate::packet_receiver::{ConditionedPacketReceiver, PacketReceiver, PacketReceiverTrait};

use super::{
    packet_receiver::PacketReceiverImpl, packet_sender::PacketSender,
    webrtc_internal::webrtc_initialize,
};

/// A client-side socket which communicates with an underlying unordered &
/// unreliable protocol
#[derive(Debug)]
pub struct Socket {
    config: SocketConfig,
}

impl Socket {
    /// Create a new Socket
    pub fn new(config: SocketConfig) -> Self {
        Socket { config }
    }

    /// Connects to the given server address
    pub fn connect(&self, server_address: SocketAddr) -> (PacketSender, PacketReceiver) {
        let message_queue = Ref::new(VecDeque::new());
        let data_channel = webrtc_initialize(
            server_address,
            self.config.rtc_endpoint_path.clone(),
            message_queue.clone(),
        );

        let dropped_outgoing_messages = Ref::new(VecDeque::new());

        let packet_sender =
            PacketSender::new(data_channel.clone(), dropped_outgoing_messages.clone());
        let packet_receiver = PacketReceiverImpl::new(
            data_channel.clone(),
            dropped_outgoing_messages.clone(),
            message_queue.clone(),
        );

        let sender = packet_sender.clone();
        let receiver: Box<dyn PacketReceiverTrait> = {
            let inner_receiver = Box::new(packet_receiver.clone());
            if let Some(config) = &self.config.link_condition_config {
                Box::new(ConditionedPacketReceiver::new(inner_receiver, config))
            } else {
                inner_receiver
            }
        };

        (sender, PacketReceiver::new(receiver))
    }
}

#[allow(unsafe_code)]
#[cfg(feature = "multithread")]
unsafe impl Send for Socket {}
#[allow(unsafe_code)]
#[cfg(feature = "multithread")]
unsafe impl Sync for Socket {}
