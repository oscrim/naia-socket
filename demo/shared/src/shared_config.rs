use naia_socket_shared::{LinkConditionerConfig, SocketSharedConfig};

pub fn get_shared_config() -> SocketSharedConfig {
    //let link_condition = None;
    let link_condition = Some(LinkConditionerConfig::average_condition());
    //    let link_condition = Some(LinkConditionerConfig {
    //        incoming_latency: 500,
    //        incoming_jitter: 1,
    //        incoming_loss: 0.0,
    //        incoming_corruption: 0.0
    //    });

    return SocketSharedConfig::new(link_condition, None);
}
