use std::net::SocketAddr;

use naia_socket_shared::{LinkConditionerConfig, SocketConfig};

pub const PING_MSG: &str = "ping";
pub const PONG_MSG: &str = "pong";

pub fn get_server_address() -> SocketAddr {
    return "127.0.0.1:14191"
        .parse()
        .expect("could not parse socket address from string");
}

pub fn get_shared_config() -> SocketConfig {
    //let link_condition = None;
    let link_condition = Some(LinkConditionerConfig::average_condition());
    //    let link_condition = Some(LinkConditionerConfig {
    //        incoming_latency: 500,
    //        incoming_jitter: 1,
    //        incoming_loss: 0.0,
    //        incoming_corruption: 0.0
    //    });

    return SocketConfig::new(link_condition, None);
}
