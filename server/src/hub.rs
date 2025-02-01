use crate::*;

use bytes::Bytes;
use hashbrown::HashMap;
use nanoid::nanoid;
use prost::Message;
use std::time::Duration;
use tokio::{
    select,
    sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    time::{interval, Instant},
};
use tracing::{error, info, warn};

const TICK_DURATION: Duration = Duration::from_millis(50);
const SPAWN_SPORE_DURATION: Duration = Duration::from_millis(2000);
const MAX_SPORE_COUNT: usize = 1000;

#[derive(Debug)]
pub struct Client {
    pub socket_addr: SocketAddr,
    pub connection_id: Arc<str>,
    pub client_agent_command_sender: UnboundedSender<command::Command>,
    pub player: Option<player::Player>,
}

#[derive(Debug)]
pub struct Hub {
    pub client_map: HashMap<Arc<str>, Client>,
    pub spore_map: HashMap<Arc<str>, spore::Spore>,
    pub command_sender: UnboundedSender<command::Command>,
    pub command_receiver: UnboundedReceiver<command::Command>,
    pub db: db::Db,
}

impl Hub {
    pub fn new(db: db::Db) -> Self {
        let (command_sender, command_receiver) = unbounded_channel::<command::Command>();
        Self {
            client_map: HashMap::new(),
            spore_map: HashMap::new(),
            command_sender,
            command_receiver,
            db,
        }
    }

    pub async fn run(mut self) {
        for _ in 0..MAX_SPORE_COUNT {
            self.spawn_spore();
        }

        let mut tick_interval = interval(TICK_DURATION);
        let mut last_tick = Instant::now();

        let mut spawn_spore_interval = interval(SPAWN_SPORE_DURATION);

        loop {
            select! {
                _ = tick_interval.tick() => {
                    let delta = last_tick.elapsed();
                    self.tick_player(delta);
                    last_tick = Instant::now();
                }
                _ = spawn_spore_interval.tick() => {
                    if self.spore_map.len() < MAX_SPORE_COUNT {
                        self.spawn_spore();
                    }
                }
                Some(command) = self.command_receiver.recv() => {
                    self.handle_command(command).await;
                }
            }
        }
    }

