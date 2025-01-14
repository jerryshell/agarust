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
        color: i64,
    },
    Join {
        connection_id: String,
        player_db_id: i64,
        nickname: String,
        color: i64,
    },
    DisconnectClinet,
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
    SyncPlayerBestScore {
        current_score: i64,
    },
    Chat {
        connection_id: String,
        msg: String,
    },
    UpdatePlayerDirectionAngle {
        connection_id: String,
        direction_angle: f64,
    },
    UpdateSporeBatch {
        spore_batch: Vec<spore::Spore>,
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
    LeaderboardRequest,
    LeaderboardResponse {
        entry_list: Vec<LeaderboardEntry>,
    },
}

#[derive(Debug, Clone)]
pub struct LeaderboardEntry {
    pub rank: u64,
    pub player_nickname: String,
    pub score: u64,
}
