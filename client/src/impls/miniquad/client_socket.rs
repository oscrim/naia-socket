use std::{collections::VecDeque, net::SocketAddr};

use naia_socket_shared::LinkConditionerConfig;

use crate::{
    ClientSocketConfig, ClientSocketTrait, PacketReceiver, PacketReceiverTrait, PacketSender,
};

use super::shared::{naia_connect, JsObject, ERROR_QUEUE, MESSAGE_QUEUE};

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
    pub fn connect(client_config: ClientSocketConfig) -> Self {
        unsafe {
            MESSAGE_QUEUE = Some(VecDeque::new());
            ERROR_QUEUE = Some(VecDeque::new());
            naia_connect(
                JsObject::string(client_config.server_address.to_string().as_str()),
                JsObject::string(client_config.shared.rtc_endpoint_path.as_str()),
            );
        }

        let client_socket = ClientSocket {
            address: client_config.server_address,
            packet_sender: PacketSender::new(),
            link_conditioner_config: client_config.shared.link_condition_config.clone(),
        };

        client_socket
    }
}

impl ClientSocketTrait for ClientSocket {
    fn get_receiver(&self) -> Box<dyn PacketReceiverTrait> {
        match &self.link_conditioner_config {
            Some(_config) => Box::new(PacketReceiver::new()),
            None => Box::new(PacketReceiver::new()),
        }
    }

    fn get_sender(&self) -> PacketSender {
        return self.packet_sender.clone();
    }

    fn with_link_conditioner(mut self, config: &LinkConditionerConfig) -> Self {
        self.link_conditioner_config = Some(config.clone());
        return self;
    }
}
