use std::{
    io::ErrorKind,
    net::{SocketAddr, UdpSocket},
    sync::{Arc, Mutex},
};

use crate::{error::NaiaClientSocketError, packet::Packet, packet_receiver::PacketReceiverTrait};

/// Handles receiving messages from the Server through a given Client Socket
#[derive(Clone)]
pub struct PacketReceiverImpl {
    address: SocketAddr,
    socket: Arc<Mutex<UdpSocket>>,
    receive_buffer: Vec<u8>,
}

impl PacketReceiverImpl {
    /// Create a new PacketReceiver, if supplied with the Server's address & a
    /// reference back to the parent Socket
    pub fn new(address: SocketAddr, socket: Arc<Mutex<UdpSocket>>) -> Self {
        PacketReceiverImpl {
            address,
            socket,
            receive_buffer: vec![0; 1472],
        }
    }
}

impl PacketReceiverTrait for PacketReceiverImpl {
    fn receive(&mut self) -> Result<Option<Packet>, NaiaClientSocketError> {
        let buffer: &mut [u8] = self.receive_buffer.as_mut();
        match self
            .socket
            .as_ref()
            .lock()
            .unwrap()
            .recv_from(buffer)
            .map(move |(recv_len, address)| (&buffer[..recv_len], address))
        {
            Ok((payload, address)) => {
                if address == self.address {
                    return Ok(Some(Packet::new(payload.to_vec())));
                } else {
                    return Err(NaiaClientSocketError::Message(
                        "Unknown sender.".to_string(),
                    ));
                }
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                //just didn't receive anything this time
                return Ok(None);
            }
            Err(e) => {
                return Err(NaiaClientSocketError::Wrapped(Box::new(e)));
            }
        }
    }
}