    async fn handle_command(&mut self, command: command::Command) {
        match command {
            command::Command::RegisterClientAgent {
                socket_addr,
                client_agent_command_sender,
                response_sender,
            } => {
                info!("RegisterClientAgent: {:?}", socket_addr);

                let connection_id: Arc<str> = nanoid!().into();
                info!("connection_id: {:?}", connection_id);

                let client = Client {
                    socket_addr,
                    connection_id: connection_id.clone(),
                    client_agent_command_sender: client_agent_command_sender.clone(),
                    player: None,
                };
                self.client_map.insert(connection_id.clone(), client);

                let _ = response_sender.send(connection_id);
            }
            command::Command::UnregisterClientAgent { connection_id } => {
                info!("UnregisterClient: {:?}", connection_id);

                self.client_map.remove(&connection_id);

                let packet = proto_util::disconnect_packet(connection_id, "unregister".into());
                self.broadcast_packet(&packet);
            }
            command::Command::Join {
                connection_id,
                player_db_id,
                nickname,
                color,
            } => {
                info!(
                    "PlayerJoin: {:?} {:?} {:?} {:#x?}",
                    connection_id, player_db_id, nickname, color
                );

                self.client_map
                    .values()
                    .flat_map(|client| client.player.as_ref())
                    .for_each(|online_player| {
                        if online_player.db_id == player_db_id {
                            if let Some(online_client) =
                                self.client_map.get(&online_player.connection_id)
                            {
                                let _ = online_client
                                    .client_agent_command_sender
                                    .send(command::Command::DisconnectClinet);
                            }
                        }
                    });

                let client = match self.client_map.get_mut(&connection_id) {
                    Some(client) => client,
                    None => {
                        error!("client not found: {:?}", connection_id);
                        return;
                    }
                };

                let player = player::Player::random(player_db_id, connection_id, nickname, color);

                let player_x = player.x;
                let player_y = player.y;

                client.player = Some(player);

                let mut spore_batch = self.spore_map.values().cloned().collect::<Vec<_>>();
                spore_batch.sort_by_cached_key(|spore| {
                    ((player_x - spore.x).powi(2) + (player_y - spore.y).powi(2)) as i64
                });

                let _ = client
                    .client_agent_command_sender
                    .send(command::Command::UpdateSporeBatch { spore_batch });
            }
            command::Command::Chat { connection_id, msg } => {
                let packet = proto_util::chat_packet(connection_id, msg);
                self.broadcast_packet(&packet);
            }
            command::Command::UpdatePlayerDirectionAngle {
                connection_id,
                direction_angle,
            } => {
                if let Some(client) = self.client_map.get_mut(&connection_id) {
                    if let Some(player) = client.player.as_mut() {
                        player.direction_angle = direction_angle;
                    }
                }
            }
            command::Command::ConsumeSpore {
                connection_id,
                spore_id,
            } => {
                if let (Some(client), Some(spore)) = (
                    self.client_map.get_mut(&connection_id),
                    self.spore_map.get_mut(&spore_id),
                ) {
                    if let Some(player) = client.player.as_mut() {
                        let is_close = util::check_distance_is_close(
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

                        let spore_mass = util::radius_to_mass(spore.radius);
                        player.increase_mass(spore_mass);

                        self.spore_map.remove(&spore_id);

                        let current_score = util::radius_to_mass(player.radius) as i64;

                        let client_agent_command_sender =
                            client.client_agent_command_sender.clone();

                        self.broadcast_packet(&proto_util::consume_spore_packet(
                            connection_id,
                            spore_id,
                        ));

                        let _ = client_agent_command_sender
                            .send(command::Command::SyncPlayerBestScore { current_score });
                    }
                }
            }
            command::Command::ConsumePlayer {
                connection_id,
                victim_connection_id,
            } => {
                if let [Some(player_client), Some(victim_client)] = self
                    .client_map
                    .get_many_mut([&connection_id, &victim_connection_id])
                {
                    if let (Some(player), Some(victim)) =
                        (&mut player_client.player, &mut victim_client.player)
                    {
                        let is_close = util::check_distance_is_close(
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

                        let victim_mass = util::radius_to_mass(victim.radius);
                        player.increase_mass(victim_mass);

                        victim.respawn();
                    }
                }
            }
            command::Command::Rush { connection_id } => {
                if let Some(client) = self.client_map.get_mut(&connection_id) {
                    if let Some(player) = client.player.as_mut() {
                        if player.radius < 20.0 {
                            return;
                        }
                        if player.rush_instant.is_some() {
                            return;
                        }
                        let player_mass = util::radius_to_mass(player.radius);
                        let drop_mass = player_mass * 0.2;
                        if let Some(mass) = player.try_drop_mass(drop_mass) {
                            player.rush();

                            let mut spore = spore::Spore::random();
                            spore.x = player.x;
                            spore.y = player.y;
                            spore.radius = util::mass_to_radius(mass);

                            let packet = proto_util::update_spore_pack(&spore);

                            self.spore_map.insert(spore.id.clone(), spore);

                            self.broadcast_packet(&packet);
                        }
                    }
                }
            }
            _ => {
                warn!("unknown command: {:?}", command);
            }
        }
    }

    fn broadcast_packet(&self, packet: &proto::Packet) {
        let bytes = packet.encode_to_vec().into();
        self.broadcast_bytes(bytes);
    }

    fn broadcast_bytes(&self, bytes: Bytes) {
        self.client_map
            .values()
            .filter(|client| client.player.is_some())
            .for_each(|client| {
                let bytes = bytes.clone();
                let _ = client
                    .client_agent_command_sender
                    .send(command::Command::SendBytes { bytes });
            });
    }

    fn spawn_spore(&mut self) {
        let spore = spore::Spore::random();

        let packet = proto_util::update_spore_pack(&spore);

        self.spore_map.insert(spore.id.clone(), spore);

        self.broadcast_packet(&packet);
    }

    fn tick_player(&mut self, delta: Duration) {
        let mut spore_packet_list = vec![];

        let player_list = self
            .client_map
            .values_mut()
            .flat_map(|client| client.player.as_mut());

        for player in player_list {
            player.tick(delta);

            let drop_mass_probability = player.radius / (MAX_SPORE_COUNT as f64 * 4.0);
            if rand::random::<f64>() < drop_mass_probability {
                let drop_mass = util::radius_to_mass((5.0 + player.radius / 50.0).min(15.0));
                if let Some(mass) = player.try_drop_mass(drop_mass) {
                    let mut spore = spore::Spore::random();
                    spore.x = player.x;
                    spore.y = player.y;
                    spore.radius = util::mass_to_radius(mass);

                    let packet = proto_util::update_spore_pack(&spore);
                    spore_packet_list.push(packet);

                    self.spore_map.insert(spore.id.clone(), spore);
                }
            }
        }

        self.sync_player();

        for spore_packet in spore_packet_list {
            self.broadcast_packet(&spore_packet);
        }
    }

    fn sync_player(&self) {
        let player_list = self
            .client_map
            .values()
            .filter_map(|client| client.player.as_ref())
            .collect::<Vec<_>>();

        let packet = proto_util::update_player_batch_packet(&player_list);
        self.broadcast_packet(&packet);
    }
}
