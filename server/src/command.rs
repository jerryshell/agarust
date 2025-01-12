use crate::*;

use std::net::SocketAddr;
use tokio::{
    sync::mpsc::UnboundedSender,
    time::{Instant, Interval},
};

#[derive(Debug)]
pub enum Command {
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
        interval: Interval,
    },
    SyncPlayer,
    Chat {
        connection_id: String,
        msg: String,
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
        victim_connection_id: String,
    },
    SpawnSpore {
        interval: Interval,
    },
}

#[derive(Debug, Clone)]
pub struct ClientRegisterEntry {
    pub socket_addr: SocketAddr,
    pub connection_id: String,
    pub command_sender: UnboundedSender<Command>,
}
