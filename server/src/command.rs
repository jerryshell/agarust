use std::net::SocketAddr;
use tokio::sync::mpsc::UnboundedSender;

use crate::proto;

#[derive(Debug, Clone)]
pub enum Command {
    Hello,
    RegisterClient {
        client_register_entry: ClientRegisterEntry,
    },
    UnregisterClient {
        connection_id: String,
    },
    Broadcast {
        packet: proto::Packet,
    },
    SendPacket {
        packet: proto::Packet,
    },
    SendRawData {
        raw_data: Vec<u8>,
    },
    TickPlayer,
    UpdatePlayerDirectionAngle {
        connection_id: String,
        direction_angle: f64,
    },
}

#[derive(Debug, Clone)]
pub struct ClientRegisterEntry {
    pub socket_addr: SocketAddr,
    pub connection_id: String,
    pub command_sender: UnboundedSender<Command>,
}
