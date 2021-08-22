extern crate log;

use std::{collections::VecDeque, net::SocketAddr};

use naia_socket_shared::{LinkConditionerConfig, Ref};

use crate::{
    packet_receiver::ConditionedPacketReceiver, ClientSocketConfig, ClientSocketTrait, Packet,
    PacketReceiver, PacketSender,
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
    pub fn connect(client_config: ClientSocketConfig) -> Self {
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

        let client_socket = ClientSocket {
            address: client_config.server_address,
            message_queue,
            packet_sender,
            packet_receiver,
            dropped_outgoing_messages,
            link_conditioner_config: client_config.shared.link_condition_config.clone(),
        };

        client_socket
    }
}

#[allow(unsafe_code)]
#[cfg(feature = "multithread")]
unsafe impl Send for ClientSocket {}
#[allow(unsafe_code)]
#[cfg(feature = "multithread")]
unsafe impl Sync for ClientSocket {}

impl ClientSocketTrait for ClientSocket {
    fn get_receiver(&self) -> Box<dyn PacketReceiver> {
        let inner_receiver = Box::new(self.packet_receiver.clone());
        if let Some(config) = &self.link_conditioner_config {
            return Box::new(ConditionedPacketReceiver::new(inner_receiver, config));
        } else {
            return inner_receiver;
        }
    }

    fn get_sender(&self) -> PacketSender {
        return self.packet_sender.clone();
    }
}
