#[macro_use]
extern crate log;

use log::LevelFilter;
use simple_logger::SimpleLogger;

use naia_server_socket::{Packet, ServerSocket, ServerSocketTrait};

const PING_MSG: &str = "ping";
const PONG_MSG: &str = "pong";

fn main() {
    // IP Address to listen on for the signaling portion of WebRTC
    let session_listen_addr = "127.0.0.1:14191"
        .parse()
        .expect("could not parse HTTP address/port");

    // IP Address to listen on for UDP WebRTC data channels
    let webrtc_listen_addr = "127.0.0.1:14192"
        .parse()
        .expect("could not parse WebRTC data address/port");

    // The public WebRTC IP address to advertise
    let public_webrtc_addr = "127.0.0.1:14192"
        .parse()
        .expect("could not parse advertised public WebRTC data address/port");

    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .expect("A logger was already initialized");

    info!("Naia Server Socket Demo started");

    let server_socket =
        ServerSocket::listen(session_listen_addr, webrtc_listen_addr, public_webrtc_addr);
    //.with_link_conditioner(&LinkConditionerConfig::good_condition());

    let sender = server_socket.get_sender();
    let mut receiver = server_socket.get_receiver();

    loop {
        match receiver.receive() {
            Ok(Some(packet)) => {
                let address = packet.address();
                let message = String::from_utf8_lossy(packet.payload());
                info!("Server recv <- {}: {}", address, message);

                if message.eq(PING_MSG) {
                    let to_client_message: String = PONG_MSG.to_string();
                    info!("Server send -> {}: {}", address, to_client_message);
                    sender.send(Packet::new(address, to_client_message.into_bytes()));
                }
            }
            Ok(None) => {}
            Err(error) => {
                info!("Server Error: {}", error);
            }
        }
    }
}
