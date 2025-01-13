use crate::*;

use hashbrown::HashMap;

pub fn hello_packet(connection_id: String) -> proto::Packet {
    proto::Packet {
        data: Some(proto::packet::Data::Hello(proto::Hello { connection_id })),
    }
}

pub fn login_ok_packet() -> proto::Packet {
    proto::Packet {
        data: Some(proto::packet::Data::LoginOk(proto::LoginOk {})),
    }
}

pub fn login_err_packet(reason: String) -> proto::Packet {
    proto::Packet {
        data: Some(proto::packet::Data::LoginErr(proto::LoginErr { reason })),
    }
}

pub fn register_ok_packet() -> proto::Packet {
    proto::Packet {
        data: Some(proto::packet::Data::RegisterOk(proto::RegisterOk {})),
    }
}

pub fn register_err_packet(reason: String) -> proto::Packet {
    proto::Packet {
        data: Some(proto::packet::Data::RegisterErr(proto::RegisterErr {
            reason,
        })),
    }
}

pub fn chat_packet(connection_id: String, msg: String) -> proto::Packet {
    proto::Packet {
        data: Some(proto::packet::Data::Chat(proto::Chat {
            connection_id,
            msg,
        })),
    }
}

pub fn update_player(player: &Player) -> proto::UpdatePlayer {
    proto::UpdatePlayer {
        connection_id: player.connection_id.clone(),
        nickname: player.nickname.clone(),
        x: player.x,
        y: player.y,
        radius: player.radius,
        direction_angle: player.direction_angle,
        speed: player.speed,
        color: player.color,
    }
}

pub fn update_player_packet(player: &Player) -> proto::Packet {
    proto::Packet {
        data: Some(proto::packet::Data::UpdatePlayer(update_player(player))),
    }
}

pub fn update_player_batch_packet(player_map: &HashMap<String, Player>) -> proto::Packet {
    let update_player_batch = player_map
        .values()
        .map(update_player)
        .collect::<Vec<proto::UpdatePlayer>>();
    proto::Packet {
        data: Some(proto::packet::Data::UpdatePlayerBatch(
            proto::UpdatePlayerBatch {
                update_player_batch,
            },
        )),
    }
}

pub fn update_spore(spore: &Spore) -> proto::UpdateSpore {
    proto::UpdateSpore {
        id: spore.id.clone(),
        x: spore.x,
        y: spore.y,
        radius: spore.radius,
    }
}

pub fn update_spore_pack(spore: &Spore) -> proto::Packet {
    proto::Packet {
        data: Some(proto::packet::Data::UpdateSpore(update_spore(spore))),
    }
}

pub fn update_spore_batch_packet(spore_list: &[Spore]) -> proto::Packet {
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

pub fn consume_spore_packet(connection_id: String, spore_id: String) -> proto::Packet {
    proto::Packet {
        data: Some(proto::packet::Data::ConsumeSpore(proto::ConsumeSpore {
            connection_id,
            spore_id,
        })),
    }
}

pub fn disconnect_packet(connection_id: String, reason: String) -> proto::Packet {
    proto::Packet {
        data: Some(proto::packet::Data::Disconnect(proto::Disconnect {
            connection_id,
            reason,
        })),
    }
}
