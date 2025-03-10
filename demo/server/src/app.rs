use naia_server_socket::{Packet, PacketReceiver, PacketSender, ServerAddrs, Socket};

use naia_socket_demo_shared::{get_server_address, get_shared_config, PING_MSG, PONG_MSG};

pub struct App {
    packet_sender: PacketSender,
    packet_receiver: PacketReceiver,
}

impl App {
    pub fn new() -> Self {
        info!("Naia Server Socket Demo started");

        let server_address = ServerAddrs::new(
            get_server_address(),
            // IP Address to listen on for UDP WebRTC data channels
            "127.0.0.1:14192"
                .parse()
                .expect("could not parse WebRTC data address/port"),
            // The public WebRTC IP address to advertise
            "127.0.0.1:14192"
                .parse()
                .expect("could not parse advertised public WebRTC data address/port"),
        );
        let shared_config = get_shared_config();

        let mut socket = Socket::new(shared_config);
        socket.listen(server_address);

        App {
            packet_sender: socket.get_packet_sender(),
            packet_receiver: socket.get_packet_receiver(),
        }
    }

    pub fn update(&mut self) {
        match self.packet_receiver.receive() {
            Ok(Some(packet)) => {
                let address = packet.address();
                let message = String::from_utf8_lossy(packet.payload());
                info!("Server recv <- {}: {}", address, message);

                if message.eq(PING_MSG) {
                    let to_client_message: String = PONG_MSG.to_string();
                    info!("Server send -> {}: {}", address, to_client_message);
                    self.packet_sender
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
