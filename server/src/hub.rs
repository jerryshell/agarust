use crate::{ClientAgent, Command};
use std::collections::HashMap;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tracing::{info, warn};

#[derive(Debug)]
pub struct Hub {
    pub client_agent_map: HashMap<String, ClientAgent>,
    pub command_sender: UnboundedSender<Command>,
    pub command_receiver: UnboundedReceiver<Command>,
}

impl Default for Hub {
    fn default() -> Self {
        let (command_sender, command_receiver) = unbounded_channel::<Command>();
        Self {
            client_agent_map: HashMap::new(),
            command_sender,
            command_receiver,
        }
    }
}

impl Hub {
    pub fn new() -> Self {
        let (command_sender, command_receiver) = unbounded_channel::<Command>();
        Self {
            client_agent_map: HashMap::new(),
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
            Command::RegisterClientAgent(client_agent) => {
                info!("RegisterClientAgent: {:?}", client_agent);
                let client_agent_command_sender = client_agent.command_sender.clone();
                let key = client_agent.client_info.connection_id.clone();
                self.client_agent_map.insert(key, client_agent);
                info!("client_agent_map: {:?}", self.client_agent_map);
                let _ = client_agent_command_sender.send(Command::Hello);
            }
            Command::UnregisterClientAgent(client_info) => {
                info!("UnregisterClientAgent: {:?}", client_info);
                let key = client_info.connection_id;
                self.client_agent_map.remove(&key);
                info!("client_agent_map: {:?}", self.client_agent_map);
            }
            _ => {
                warn!("unknow command: {:?}", command);
            }
        }
    }
}
