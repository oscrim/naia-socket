extern crate log;

use std::net::{SocketAddr, UdpSocket};

use naia_socket_shared::{find_my_ip_address, Ref, SocketConfig};

use crate::{packet_receiver::ConditionedPacketReceiver, PacketReceiver, PacketSender};

use super::packet_receiver::PacketReceiverImpl;

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
        let client_ip_address = find_my_ip_address().expect("cannot find current ip address");

        let socket = Ref::new(UdpSocket::bind((client_ip_address, 0)).unwrap());
        socket
            .borrow()
            .set_nonblocking(true)
            .expect("can't set socket to non-blocking!");

        let packet_sender = PacketSender::new(server_address, socket.clone());

        let conditioner_config = self.config.link_condition_config.clone();

        let sender = packet_sender.clone();
        let receiver: Box<dyn PacketReceiver> = {
            let inner_receiver = Box::new(PacketReceiverImpl::new(server_address, socket.clone()));
            if let Some(config) = &conditioner_config {
                Box::new(ConditionedPacketReceiver::new(inner_receiver, config))
            } else {
                inner_receiver
            }
        };

        (sender, receiver)
    }
}
