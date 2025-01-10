pub mod proto;

use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use nanoid::nanoid;
use prost::Message as _;
use std::{collections::HashMap, io::Cursor, net::SocketAddr};
use tokio::{
    net::TcpStream,
    sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
use tracing::{error, info, warn};

pub async fn handle_tcp_stream(
    tcp_stream: TcpStream,
    addr: SocketAddr,
    hub_command_sender: UnboundedSender<Command>,
) {
    let ws_stream = tokio_tungstenite::accept_async(tcp_stream)
        .await
        .expect("Error during the websocket handshake occurred");

    info!("Accept ws_stream: {:?}", addr);

    let (client_writer, client_reader) = ws_stream.split();

    let (command_sender, command_receiver) = unbounded_channel::<Command>();

    let connection_id = nanoid!();

    let client_info = ClientInfo {
        addr,
        connection_id,
    };

    let client_agent = {
        let client_info = client_info.clone();
        ClientAgent {
            client_info,
            command_sender,
        }
    };

    let client_agent_register_command = Command::RegisterClientAgent(client_agent);
    let command_send_result = hub_command_sender.send(client_agent_register_command);
    if let Err(send_error) = command_send_result {
        error!("client_agent_register_command send_error: {:?}", send_error);
        return;
    }

    let mut client_reader_task = {
        let client_info = client_info.clone();
        tokio::spawn(async move { client_reader_pump(client_info, client_reader).await })
    };

    let mut client_writer_task = {
        let client_info = client_info.clone();
        tokio::spawn(async move {
            client_writer_pump(client_info, client_writer, command_receiver).await
        })
    };

    tokio::select! {
        _ = (&mut client_reader_task) => client_writer_task.abort(),
        _ = (&mut client_writer_task) => client_reader_task.abort(),
    };
}

async fn client_reader_pump(
    client_info: ClientInfo,
    mut client_reader: SplitStream<WebSocketStream<TcpStream>>,
) {
    while let Some(read_result) = client_reader.next().await {
        match read_result {
            Ok(message) => {
                info!("message {:?}", message);
                if let Message::Binary(bytes) = message {
                    match proto::Packet::decode(Cursor::new(bytes)) {
                        Ok(mut packet) => {
                            packet.connection_id = client_info.connection_id.clone();
                            info!("packet {:?}", packet);
                        }
                        Err(error) => {
                            warn!(
                                "proto decode error from {:?}: {:?}, close connect",
                                client_info.addr, error
                            );
                            continue;
                        }
                    }
                }
            }
            Err(error) => {
                warn!(
                    "error from {:?}: {:?}, close connect",
                    client_info.addr, error
                );
                break;
            }
        }
    }
}

async fn client_writer_pump(
    client_info: ClientInfo,
    mut client_writer: SplitSink<WebSocketStream<TcpStream>, Message>,
    mut command_receiver: UnboundedReceiver<Command>,
) {
    while let Some(command) = command_receiver.recv().await {
        match command {
            Command::Hello => {
                let packet = proto::Packet {
                    connection_id: client_info.connection_id.clone(),
                    data: Some(proto::packet::Data::Hello(proto::Hello {
                        connection_id: client_info.connection_id.clone(),
                    })),
                };
                let bytes = packet.encode_to_vec();
                let message = Message::binary(bytes);
                let _ = client_writer.send(message).await;
            }
            _ => {
                warn!("ClientAgent unknow command: {:?}", command);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClientInfo {
    pub addr: SocketAddr,
    pub connection_id: String,
}

#[derive(Debug, Clone)]
pub struct ClientAgent {
    pub client_info: ClientInfo,
    pub command_sender: UnboundedSender<Command>,
}

#[derive(Debug, Clone)]
pub enum Command {
    Hello,
    RegisterClientAgent(ClientAgent),
}

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
            _ => {
                warn!("Hub unknow command: {:?}", command);
            }
        }
    }
}
