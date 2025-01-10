use crate::{
    hub::{Player, Spore},
    proto,
};
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
