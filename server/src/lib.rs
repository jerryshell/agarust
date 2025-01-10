pub mod hub;
pub mod proto;

use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use nanoid::nanoid;
use prost::Message as _;
use std::{io::Cursor, net::SocketAddr};
use tokio::{
    net::TcpStream,
    sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
use tracing::{error, info, warn};

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
    UnregisterClientAgent(ClientInfo),
}

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

    let unregister_command = Command::UnregisterClientAgent(client_info);
    let _ = hub_command_sender.send(unregister_command);
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
