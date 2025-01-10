use std::net::SocketAddr;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug, Clone)]
pub enum Command {
    Hello,
    RegisterClient {
        client_register_entry: ClientRegisterEntry,
    },
    UnregisterClient {
        connection_id: String,
    },
}

#[derive(Debug, Clone)]
pub struct ClientRegisterEntry {
    pub socket_addr: SocketAddr,
    pub connection_id: String,
    pub command_sender: UnboundedSender<Command>,
}
