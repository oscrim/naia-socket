//! # Naia Socket Shared
//! Common data types shared between Naia Server Socket & Naia Client Socket

#![deny(
    missing_docs,
    missing_debug_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]

#[macro_use]
extern crate cfg_if;

/// Logic shared between client & server sockets related to simulating network
/// conditions
pub mod link_condition_logic;

mod find_my_ip_address;
mod impls;
mod link_conditioner_config;
mod packet_reader;
mod reference;
mod shared_config;
mod time_queue;

pub use find_my_ip_address::find_my_ip_address;
pub use impls::{Instant, Random, Timer, Timestamp};
pub use link_conditioner_config::LinkConditionerConfig;
pub use packet_reader::PacketReader;
pub use reference::Ref;
pub use shared_config::SocketConfig;
pub use time_queue::TimeQueue;

cfg_if! {
    if #[cfg(all(target_arch = "wasm32", feature = "wbindgen", feature = "mquad"))]
    {
        // Use both protocols...
        compile_error!("wasm target for 'naia_socket_shared' crate requires either the 'wbindgen' OR 'mquad' feature to be enabled, you must pick one.");
    }
    else if #[cfg(all(target_arch = "wasm32", not(feature = "wbindgen"), not(feature = "mquad")))]
    {
        // Use no protocols...
        compile_error!("wasm target for 'naia_socket_shared' crate requires either the 'wbindgen' or 'mquad' feature to be enabled, you must pick one.");
    }
}
