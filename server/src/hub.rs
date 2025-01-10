use crate::{
    command::{ClientRegisterEntry, Command},
    proto,
};
use prost::Message;
use std::collections::HashMap;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tracing::{info, warn};

#[derive(Debug, Clone)]
pub struct Player {
    pub connection_id: String,
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub radius: f64,
    pub direction: f64,
    pub speed: f64,
    pub color: i32,
}

#[derive(Debug)]
pub struct Hub {
    pub player_map: HashMap<String, Player>,
    pub client_map: HashMap<String, ClientRegisterEntry>,
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
            player_map: HashMap::new(),
            client_map: HashMap::new(),
            command_sender,
            command_receiver,
        }
    }

    pub async fn run(&mut self) {
        while let Some(command) = self.command_receiver.recv().await {
            info!("command: {:?}", command);
            self.handle_command(command).await;
        }
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
                info!("client_agent_map: {:?}", self.client_map);
                let _ = client_agent_command_sender.send(Command::Hello);

                let player = Player {
                    connection_id: connection_id.clone(),
                    name: "todo!()".to_string(),
                    x: 1.0,
                    y: 2.0,
                    radius: 20.0,
                    direction: 0.0,
                    speed: 200.0,
                    color: 1,
                };
                let _ = client_agent_command_sender.send(Command::SendPacket {
                    packet: crate::proto::Packet {
                        connection_id: connection_id.clone(),
                        data: Some(proto::packet::Data::UpdatePlayer(proto::UpdatePlayer {
                            connection_id: player.connection_id,
                            name: player.name,
                            x: player.x,
                            y: player.y,
                            radius: player.radius,
                            direction: player.direction,
                            speed: player.speed,
                            color: player.color,
                        })),
                    },
                });
            }
            Command::UnregisterClient { connection_id } => {
                info!("UnregisterClient: {:?}", connection_id);
                self.client_map.remove(&connection_id);
                info!("client_agent_map: {:?}", self.client_map);
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
            _ => {
                warn!("unknow command: {:?}", command);
            }
        }
    }
}
