use crate::{hub::Player, proto};
use std::collections::HashMap;

pub fn hello_packet(connection_id: String) -> proto::Packet {
    proto::Packet {
        data: Some(proto::packet::Data::Hello(proto::Hello { connection_id })),
    }
}

pub fn update_player(player: &Player) -> proto::UpdatePlayer {
    proto::UpdatePlayer {
        connection_id: player.connection_id.clone(),
        name: player.name.clone(),
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
