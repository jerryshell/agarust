use crate::command::{ClientRegisterEntry, Command};
use prost::Message;
use std::collections::HashMap;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tracing::{info, warn};

#[derive(Debug)]
pub struct Hub {
    pub client_map: HashMap<String, ClientRegisterEntry>,
    pub command_sender: UnboundedSender<Command>,
    pub command_receiver: UnboundedReceiver<Command>,
}

impl Default for Hub {
    fn default() -> Self {
        let (command_sender, command_receiver) = unbounded_channel::<Command>();
        Self {
            client_map: HashMap::new(),
            command_sender,
            command_receiver,
        }
    }
}

impl Hub {
    pub fn new() -> Self {
        let (command_sender, command_receiver) = unbounded_channel::<Command>();
        Self {
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
                let key = client_register_entry.connection_id.clone();
                self.client_map.insert(key, client_register_entry);
                info!("client_agent_map: {:?}", self.client_map);
                let _ = client_agent_command_sender.send(Command::Hello);
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
