use std::net::SocketAddr;

mod shared_config;
pub use shared_config::get_shared_config;

pub fn get_server_address() -> SocketAddr {
    return "127.0.0.1:14191"
        .parse()
        .expect("could not parse socket address from string");
}

pub const PING_MSG: &str = "ping";
pub const PONG_MSG: &str = "pong";
