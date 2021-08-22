extern crate log;
use log::info;

use std::{collections::VecDeque, net::SocketAddr};

use crate::{
    error::NaiaClientSocketError, link_conditioner::LinkConditioner, ClientSocketConfig,
    ClientSocketTrait, MessageSender, Packet,
};

use naia_socket_shared::{LinkConditionerConfig, Ref};

use super::webrtc_internal::webrtc_initialize;

/// A client-side socket which communicates with an underlying unordered &
/// unreliable protocol
#[derive(Debug)]
pub struct ClientSocket {
    address: SocketAddr,
    message_queue: Ref<VecDeque<Result<Option<Packet>, NaiaClientSocketError>>>,
    packet_sender: PacketSender,
    dropped_outgoing_messages: Ref<VecDeque<Packet>>,
}

impl ClientSocket {
    /// Returns a new ClientSocket, connected to the given socket address
    pub fn connect(client_config: ClientSocketConfig) -> Box<dyn ClientSocketTrait> {
        let message_queue = Ref::new(VecDeque::new());
        let data_channel = webrtc_initialize(
            client_config.server_address,
            client_config.shared.rtc_endpoint_path,
            message_queue.clone(),
        );

        let dropped_outgoing_messages = Ref::new(VecDeque::new());

        let packet_sender =
            PacketSender::new(data_channel.clone(), dropped_outgoing_messages.clone());

        let mut client_socket: Box<dyn ClientSocketTrait> = Box::new(ClientSocket {
            address: client_config.server_address,
            message_queue,
            packet_sender,
            dropped_outgoing_messages,
        });

        if let Some(config) = &client_config.shared.link_condition_config {
            client_socket = client_socket.with_link_conditioner(config);
        }

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
    fn receive(&mut self) -> Result<Option<Packet>, NaiaClientSocketError> {
        if !self.dropped_outgoing_messages.borrow().is_empty() {
            if let Some(dropped_packets) = {
                let mut dom = self.dropped_outgoing_messages.borrow_mut();
                let dropped_packets: Vec<Packet> = dom.drain(..).collect::<Vec<Packet>>();
                Some(dropped_packets)
            } {
                for dropped_packet in dropped_packets {
                    self.message_sender
                        .send(dropped_packet)
                        .unwrap_or_else(|err| {
                            info!("Can't send dropped packet. Original Error: {:?}", err)
                        });
                }
            }
        }

        loop {
            if self.message_queue.borrow().is_empty() {
                return Ok(None);
            }

            match self
                .message_queue
                .borrow_mut()
                .pop_front()
                .expect("message queue shouldn't be empty!")
            {
                Ok(Some(packet)) => {
                    return Ok(Some(packet));
                }
                Ok(inner) => {
                    return Ok(inner);
                }
                Err(err) => {
                    return Err(err);
                }
            }
        }
    }

    fn get_sender(&mut self) -> PacketSender {
        return self.packet_sender.clone();
    }

    fn with_link_conditioner(
        self: Box<Self>,
        config: &LinkConditionerConfig,
    ) -> Box<dyn ClientSocketTrait> {
        Box::new(LinkConditioner::new(config, self))
    }
}
