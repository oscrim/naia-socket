use std::{collections::VecDeque, net::SocketAddr};

use super::shared::{
    naia_connect, naia_resend_dropped_messages, JsObject, ERROR_QUEUE, MESSAGE_QUEUE,
};

use crate::{
    error::NaiaClientSocketError, link_conditioner::LinkConditioner, ClientSocketConfig,
    ClientSocketTrait, MessageSender, Packet,
};

use naia_socket_shared::LinkConditionerConfig;

/// A client-side socket which communicates with an underlying unordered &
/// unreliable protocol
#[derive(Debug)]
pub struct ClientSocket {
    address: SocketAddr,
    packet_sender: PacketSender,
}

impl ClientSocket {
    /// Returns a new ClientSocket, connected to the given socket address
    pub fn connect(client_config: ClientSocketConfig) -> Box<dyn ClientSocketTrait> {
        unsafe {
            MESSAGE_QUEUE = Some(VecDeque::new());
            ERROR_QUEUE = Some(VecDeque::new());
            naia_connect(
                JsObject::string(client_config.server_address.to_string().as_str()),
                JsObject::string(client_config.shared.rtc_endpoint_path.as_str()),
            );
        }

        let mut client_socket: Box<dyn ClientSocketTrait> = Box::new(ClientSocket {
            address: client_config.server_address,
            packet_sender: PacketSender::new(),
        });

        if let Some(config) = &client_config.shared.link_condition_config {
            client_socket = client_socket.with_link_conditioner(config);
        }

        client_socket
    }
}

impl ClientSocketTrait for ClientSocket {
    fn receive(&mut self) -> Result<Option<Packet>, NaiaClientSocketError> {
        unsafe {
            naia_resend_dropped_messages();

            if let Some(msg_queue) = &mut MESSAGE_QUEUE {
                if let Some(message) = msg_queue.pop_front() {
                    return Ok(Some(Packet::new_raw(message)));
                }
            }

            if let Some(error_queue) = &mut ERROR_QUEUE {
                if let Some(error) = error_queue.pop_front() {
                    return Err(NaiaClientSocketError::Message(error));
                }
            }
        };

        Ok(None)
    }

    fn get_sender(&mut self) -> Packet {
        return self.packet_sender.clone();
    }

    fn with_link_conditioner(
        self: Box<Self>,
        config: &LinkConditionerConfig,
    ) -> Box<dyn ClientSocketTrait> {
        Box::new(LinkConditioner::new(config, self))
    }
}
