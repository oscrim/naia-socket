extern crate log;

use std::net::{SocketAddr, UdpSocket};

use naia_socket_shared::{find_my_ip_address, LinkConditionerConfig, Ref};

use crate::{
    packet_receiver::ConditionedPacketReceiver, ClientSocketConfig, PacketReceiver, PacketSender,
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
    pub fn connect(client_config: ClientSocketConfig) -> (PacketSender, Box<dyn PacketReceiver>) {
        let client_ip_address = find_my_ip_address().expect("cannot find current ip address");

        let socket = Ref::new(UdpSocket::bind((client_ip_address, 0)).unwrap());
        socket
            .borrow()
            .set_nonblocking(true)
            .expect("can't set socket to non-blocking!");

        let packet_sender = PacketSender::new(client_config.server_address, socket.clone());

        let conditioner_config = client_config.shared.link_condition_config.clone();

        let sender = packet_sender.clone();
        let receiver: Box<dyn PacketReceiver> = {
            let inner_receiver = Box::new(PacketReceiverImpl::new(
                client_config.server_address,
                socket.clone(),
            ));
            if let Some(config) = &conditioner_config {
                Box::new(ConditionedPacketReceiver::new(inner_receiver, config))
            } else {
                inner_receiver
            }
        };

        (sender, receiver)
    }
}
