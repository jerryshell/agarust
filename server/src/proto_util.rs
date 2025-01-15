use crate::*;

pub fn hello_packet(connection_id: Arc<str>) -> proto::Packet {
    proto::Packet {
        data: Some(proto::packet::Data::Hello(proto::Hello {
            connection_id: connection_id.to_string(),
        })),
    }
}

pub fn login_ok_packet() -> proto::Packet {
    proto::Packet {
        data: Some(proto::packet::Data::LoginOk(proto::LoginOk {})),
    }
}

pub fn login_err_packet(reason: Arc<str>) -> proto::Packet {
    proto::Packet {
        data: Some(proto::packet::Data::LoginErr(proto::LoginErr {
            reason: reason.to_string(),
        })),
    }
}

pub fn register_ok_packet() -> proto::Packet {
    proto::Packet {
        data: Some(proto::packet::Data::RegisterOk(proto::RegisterOk {})),
    }
}

pub fn register_err_packet(reason: Arc<str>) -> proto::Packet {
    proto::Packet {
        data: Some(proto::packet::Data::RegisterErr(proto::RegisterErr {
            reason: reason.to_string(),
        })),
    }
}

pub fn chat_packet(connection_id: Arc<str>, msg: Arc<str>) -> proto::Packet {
    proto::Packet {
        data: Some(proto::packet::Data::Chat(proto::Chat {
            connection_id: connection_id.to_string(),
            msg: msg.to_string(),
        })),
    }
}

pub fn update_player(player: &player::Player) -> proto::UpdatePlayer {
    proto::UpdatePlayer {
        connection_id: player.connection_id.to_string(),
        nickname: player.nickname.to_string(),
        x: player.x,
        y: player.y,
        radius: player.radius,
        direction_angle: player.direction_angle,
        speed: player.speed,
        color: player.color,
        is_rushing: player.rush_instant.is_some(),
    }
}

pub fn update_player_packet(player: &player::Player) -> proto::Packet {
    proto::Packet {
        data: Some(proto::packet::Data::UpdatePlayer(update_player(player))),
    }
}

pub fn update_player_batch_packet(player_list: &[&player::Player]) -> proto::Packet {
    let update_player_batch = player_list
        .iter()
        .map(|player| update_player(player))
        .collect::<Vec<proto::UpdatePlayer>>();
    proto::Packet {
        data: Some(proto::packet::Data::UpdatePlayerBatch(
            proto::UpdatePlayerBatch {
                update_player_batch,
            },
        )),
    }
}

pub fn update_spore(spore: &spore::Spore) -> proto::UpdateSpore {
    proto::UpdateSpore {
        id: spore.id.to_string(),
        x: spore.x,
        y: spore.y,
        radius: spore.radius,
    }
}

pub fn update_spore_pack(spore: &spore::Spore) -> proto::Packet {
    proto::Packet {
        data: Some(proto::packet::Data::UpdateSpore(update_spore(spore))),
    }
}

pub fn update_spore_batch_packet(spore_list: &[spore::Spore]) -> proto::Packet {
    let update_spore_batch = spore_list
        .iter()
        .map(update_spore)
        .collect::<Vec<proto::UpdateSpore>>();
    proto::Packet {
        data: Some(proto::packet::Data::UpdateSporeBatch(
            proto::UpdateSporeBatch { update_spore_batch },
        )),
    }
}

pub fn consume_spore_packet(connection_id: Arc<str>, spore_id: Arc<str>) -> proto::Packet {
    proto::Packet {
        data: Some(proto::packet::Data::ConsumeSpore(proto::ConsumeSpore {
            connection_id: connection_id.to_string(),
            spore_id: spore_id.to_string(),
        })),
    }
}

pub fn disconnect_packet(connection_id: Arc<str>, reason: Arc<str>) -> proto::Packet {
    proto::Packet {
        data: Some(proto::packet::Data::Disconnect(proto::Disconnect {
            connection_id: connection_id.to_string(),
            reason: reason.to_string(),
        })),
    }
}

pub fn leaderboard_response(leaderboard_entry_list: &[command::LeaderboardEntry]) -> proto::Packet {
    let leaderboard_entry_list = leaderboard_entry_list
        .iter()
        .map(|entry| proto::LeaderboardEntry {
            rank: entry.rank,
            player_nickname: entry.player_nickname.to_string(),
            score: entry.score,
        })
        .collect::<Vec<_>>();
    proto::Packet {
        data: Some(proto::packet::Data::LeaderboardResponse(
            proto::LeaderboardResponse {
                leaderboard_entry_list,
            },
        )),
    }
}
