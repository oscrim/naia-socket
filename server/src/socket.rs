use std::fmt::Debug;

use crossbeam::channel;

use futures_util::SinkExt;

use naia_socket_shared::SocketConfig;

use crate::{executor, impls::Socket as AsyncSocket};

use super::{
    async_socket::AsyncSocketTrait,
    packet_receiver::{
        ConditionedPacketReceiverImpl, PacketReceiver, PacketReceiverImpl, PacketReceiverTrait,
    },
    packet_sender::PacketSender,
    server_addrs::ServerAddrs,
};

/// Socket is able to send and receive messages from remote Clients
#[derive(Debug)]
pub struct Socket {
    config: SocketConfig,
    io: Option<Io>,
}

/// Contains internal socket packet sender/receiver
#[derive(Debug)]
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

    /// Listens on the Socket for incoming communication from Clients
    pub fn listen(&mut self, server_addrs: ServerAddrs) {
        if self.io.is_some() {
            panic!("Socket already listening!");
        }

        // Set up receiver loop
        let (from_client_sender, from_client_receiver) = channel::unbounded();
        let (sender_sender, sender_receiver) = channel::bounded(1);

        let server_addrs_clone = server_addrs.clone();
        let config_clone = self.config.clone();

        executor::spawn(async move {
            // Create async socket
            let mut async_socket = AsyncSocket::listen(server_addrs_clone, config_clone).await;

            sender_sender.send(async_socket.get_sender()).unwrap(); //TODO: handle result..

            loop {
                let out_message = async_socket.receive().await;
                from_client_sender.send(out_message).unwrap(); //TODO: handle
                                                               // result..
            }
        })
        .detach();

        // Set up sender loop
        let (to_client_sender, to_client_receiver) = channel::unbounded();

        executor::spawn(async move {
            // Create async socket
            let mut async_sender = sender_receiver.recv().unwrap();

            loop {
                if let Ok(msg) = to_client_receiver.recv() {
                    async_sender.send(msg).await.unwrap(); //TODO: handle
                                                           // result..
                }
            }
        })
        .detach();

        let conditioner_config = self.config.link_condition_config.clone();

        let receiver: Box<dyn PacketReceiverTrait> = match &conditioner_config {
            Some(config) => Box::new(ConditionedPacketReceiverImpl::new(
                from_client_receiver.clone(),
                config,
            )),
            None => Box::new(PacketReceiverImpl::new(from_client_receiver.clone())),
        };
        let sender = PacketSender::new(to_client_sender.clone());

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
            .expect("Socket is not listening yet! Call Socket.listen() before this.")
            .packet_sender
            .clone();
    }

    /// Gets a PacketReceiver which can be used to receive packets from the
    /// Socket
    pub fn get_packet_receiver(&self) -> PacketReceiver {
        return self
            .io
            .as_ref()
            .expect("Socket is not listening yet! Call Socket.listen() before this.")
            .packet_receiver
            .clone();
    }
}
