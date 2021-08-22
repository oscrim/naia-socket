use std::fmt::Debug;

use crossbeam::{
    channel,
    channel::{Receiver, Sender},
};

use futures_util::SinkExt;

use naia_socket_shared::LinkConditionerConfig;

use super::{
    async_server_socket::AsyncServerSocketTrait,
    packet::Packet,
    packet_receiver::{ConditionedPacketReceiver, PacketReceiver, PacketReceiverTrait},
    packet_sender::PacketSender,
};
use crate::{
    error::NaiaServerSocketError, executor, impls::ServerSocket as AsyncServerSocket,
    ServerSocketConfig,
};

/// Defines the functionality of a Naia Server Socket
pub trait ServerSocketTrait: Debug + Send + Sync {
    /// Gets a MessageReceiver you can use to receive messages from the Server
    /// Socket
    fn get_receiver(&self) -> Box<dyn PacketReceiverTrait>;
    /// Gets a MessageSender you can use to send messages through the Server
    /// Socket
    fn get_sender(&self) -> PacketSender;
    /// Wraps the current socket in a LinkConditioner
    fn with_link_conditioner(self, config: &LinkConditionerConfig) -> Self;
}

/// Server Socket is able to send and receive messages from remote Clients
#[derive(Debug)]
pub struct ServerSocket {
    to_client_sender: Sender<Packet>,
    from_client_receiver: Receiver<Result<Packet, NaiaServerSocketError>>,
    link_conditioner_config: Option<LinkConditionerConfig>,
}

impl ServerSocket {
    /// Returns a new ServerSocket, listening at the given socket addresses
    pub fn listen(server_socket_config: ServerSocketConfig) -> Self {
        // Set up receiver loop
        let (from_client_sender, from_client_receiver) = channel::unbounded();
        let (sender_sender, sender_receiver) = channel::bounded(1);

        let shared_config = server_socket_config.clone();

        executor::spawn(async move {
            // Create async socket
            let mut async_socket = AsyncServerSocket::listen(shared_config).await;

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

        let socket = ServerSocket {
            to_client_sender,
            from_client_receiver,
            link_conditioner_config: server_socket_config.shared.link_condition_config.clone(),
        };

        socket
    }
}

impl ServerSocketTrait for ServerSocket {
    fn get_receiver(&self) -> Box<dyn PacketReceiverTrait> {
        match &self.link_conditioner_config {
            Some(config) => Box::new(ConditionedPacketReceiver::new(
                self.from_client_receiver.clone(),
                config,
            )),
            None => Box::new(PacketReceiver::new(self.from_client_receiver.clone())),
        }
    }

    fn get_sender(&self) -> PacketSender {
        PacketSender::new(self.to_client_sender.clone())
    }

    fn with_link_conditioner(mut self, config: &LinkConditionerConfig) -> Self {
        self.link_conditioner_config = Some(config.clone());
        return self;
    }
}
