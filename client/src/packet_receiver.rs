use std::fmt::Debug;

use naia_socket_shared::{link_condition_logic, LinkConditionerConfig, TimeQueue};

use super::{error::NaiaClientSocketError, packet::Packet};

/// Used to receive packets from the Client Socket
#[derive(Clone, Debug)]
pub struct PacketReceiver {
    inner: Box<dyn PacketReceiverTrait>,
}

impl PacketReceiver {
    /// Create a new PacketReceiver
    pub fn new(inner: Box<dyn PacketReceiverTrait>) -> Self {
        PacketReceiver { inner }
    }

    /// Receives a packet from the Client Socket
    pub fn receive(&mut self) -> Result<Option<Packet>, NaiaClientSocketError> {
        return self.inner.receive();
    }
}

cfg_if! {
    if #[cfg(feature = "multithread")]
    {
        /// Used to receive packets from the Client Socket
        pub trait PacketReceiverTrait: PacketReceiverClone + Send + Sync + Debug {
            /// Receives a packet from the Client Socket
            fn receive(&mut self) -> Result<Option<Packet>, NaiaClientSocketError>;
        }
    }
    else
    {
        /// Used to receive packets from the Client Socket
        pub trait PacketReceiverTrait: PacketReceiverClone + Debug {
            /// Receives a packet from the Client Socket
            fn receive(&mut self) -> Result<Option<Packet>, NaiaClientSocketError>;
        }
    }
}

/// Used to receive packets from the Client Socket
#[derive(Clone, Debug)]
pub struct ConditionedPacketReceiver {
    inner_receiver: Box<dyn PacketReceiverTrait>,
    link_conditioner_config: LinkConditionerConfig,
    time_queue: TimeQueue<Packet>,
}

impl ConditionedPacketReceiver {
    /// Creates a new ConditionedPacketReceiver
    pub fn new(
        inner_receiver: Box<dyn PacketReceiverTrait>,
        link_conditioner_config: &LinkConditionerConfig,
    ) -> Self {
        ConditionedPacketReceiver {
            inner_receiver,
            link_conditioner_config: link_conditioner_config.clone(),
            time_queue: TimeQueue::new(),
        }
    }

    fn process_packet(&mut self, packet: Packet) {
        link_condition_logic::process_packet(
            &self.link_conditioner_config,
            &mut self.time_queue,
            packet,
        );
    }

    fn has_packet(&self) -> bool {
        self.time_queue.has_item()
    }

    fn get_packet(&mut self) -> Packet {
        self.time_queue.pop_item().unwrap()
    }
}

impl PacketReceiverTrait for ConditionedPacketReceiver {
    fn receive(&mut self) -> Result<Option<Packet>, NaiaClientSocketError> {
        loop {
            match self.inner_receiver.receive() {
                Ok(option) => match option {
                    None => {
                        break; //TODO: Handle error here
                    }
                    Some(packet) => {
                        self.process_packet(packet);
                    }
                },
                Err(_) => {
                    break; //TODO: Handle error here
                }
            }
        }

        if self.has_packet() {
            return Ok(Some(self.get_packet()));
        } else {
            return Ok(None);
        }
    }
}

/// Used to clone Box<dyn PacketReceiverTrait>
pub trait PacketReceiverClone {
    /// Clone the boxed PacketReceiver
    fn clone_box(&self) -> Box<dyn PacketReceiverTrait>;
}

impl<T: 'static + PacketReceiverTrait + Clone> PacketReceiverClone for T {
    fn clone_box(&self) -> Box<dyn PacketReceiverTrait> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn PacketReceiverTrait> {
    fn clone(&self) -> Box<dyn PacketReceiverTrait> {
        PacketReceiverClone::clone_box(self.as_ref())
    }
}
