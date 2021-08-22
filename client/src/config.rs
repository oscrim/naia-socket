use std::net::SocketAddr;

use naia_socket_shared::SocketSharedConfig;

/// Config used to initialize a ClientSocket
#[derive(Clone, Debug)]
pub struct ClientSocketConfig {
    /// IP Address of the Server Socket
    pub server_address: SocketAddr,
    /// Config which is shared between Client & Server
    pub shared: SocketSharedConfig,
}

impl ClientSocketConfig {
    /// Create a new Config which will be used to initialize a ClientSocket
    pub fn new(server_address: SocketAddr, shared_config: SocketSharedConfig) -> Self {
        ClientSocketConfig {
            server_address,
            shared: shared_config,
        }
    }
}
