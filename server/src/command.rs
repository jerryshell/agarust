use crate::*;

#[derive(Debug)]
pub enum Command {
    RegisterClientAgent {
        socket_addr: SocketAddr,
        connection_id: Arc<str>,
        client_agent_command_sender: UnboundedSender<command::Command>,
    },
    UnregisterClientAgent {
        connection_id: Arc<str>,
    },
    Login {
        username: Arc<str>,
        password: Arc<str>,
    },
    Register {
        username: Arc<str>,
        password: Arc<str>,
        color: i64,
    },
    Join {
        connection_id: Arc<str>,
        player_db_id: i64,
        nickname: Arc<str>,
        color: i64,
    },
    DisconnectClinet,
    SendPacket {
        packet: proto::Packet,
    },
    SendRawData {
        raw_data: Vec<u8>,
    },
    SyncPlayerBestScore {
        current_score: i64,
    },
    Chat {
        connection_id: Arc<str>,
        msg: Arc<str>,
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
        spore_id: Arc<str>,
    },
    ConsumePlayer {
        connection_id: Arc<str>,
        victim_connection_id: Arc<str>,
    },
    Rush {
        connection_id: Arc<str>,
    },
    LeaderboardRequest,
    LeaderboardResponse {
        entry_list: Vec<LeaderboardEntry>,
    },
}

#[derive(Debug, Clone)]
pub struct LeaderboardEntry {
    pub rank: u64,
    pub player_nickname: Arc<str>,
    pub score: u64,
}
