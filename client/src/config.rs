use std::net::SocketAddr;

use naia_socket_shared::SocketSharedConfig;

/// Config used to initialize a ClientSocket
#[derive(Clone, Debug)]
pub struct ClientSocketConfig {
    /// Config which is shared between Client & Server
    pub shared: SocketSharedConfig,
}

impl ClientSocketConfig {
    /// Create a new Config which will be used to initialize a ClientSocket
    pub fn new(shared: SocketSharedConfig) -> Self {
        ClientSocketConfig { shared }
    }
}
