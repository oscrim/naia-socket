use std::{net::SocketAddr, time::Duration};

cfg_if! {
    if #[cfg(feature = "mquad")] {
        use miniquad::info;
    } else {
        use log::info;
    }
}

use naia_client_socket::{ClientSocket, ClientSocketTrait, MessageSender, Packet, Timer};

const PING_MSG: &str = "ping";
const PONG_MSG: &str = "pong";

pub struct App {
    client_socket: Box<dyn ClientSocketTrait>,
    message_sender: MessageSender,
    message_count: u8,
    timer: Timer,
}

impl App {
    pub fn new() -> App {
        info!("Naia Client Socket Demo started");

        // Put your Server's IP Address here!, can't easily find this automatically from the browser
        // Note: a 127.0.0.1 IP address will not function correctly on Firefox
        let server_ip_address: SocketAddr = "127.0.0.1:14191"
            .parse()
            .expect("couldn't parse input IP address");

        let mut client_socket = ClientSocket::connect(server_ip_address);
        //.with_link_conditioner(&LinkConditionerConfig::good_condition());
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
        loop {
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
}
