extern crate log;

use std::{cell::RefCell, collections::VecDeque, net::SocketAddr, rc::Rc};

use naia_socket_shared::SocketConfig;

use crate::packet_receiver::{ConditionedPacketReceiver, PacketReceiver, PacketReceiverTrait};

use super::{
    packet_receiver::PacketReceiverImpl, packet_sender::PacketSender,
    webrtc_internal::webrtc_initialize,
};

/// A client-side socket which communicates with an underlying unordered &
/// unreliable protocol

pub struct Socket {
    config: SocketConfig,
    io: Option<Io>,
}

/// Contains internal socket packet sender/receiver

struct Io {
    /// Used to send packets through the socket
    pub packet_sender: PacketSender,
    /// Used to receive packets from the socket
    pub packet_receiver: PacketReceiver,
}

impl Socket {
    /// Create a new Socket
    pub fn new(config: SocketConfig) -> Self {
        Socket { config, io: None }
    }

    /// Connects to the given server address
    pub fn connect(&mut self, server_address: SocketAddr) {
        if self.io.is_some() {
            panic!("Socket already listening!");
        }

        let message_queue = Rc::new(RefCell::new(VecDeque::new()));
        let data_channel = webrtc_initialize(
            server_address,
            self.config.rtc_endpoint_path.clone(),
            message_queue.clone(),
        );

        let dropped_outgoing_messages = Rc::new(RefCell::new(VecDeque::new()));

        let packet_sender =
            PacketSender::new(data_channel.clone(), dropped_outgoing_messages.clone());
        let packet_receiver = PacketReceiverImpl::new(
            data_channel.clone(),
            dropped_outgoing_messages.clone(),
            message_queue.clone(),
        );

        let sender = packet_sender.clone();
        let receiver: Box<dyn PacketReceiverTrait> = {
            let inner_receiver = Box::new(packet_receiver.clone());
            if let Some(config) = &self.config.link_condition_config {
                Box::new(ConditionedPacketReceiver::new(inner_receiver, config))
            } else {
                inner_receiver
            }
        };

        self.io = Some(Io {
            packet_sender: sender,
            packet_receiver: PacketReceiver::new(receiver),
        });
    }

    /// Gets a PacketSender which can be used to send packets through the Socket
    pub fn get_packet_sender(&self) -> PacketSender {
        return self
            .io
            .as_ref()
            .expect("Socket is not connected yet! Call Socket.connect() before this.")
            .packet_sender
            .clone();
    }

    /// Gets a PacketReceiver which can be used to receive packets from the
    /// Socket
    pub fn get_packet_receiver(&self) -> PacketReceiver {
        return self
            .io
            .as_ref()
            .expect("Socket is not connected yet! Call Socket.connect() before this.")
            .packet_receiver
            .clone();
    }
}

unsafe impl Send for Socket {}
unsafe impl Sync for Socket {}
