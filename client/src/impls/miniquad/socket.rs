use std::{collections::VecDeque, net::SocketAddr};

use naia_socket_shared::SocketConfig;

use crate::{packet_receiver::ConditionedPacketReceiver, PacketReceiver, PacketSender};

use super::{
    packet_receiver::PacketReceiverImpl,
    shared::{naia_connect, JsObject, ERROR_QUEUE, MESSAGE_QUEUE},
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
    pub fn connect(&self, server_address: SocketAddr) -> (PacketSender, Box<dyn PacketReceiver>) {
        unsafe {
            MESSAGE_QUEUE = Some(VecDeque::new());
            ERROR_QUEUE = Some(VecDeque::new());
            naia_connect(
                JsObject::string(server_address.to_string().as_str()),
                JsObject::string(self.config.rtc_endpoint_path.as_str()),
            );
        }

        let conditioner_config = self.config.link_condition_config.clone();

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
