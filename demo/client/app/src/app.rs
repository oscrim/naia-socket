use std::time::Duration;

cfg_if! {
    if #[cfg(feature = "mquad")] {
        use miniquad::info;
    } else {
        use log::info;
    }
}

use naia_client_socket::{
    ClientSocket, ClientSocketConfig, ClientSocketTrait, MessageSender, Packet, Timer,
};

use naia_socket_demo_shared::{get_server_address, get_shared_config, PING_MSG, PONG_MSG};

pub struct App {
    client_socket: Box<dyn ClientSocketTrait>,
    message_sender: MessageSender,
    message_count: u8,
    timer: Timer,
}

impl App {
    pub fn new() -> App {
        info!("Naia Client Socket Demo started");

        let client_socket_config =
            ClientSocketConfig::new(get_server_address(), get_shared_config());

        let mut client_socket = ClientSocket::connect(client_socket_config);
        let mut message_sender = client_socket.get_sender();

        message_sender
            .send(Packet::new(PING_MSG.to_string().into_bytes()))
            .unwrap();

        App {
            client_socket,
            message_sender,
            message_count: 0,
            timer: Timer::new(Duration::from_secs(1)),
        }
    }

    pub fn update(&mut self) {
        match self.client_socket.receive() {
            Ok(event) => match event {
                Some(packet) => {
                    let message = String::from_utf8_lossy(packet.payload());
                    info!("Client recv: {}", message);

                    if message.eq(PONG_MSG) {
                        self.message_count += 1;
                    }
                }
                None => {
                    if self.timer.ringing() {
                        self.timer.reset();
                        if self.message_count < 10 {
                            let to_server_message: String = PING_MSG.to_string();
                            info!("Client send: {}", to_server_message,);
                            self.message_sender
                                .send(Packet::new(to_server_message.into_bytes()))
                                .expect("send error");
                        }
                    }
                }
            },
            Err(err) => {
                info!("Client Error: {}", err);
            }
        }
    }
}
