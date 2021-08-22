use std::{collections::VecDeque, net::SocketAddr};

use naia_socket_shared::LinkConditionerConfig;

use crate::{
    packet_receiver::ConditionedPacketReceiver, ClientSocketConfig, PacketReceiver, PacketSender,
};

use super::{
    packet_receiver::PacketReceiverImpl,
    shared::{naia_connect, JsObject, ERROR_QUEUE, MESSAGE_QUEUE},
};

/// A client-side socket which communicates with an underlying unordered &
/// unreliable protocol
#[derive(Debug)]
pub struct ClientSocket {
    address: SocketAddr,
    packet_sender: PacketSender,
    link_conditioner_config: Option<LinkConditionerConfig>,
}

impl ClientSocket {
    /// Returns a new ClientSocket, connected to the given socket address
    pub fn connect(client_config: ClientSocketConfig) -> (PacketSender, Box<dyn PacketReceiver>) {
        unsafe {
            MESSAGE_QUEUE = Some(VecDeque::new());
            ERROR_QUEUE = Some(VecDeque::new());
            naia_connect(
                JsObject::string(client_config.server_address.to_string().as_str()),
                JsObject::string(client_config.shared.rtc_endpoint_path.as_str()),
            );
        }

        let conditioner_config = client_config.shared.link_condition_config.clone();

        let sender = PacketSender::new();
        let receiver: Box<dyn PacketReceiver> = {
            let inner_receiver = Box::new(PacketReceiverImpl::new());
            if let Some(config) = &conditioner_config {
                Box::new(ConditionedPacketReceiver::new(inner_receiver, config))
            } else {
                inner_receiver
            }
        };

        (sender, receiver)
    }
}
