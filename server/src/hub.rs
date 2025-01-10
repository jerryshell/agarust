use crate::{
    command::{ClientRegisterEntry, Command},
    proto_util,
};
use nanoid::nanoid;
use prost::Message;
use std::{collections::HashMap, time::Duration};
use tokio::{
    sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    time::Instant,
};
use tracing::{info, warn};

const TICK_DELTA: Duration = Duration::from_millis(50);
const SPORE_BOUND: f64 = 3000.0;
const MAX_SPORE_COUNT: usize = 1000;

#[derive(Debug, Clone)]
pub struct Player {
    pub connection_id: String,
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub radius: f64,
    pub direction_angle: f64,
    pub speed: f64,
    pub color: i32,
}

#[derive(Debug, Clone)]
pub struct Spore {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub radius: f64,
}

impl Spore {
    pub fn random() -> Self {
        let x = (rand::random::<f64>() * 2.0 - 1.0) * SPORE_BOUND;
        let y = (rand::random::<f64>() * 2.0 - 1.0) * SPORE_BOUND;
        let radius = (rand::random::<f64>() * 3.0 + 10.0).max(5.0);
        Self {
            id: nanoid!(),
            x,
            y,
            radius,
        }
    }
}

#[derive(Debug)]
pub struct Hub {
    pub client_map: HashMap<String, ClientRegisterEntry>,
    pub player_map: HashMap<String, Player>,
    pub spore_map: HashMap<String, Spore>,
    pub command_sender: UnboundedSender<Command>,
    pub command_receiver: UnboundedReceiver<Command>,
}

impl Default for Hub {
    fn default() -> Self {
        Self::new()
    }
}

impl Hub {
    pub fn new() -> Self {
        let (command_sender, command_receiver) = unbounded_channel::<Command>();
        Self {
            client_map: HashMap::new(),
            player_map: HashMap::new(),
            spore_map: HashMap::new(),
            command_sender,
            command_receiver,
        }
    }

    pub async fn run(&mut self) {
        for _ in 0..MAX_SPORE_COUNT {
            self.new_spore().await;
        }

        let tick_player_task = {
            let hub_command_sender = self.command_sender.clone();
            tokio::spawn(async move {
                let mut last_tick = Instant::now();
                loop {
                    let _ = hub_command_sender.send(Command::TickPlayer);
                    let timeout = TICK_DELTA.saturating_sub(last_tick.elapsed());
                    tokio::time::sleep(timeout).await;
                    last_tick = Instant::now();
                }
            })
        };

        while let Some(command) = self.command_receiver.recv().await {
            self.handle_command(command).await;
        }

        tick_player_task.abort();
    }

    async fn new_spore(&mut self) {
        let spore = Spore::random();
        self.spore_map.insert(spore.id.clone(), spore);
    }

    async fn handle_command(&mut self, command: Command) {
        match command {
            Command::RegisterClient {
                client_register_entry,
            } => {
                info!("RegisterClient: {:?}", client_register_entry);
                let client_agent_command_sender = client_register_entry.command_sender.clone();
                let connection_id = client_register_entry.connection_id.clone();
                self.client_map
                    .insert(connection_id.clone(), client_register_entry);
                let _ = client_agent_command_sender.send(Command::Hello);

                let player = Player {
                    connection_id: connection_id.clone(),
                    name: connection_id.clone(),
                    x: 1.0,
                    y: 2.0,
                    radius: 20.0,
                    direction_angle: 0.0,
                    speed: 200.0,
                    color: 1,
                };
                self.player_map.insert(connection_id, player);

                let packet = proto_util::update_spore_batch_packet(&self.spore_map);
                let _ = client_agent_command_sender.send(Command::SendPacket { packet });
            }
            Command::UnregisterClient { connection_id } => {
                info!("UnregisterClient: {:?}", connection_id);
                self.client_map.remove(&connection_id);
                self.player_map.remove(&connection_id);
            }
            Command::Broadcast { packet } => {
                info!("Broadcast: {:?}", packet);
                let raw_data = packet.encode_to_vec();
                self.client_map.values().for_each(|client| {
                    let raw_data = raw_data.clone();
                    let _ = client
                        .command_sender
                        .send(Command::SendRawData { raw_data });
                })
            }
            Command::TickPlayer => {
                if self.player_map.is_empty() {
                    return;
                }

                // tick player
                self.tick_player().await;

                // sync player
                let packet = proto_util::update_player_batch_packet(&self.player_map);
                let raw_data = packet.encode_to_vec();
                self.client_map.values().for_each(|client| {
                    let _ = client.command_sender.send(Command::SendRawData {
                        raw_data: raw_data.clone(),
                    });
                })
            }
            Command::UpdatePlayerDirectionAngle {
                connection_id,
                direction_angle,
            } => {
                info!("UpdatePlayerDirectionAngle {connection_id:?} {direction_angle:?}");
                if let Some(player) = self.player_map.get_mut(&connection_id) {
                    player.direction_angle = direction_angle;
                }
            }
            _ => {
                warn!("unknow command: {:?}", command);
            }
        }
    }

    async fn tick_player(&mut self) {
        self.player_map.values_mut().for_each(|player| {
            let new_x =
                player.x + player.speed * player.direction_angle.cos() * TICK_DELTA.as_secs_f64();
            let new_y =
                player.y + player.speed * player.direction_angle.sin() * TICK_DELTA.as_secs_f64();

            player.x = new_x;
            player.y = new_y;
        });
    }
}
