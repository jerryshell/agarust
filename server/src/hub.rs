use crate::*;

use hashbrown::HashMap;
use prost::Message;
use std::time::Duration;
use tokio::{
    sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    time::{interval, Instant},
};
use tracing::{error, info, warn};

const TICK_DURATION: Duration = Duration::from_millis(50);
const SPAWN_SPORE_DURATION: Duration = Duration::from_millis(2000);
const MAX_SPORE_COUNT: usize = 1000;

#[derive(Debug)]
pub struct Hub {
    pub client_map: HashMap<String, ClientRegisterEntry>,
    pub player_map: HashMap<String, Player>,
    pub spore_map: HashMap<String, Spore>,
    pub command_sender: UnboundedSender<Command>,
    pub command_receiver: UnboundedReceiver<Command>,
    pub db_pool: sqlx::Pool<sqlx::Sqlite>,
}

impl Hub {
    pub fn new(db_pool: sqlx::Pool<sqlx::Sqlite>) -> Self {
        let (command_sender, command_receiver) = unbounded_channel::<Command>();
        Self {
            client_map: HashMap::new(),
            player_map: HashMap::new(),
            spore_map: HashMap::new(),
            command_sender,
            command_receiver,
            db_pool,
        }
    }

    pub async fn run(&mut self) {
        for _ in 0..MAX_SPORE_COUNT {
            self.spawn_spore();
        }

        let _ = self.command_sender.send(Command::Tick {
            last_tick: Instant::now(),
            interval: interval(TICK_DURATION),
        });

        let _ = self.command_sender.send(Command::SpawnSpore {
            interval: interval(SPAWN_SPORE_DURATION),
        });

        while let Some(command) = self.command_receiver.recv().await {
            self.handle_command(command);
        }
    }

