use std::{fmt::Debug, net::SocketAddr};

use crossbeam::{
    channel,
    channel::{Receiver, Sender},
};

use futures_util::SinkExt;

use naia_socket_shared::LinkConditionerConfig;

use super::{
    async_server_socket::AsyncServerSocketTrait, packet::Packet, packet_receiver::PacketReceiver,
    packet_sender::PacketSender,
};
use crate::{error::NaiaServerSocketError, impls::ServerSocket as AsyncServerSocket};

/// Defines the functionality of a Naia Server Socket
pub trait ServerSocketTrait: Debug + Send + Sync {
    /// Gets a MessageReceiver you can use to receive messages from the Server
    /// Socket
    fn get_receiver(&self) -> PacketReceiver;
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
    pub fn listen(
        session_listen_addr: SocketAddr,
        webrtc_listen_addr: SocketAddr,
        public_webrtc_addr: SocketAddr,
    ) -> Self {
        // Set up receiver loop
        let (from_client_sender, from_client_receiver) = channel::unbounded();
        let (sender_sender, sender_receiver) = channel::bounded(1);

        smol::spawn(async move {
            // Create async socket
            let mut async_socket = AsyncServerSocket::listen(
                session_listen_addr,
                webrtc_listen_addr,
                public_webrtc_addr,
            )
            .await;

            sender_sender.send(async_socket.get_sender()).unwrap(); //TODO: handle result..

            loop {
                let out_message = async_socket.receive().await;
                from_client_sender.send(out_message).unwrap(); //TODO: handle result..
            }
        })
        .detach();

        // Set up sender loop
        let (to_client_sender, to_client_receiver) = channel::unbounded();

        smol::spawn(async move {
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
            link_conditioner_config: None,
        };

        socket
    }
}

impl ServerSocketTrait for ServerSocket {
    fn get_receiver(&self) -> PacketReceiver {
        PacketReceiver::new(self.from_client_receiver.clone())
    }

    fn get_sender(&self) -> PacketSender {
        PacketSender::new(self.to_client_sender.clone())
    }

    fn with_link_conditioner(mut self, config: &LinkConditionerConfig) -> Self {
        self.link_conditioner_config = Some(config.clone());
        return self;
    }
}
