use std::error::Error;

use super::shared::{naia_create_u8_array, naia_send};
use crate::Packet;

/// Handles receiving messages from the Server through a given Client Socket
#[derive(Clone, Debug)]
pub struct PacketReceiver {}

impl PacketReceiver {
    /// Create a new PacketReceiver, if supplied with the RtcDataChannel and a
    /// reference to a list of dropped messages
    pub fn new() -> Self {
        PacketReceiver {}
    }

    /// Send a Packet to the Server
    pub fn send(&mut self, packet: Packet) -> Result<(), Box<dyn Error + Send + Sync>> {
        unsafe {
            let payload: &[u8] = packet.payload();
            let ptr = payload.as_ptr();
            let len = payload.len();
            let js_obj = naia_create_u8_array(ptr as _, len as _);
            naia_send(js_obj);
        }

        Ok(())
    }
}
