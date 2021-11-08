use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use web_sys::RtcDataChannel;

use crate::{error::NaiaClientSocketError, packet::Packet, packet_receiver::PacketReceiverTrait};

/// Handles receiving messages from the Server through a given Client Socket
#[derive(Clone)]
pub struct PacketReceiverImpl {
    data_channel: RtcDataChannel,
    dropped_outgoing_messages: Rc<RefCell<VecDeque<Packet>>>,
    message_queue: Rc<RefCell<VecDeque<Packet>>>,
}

impl PacketReceiverImpl {
    /// Create a new PacketReceiver, if supplied with the RtcDataChannel and a
    /// reference to a list of dropped messages
    pub fn new(
        data_channel: RtcDataChannel,
        dropped_outgoing_messages: Rc<RefCell<VecDeque<Packet>>>,
        message_queue: Rc<RefCell<VecDeque<Packet>>>,
    ) -> Self {
        PacketReceiverImpl {
            data_channel,
            dropped_outgoing_messages,
            message_queue,
        }
    }
}

impl PacketReceiverTrait for PacketReceiverImpl {
    fn receive(&mut self) -> Result<Option<Packet>, NaiaClientSocketError> {
        match self.message_queue.borrow_mut().pop_front() {
            Some(packet) => {
                return Ok(Some(packet));
            }
            None => {
                return Ok(None);
            }
        }
    }
}

unsafe impl Send for PacketReceiverImpl {}
unsafe impl Sync for PacketReceiverImpl {}
