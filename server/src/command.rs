use crate::*;

use std::net::SocketAddr;
use tokio::{
    sync::mpsc::UnboundedSender,
    time::{Instant, Interval},
};

#[derive(Debug)]
pub enum Command {
    RegisterClient {
        socket_addr: SocketAddr,
        connection_id: String,
        command_sender: UnboundedSender<Command>,
    },
    UnregisterClient {
        connection_id: String,
    },
    Login {
        username: String,
        password: String,
    },
    Register {
        username: String,
        password: String,
        color: i32,
    },
    Join {
        player_db_id: i64,
        connection_id: String,
        color: i32,
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
