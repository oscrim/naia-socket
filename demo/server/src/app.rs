use naia_server_socket::{
    Packet, PacketReceiverTrait, PacketSender, ServerSocket, ServerSocketConfig, ServerSocketTrait,
};

use naia_socket_demo_shared::{get_server_address, get_shared_config, PING_MSG, PONG_MSG};

pub struct App {
    sender: PacketSender,
    receiver: Box<dyn PacketReceiverTrait>,
}

impl App {
    pub fn new() -> Self {
        info!("Naia Server Socket Demo started");

        let server_socket_config = ServerSocketConfig::new(
            get_server_address(),
            // IP Address to listen on for UDP WebRTC data channels
            "127.0.0.1:14192"
                .parse()
                .expect("could not parse WebRTC data address/port"),
            // The public WebRTC IP address to advertise
            "127.0.0.1:14192"
                .parse()
                .expect("could not parse advertised public WebRTC data address/port"),
            get_shared_config(),
        );

        let server_socket = ServerSocket::listen(server_socket_config);

        App {
            sender: server_socket.get_sender(),
            receiver: server_socket.get_receiver(),
        }
    }

    pub fn update(&mut self) {
        match self.receiver.receive() {
            Ok(Some(packet)) => {
                let address = packet.address();
                let message = String::from_utf8_lossy(packet.payload());
                info!("Server recv <- {}: {}", address, message);

                if message.eq(PING_MSG) {
                    let to_client_message: String = PONG_MSG.to_string();
                    info!("Server send -> {}: {}", address, to_client_message);
                    self.sender
                        .send(Packet::new(address, to_client_message.into_bytes()));
                }
            }
            Ok(None) => {}
            Err(error) => {
                info!("Server Error: {}", error);
            }
        }
    }
}
