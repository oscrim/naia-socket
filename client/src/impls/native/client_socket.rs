extern crate log;

use std::net::{SocketAddr, UdpSocket};

use naia_socket_shared::{find_my_ip_address, LinkConditionerConfig, Ref};

use crate::{
    packet_receiver::ConditionedPacketReceiver, ClientSocketConfig, ClientSocketTrait,
    PacketReceiver, PacketSender,
};

use super::packet_receiver::PacketReceiverImpl;

/// A client-side socket which communicates with an underlying unordered &
/// unreliable protocol
#[derive(Debug)]
pub struct ClientSocket {
    address: SocketAddr,
    socket: Ref<UdpSocket>,
    packet_sender: PacketSender,
    link_conditioner_config: Option<LinkConditionerConfig>,
}

impl ClientSocket {
    /// Returns a new ClientSocket, connected to the given socket address
    pub fn connect(client_config: ClientSocketConfig) -> Self {
        let client_ip_address = find_my_ip_address().expect("cannot find current ip address");

        let socket = Ref::new(UdpSocket::bind((client_ip_address, 0)).unwrap());
        socket
            .borrow()
            .set_nonblocking(true)
            .expect("can't set socket to non-blocking!");

        let packet_sender = PacketSender::new(client_config.server_address, socket.clone());

        let client_socket = ClientSocket {
            packet_sender,
            address: client_config.server_address,
            socket,
            link_conditioner_config: client_config.shared.link_condition_config.clone(),
        };

        client_socket
    }
}

impl ClientSocketTrait for ClientSocket {
    fn get_receiver(&self) -> Box<dyn PacketReceiver> {
        let inner_receiver = Box::new(PacketReceiverImpl::new(self.address, self.socket.clone()));
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
