//! # Naia Client Socket
//! A Socket abstraction over either a UDP socket on native Linux, or a
//! unreliable WebRTC datachannel on the browser

#![deny(
    missing_docs,
    missing_debug_implementations,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]

#[macro_use]
extern crate cfg_if;

cfg_if! {
    if #[cfg(all(target_arch = "wasm32", feature = "wbindgen"))] {
        #[macro_use]
        extern crate serde_derive;
    }
    else {
    }
}

mod error;
mod impls;
mod packet;
mod packet_receiver;

pub use naia_socket_shared::Timer;

pub use error::NaiaClientSocketError;
pub use impls::{PacketSender, Socket};
pub use packet::Packet;
pub use packet_receiver::PacketReceiver;
