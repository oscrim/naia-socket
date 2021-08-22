extern crate log;

use std::{collections::VecDeque, net::SocketAddr};

use naia_socket_shared::{LinkConditionerConfig, Ref};

use crate::{
    packet_receiver::ConditionedPacketReceiver, ClientSocketConfig, Packet, PacketReceiver,
    PacketSender,
};

use super::{packet_receiver::PacketReceiverImpl, webrtc_internal::webrtc_initialize};

/// A client-side socket which communicates with an underlying unordered &
/// unreliable protocol
#[derive(Debug)]
pub struct ClientSocket {
    address: SocketAddr,
    message_queue: Ref<VecDeque<Packet>>,
    packet_sender: PacketSender,
    packet_receiver: PacketReceiverImpl,
    dropped_outgoing_messages: Ref<VecDeque<Packet>>,
    link_conditioner_config: Option<LinkConditionerConfig>,
}

impl ClientSocket {
    /// Returns a new ClientSocket, connected to the given socket address
    pub fn connect(client_config: ClientSocketConfig) -> (PacketSender, Box<dyn PacketReceiver>) {
        let message_queue = Ref::new(VecDeque::new());
        let data_channel = webrtc_initialize(
            client_config.server_address,
            client_config.shared.rtc_endpoint_path,
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
        let receiver: Box<dyn PacketReceiver> = {
            let inner_receiver = Box::new(packet_receiver.clone());
            if let Some(config) = &client_config.shared.link_condition_config {
                Box::new(ConditionedPacketReceiver::new(inner_receiver, config))
            } else {
                inner_receiver
            }
        };

        (sender, receiver)
    }
}

#[allow(unsafe_code)]
#[cfg(feature = "multithread")]
unsafe impl Send for ClientSocket {}
#[allow(unsafe_code)]
#[cfg(feature = "multithread")]
unsafe impl Sync for ClientSocket {}
