use std::fmt::Debug;

use crate::{PacketReceiver, PacketSender};

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
    fn get_receiver(&self) -> Box<dyn PacketReceiver>;
    /// Gets a PacketSender you can use to send messages through the Client
    /// Socket
    fn get_sender(&self) -> PacketSender;
}
