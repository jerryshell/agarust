use crate::{hub::Spore, proto};
use std::net::SocketAddr;
use tokio::{sync::mpsc::UnboundedSender, time::Instant};

#[derive(Debug, Clone)]
pub enum Command {
    Hello,
    RegisterClient {
        client_register_entry: ClientRegisterEntry,
    },
    UnregisterClient {
        connection_id: String,
    },
    BroadcastPacket {
        packet: proto::Packet,
    },
    BroadcastRawData {
        raw_data: Vec<u8>,
    },
    SendPacket {
        packet: proto::Packet,
    },
    SendRawData {
        raw_data: Vec<u8>,
    },
    Tick {
        last_tick: Instant,
    },
    UpdatePlayerDirectionAngle {
        connection_id: String,
        direction_angle: f64,
    },
    UpdateSporeBatch {
        spore_batch: Vec<Spore>,
    },
    ConsumeSpore {
        connection_id: String,
        spore_id: String,
    },
    ConsumePlayer {
        connection_id: String,
        player_id: String,
    },
}

#[derive(Debug, Clone)]
pub struct ClientRegisterEntry {
    pub socket_addr: SocketAddr,
    pub connection_id: String,
    pub command_sender: UnboundedSender<Command>,
}
