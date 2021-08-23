use std::fmt::Debug;

use crossbeam::channel;

use futures_util::SinkExt;

use super::{
    async_server_socket::AsyncServerSocketTrait,
    packet_receiver::{ConditionedPacketReceiverImpl, PacketReceiver, PacketReceiverImpl},
    packet_sender::PacketSender,
};
use crate::{executor, impls::ServerSocket as AsyncServerSocket, ServerSocketConfig};

/// Server Socket is able to send and receive messages from remote Clients
#[derive(Debug)]
pub struct ServerSocket;

impl ServerSocket {
    /// Returns a new ServerSocket, listening at the given socket addresses
    pub fn listen(
        server_socket_config: ServerSocketConfig,
    ) -> (PacketSender, Box<dyn PacketReceiver>) {
        // Set up receiver loop
        let (from_client_sender, from_client_receiver) = channel::unbounded();
        let (sender_sender, sender_receiver) = channel::bounded(1);

        let server_config = server_socket_config.clone();

        executor::spawn(async move {
            // Create async socket
            let mut async_socket = AsyncServerSocket::listen(server_config).await;

            sender_sender.send(async_socket.get_sender()).unwrap(); //TODO: handle result..

            loop {
                let out_message = async_socket.receive().await;
                from_client_sender.send(out_message).unwrap(); //TODO: handle result..
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
                    async_sender.send(msg).await.unwrap(); //TODO: handle result..
                }
            }
        })
        .detach();

        let conditioner_config = server_socket_config.shared.link_condition_config.clone();

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