    fn handle_command(&mut self, command: Command) {
        match command {
            Command::RegisterClient {
                client_register_entry,
            } => {
                info!("RegisterClient: {:?}", client_register_entry);

                let client_agent_command_sender = client_register_entry.command_sender.clone();
                let connection_id = client_register_entry.connection_id.clone();
                self.client_map
                    .insert(connection_id.clone(), client_register_entry);

                let packet = proto_util::hello_packet(connection_id.clone());
                let _ = client_agent_command_sender.send(Command::SendPacket { packet });

                let player = Player::random(connection_id.clone(), connection_id.clone(), 0);

                let packet = proto_util::update_player_packet(&player);
                let _ = client_agent_command_sender.send(Command::SendPacket { packet });

                let mut spore_batch = self.spore_map.values().cloned().collect::<Vec<_>>();
                spore_batch.sort_by_cached_key(|spore| {
                    ((player.x - spore.x).powi(2) + (player.y - spore.y).powi(2)) as i64
                });
                let _ = client_agent_command_sender.send(Command::UpdateSporeBatch { spore_batch });

                self.player_map.insert(connection_id, player);
            }
            Command::UnregisterClient { connection_id } => {
                info!("UnregisterClient: {:?}", connection_id);

                self.client_map.remove(&connection_id);
                self.player_map.remove(&connection_id);

                let packet = proto_util::disconnect_packet(connection_id, "unregister".to_string());
                let _ = self
                    .command_sender
                    .send(Command::BroadcastPacket { packet });
            }
            Command::BroadcastPacket { packet } => {
                let raw_data = packet.encode_to_vec();
                let _ = self
                    .command_sender
                    .send(Command::BroadcastRawData { raw_data });
            }
            Command::BroadcastRawData { raw_data } => {
                self.client_map.values().for_each(|client| {
                    let raw_data = raw_data.clone();
                    let _ = client
                        .command_sender
                        .send(Command::SendRawData { raw_data });
                });
            }
            Command::Tick {
                mut last_tick,
                mut interval,
            } => {
                let delta = last_tick.elapsed();

                self.tick_player(delta);
                last_tick = Instant::now();

                let _ = self.command_sender.send(Command::SyncPlayer);

                let hub_command_sender = self.command_sender.clone();
                tokio::spawn(async move {
                    interval.tick().await;
                    if let Err(error) = hub_command_sender.send(Command::Tick {
                        last_tick,
                        interval,
                    }) {
                        error!("send Command::Tick error: {:?}", error);
                    }
                });
            }
            Command::SyncPlayer => {
                let packet = proto_util::update_player_batch_packet(&self.player_map);
                let _ = self
                    .command_sender
                    .send(Command::BroadcastPacket { packet });
            }
            Command::Chat { connection_id, msg } => {
                let packet = proto_util::chat_packet(connection_id, msg);
                let _ = self
                    .command_sender
                    .send(Command::BroadcastPacket { packet });
            }
            Command::UpdatePlayerDirectionAngle {
                connection_id,
                direction_angle,
            } => {
                if let Some(player) = self.player_map.get_mut(&connection_id) {
                    player.direction_angle = direction_angle;
                }
            }
            Command::ConsumeSpore {
                connection_id,
                spore_id,
            } => {
                if let (Some(player), Some(spore)) = (
                    self.player_map.get_mut(&connection_id),
                    self.spore_map.get_mut(&spore_id),
                ) {
                    let is_close = check_distance_is_close(
                        player.x,
                        player.y,
                        player.radius,
                        spore.x,
                        spore.y,
                        spore.radius,
                    );

                    if !is_close {
                        warn!("consume spore error, distance too far");
                        return;
                    }

                    let spore_mass = radius_to_mass(spore.radius);
                    player.increase_mass(spore_mass);

                    self.spore_map.remove(&spore_id);

                    let packet = proto_util::consume_spore_packet(connection_id, spore_id);
                    let _ = self
                        .command_sender
                        .send(Command::BroadcastPacket { packet });
                }
            }
            Command::ConsumePlayer {
                connection_id,
                victim_connection_id,
            } => {
                match self
                    .player_map
                    .get_many_mut([&connection_id, &victim_connection_id])
                {
                    [Some(player), Some(victim)] => {
                        let is_close = check_distance_is_close(
                            player.x,
                            player.y,
                            player.radius,
                            victim.x,
                            victim.y,
                            victim.radius,
                        );

                        if !is_close {
                            warn!("consume player error, distance too far");
                            return;
                        }

                        let victim_mass = radius_to_mass(victim.radius);
                        player.increase_mass(victim_mass);

                        victim.respawn();
                    }
                    _ => {
                        warn!("not found player or victim in player_map, connection_id: {connection_id:?}, victim_connection_id: {victim_connection_id:?}");
                    }
                }
            }
            Command::SpawnSpore { mut interval } => {
                if self.spore_map.len() < MAX_SPORE_COUNT {
                    let spore = Spore::random();
                    self.spore_map.insert(spore.id.clone(), spore);
                }

                let hub_command_sender = self.command_sender.clone();
                tokio::spawn(async move {
                    interval.tick().await;
                    let _ = hub_command_sender.send(Command::SpawnSpore { interval });
                });
            }
            _ => {
                warn!("unknow command: {:?}", command);
            }
        }
    }

    fn spawn_spore(&mut self) {
        let spore = Spore::random();
        self.spore_map.insert(spore.id.clone(), spore);
    }

    fn tick_player(&mut self, delta: Duration) {
        let delta_secs = delta.as_secs_f64();
        self.player_map.values_mut().for_each(|player| {
            let new_x = player.x + player.speed * player.direction_angle.cos() * delta_secs;
            let new_y = player.y + player.speed * player.direction_angle.sin() * delta_secs;

            player.x = new_x;
            player.y = new_y;

            let drop_mass_probability = player.radius / (MAX_SPORE_COUNT as f64 * 2.0);
            if rand::random::<f64>() < drop_mass_probability {
                if let Some(mass) = player.try_drop_mass() {
                    let mut spore = Spore::random();
                    spore.x = player.x;
                    spore.y = player.y;
                    spore.radius = mass_to_radius(mass);

                    let packet = proto_util::update_spore_pack(&spore);
                    let _ = self
                        .command_sender
                        .send(Command::BroadcastPacket { packet });

                    self.spore_map.insert(spore.id.clone(), spore);
                }
            }
        });
    }
}
