use std::fmt::Debug;

use naia_socket_shared::LinkConditionerConfig;

use crate::{PacketReceiverTrait, PacketSender};

cfg_if! {
    if #[cfg(feature = "multithread")] {
        pub trait ClientSocketBaseTrait: Debug + Send + Sync {}
        impl < T > ClientSocketBaseTrait for T where T: Debug + Send + Sync {}
    } else {
        pub trait ClientSocketBaseTrait: Debug {}
        impl < T > ClientSocketBaseTrait for T where T: Debug {}
    }
}
/// Defines the functionality of a Naia Client Socket
pub trait ClientSocketTrait: ClientSocketBaseTrait {
    /// Gets a PacketReceiver you can use to receive messages from the Client
    /// Socket
    fn get_receiver(&self) -> Box<dyn PacketReceiverTrait>;
    /// Gets a PacketSender you can use to send messages through the Client
    /// Socket
    fn get_sender(&self) -> PacketSender;
    /// Wraps the current socket in a LinkConditioner
    fn with_link_conditioner(self, config: &LinkConditionerConfig) -> Self;
}
