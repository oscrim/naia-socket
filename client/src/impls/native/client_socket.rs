extern crate log;

use std::net::{SocketAddr, UdpSocket};

use naia_socket_shared::{find_my_ip_address, LinkConditionerConfig, Ref};

use crate::{
    ClientSocketConfig, ClientSocketTrait, PacketReceiver, PacketReceiverTrait, PacketSender,
};

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
    fn get_receiver(&self) -> Box<dyn PacketReceiverTrait> {
        match &self.link_conditioner_config {
            Some(_config) => Box::new(PacketReceiver::new(self.address, self.socket.clone())),
            None => Box::new(PacketReceiver::new(self.address, self.socket.clone())),
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
