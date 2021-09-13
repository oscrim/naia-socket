use std::fmt::Debug;

use crossbeam::channel;

use futures_util::SinkExt;

use naia_socket_shared::SocketConfig;

use crate::{executor, impls::Socket as AsyncSocket};

use super::{
    async_socket::AsyncSocketTrait,
    packet_receiver::{ConditionedPacketReceiverImpl, PacketReceiver, PacketReceiverImpl},
    packet_sender::PacketSender,
    server_addrs::ServerAddrs,
};

/// Socket is able to send and receive messages from remote Clients
#[derive(Debug)]
pub struct Socket {
    config: SocketConfig,
}

impl Socket {
    /// Create a new Socket
    pub fn new(config: SocketConfig) -> Self {
        Socket { config }
    }

    /// Listens on the Socket for incoming communication from Clients
    pub fn listen(&self, server_addrs: ServerAddrs) -> (PacketSender, Box<dyn PacketReceiver>) {
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

        let receiver: Box<dyn PacketReceiver> = match &conditioner_config {
            Some(config) => Box::new(ConditionedPacketReceiverImpl::new(
                from_client_receiver.clone(),
                config,
            )),
            None => Box::new(PacketReceiverImpl::new(from_client_receiver.clone())),
        };
        let sender = PacketSender::new(to_client_sender.clone());

        (sender, receiver)
    }
}
