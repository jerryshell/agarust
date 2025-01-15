use crate::*;

use client_agent::ClientAgent;
use tokio::time::{Instant, Interval};

#[derive(Debug)]
pub enum Command {
    RegisterClientAgent {
        client_agent: ClientAgent,
    },
    UnregisterClientAgent {
        connection_id: Arc<str>,
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
        connection_id: Arc<str>,
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
        connection_id: Arc<str>,
        msg: String,
    },
    UpdatePlayerDirectionAngle {
        connection_id: Arc<str>,
        direction_angle: f64,
    },
    UpdateSporeBatch {
        spore_batch: Vec<spore::Spore>,
    },
    ConsumeSpore {
        connection_id: Arc<str>,
        spore_id: String,
    },
    ConsumePlayer {
        connection_id: Arc<str>,
        victim_connection_id: Arc<str>,
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
