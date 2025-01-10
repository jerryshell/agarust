use crate::{
    command::{ClientRegisterEntry, Command},
    proto,
};
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use prost::Message as _;
use std::{io::Cursor, net::SocketAddr};
use tokio::{
    net::TcpStream,
    sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
use tracing::{error, info, warn};

#[derive(Debug, Clone)]
pub struct ClientAgent {
    pub socket_addr: SocketAddr,
    pub connection_id: String,
    pub hub_command_sender: UnboundedSender<Command>,
}

impl ClientAgent {
    pub async fn run(&self, ws_stream: WebSocketStream<TcpStream>) {
        let (client_writer, client_reader) = ws_stream.split();

        let (command_sender, command_receiver) = unbounded_channel::<Command>();

        let mut client_reader_task = {
            let socket_addr = self.socket_addr;
            let connection_id = self.connection_id.clone();
            let hub_command_sender = self.hub_command_sender.clone();
            tokio::spawn(async move {
                client_reader_pump(
                    socket_addr,
                    connection_id,
                    client_reader,
                    hub_command_sender,
                )
                .await
            })
        };

        let mut client_writer_task = {
            let connection_id = self.connection_id.clone();
            tokio::spawn(async move {
                client_writer_pump(connection_id, command_receiver, client_writer).await
            })
        };

        {
            let socket_addr = self.socket_addr;
            let connection_id = self.connection_id.clone();
            let register_client_command = Command::RegisterClient {
                client_register_entry: ClientRegisterEntry {
                    socket_addr,
                    connection_id,
                    command_sender,
                },
            };
            let command_send_result = self.hub_command_sender.send(register_client_command);
            if let Err(error) = command_send_result {
                error!("register_client_command error: {:?}", error);
                return;
            }
        }

        tokio::select! {
            _ = (&mut client_reader_task) => client_writer_task.abort(),
            _ = (&mut client_writer_task) => client_reader_task.abort(),
        };
    }
}

async fn handle_packet(packet: proto::Packet, hub_command_sender: UnboundedSender<Command>) {
    info!("handle_packet {:?}", packet);
    if let Some(data) = packet.clone().data {
        match data {
            proto::packet::Data::Chat(_chat) => {
                let _ = hub_command_sender.send(Command::Broadcast { packet });
            }
            _ => {
                warn!("unknow packet: {:?}", packet);
            }
        }
    }
}

async fn client_reader_pump(
    socket_addr: SocketAddr,
    connection_id: String,
    mut client_reader: SplitStream<WebSocketStream<TcpStream>>,
    hub_command_sender: UnboundedSender<Command>,
) {
    while let Some(read_result) = client_reader.next().await {
        match read_result {
            Ok(message) => {
                info!("message {:?}", message);
                if let Message::Binary(bytes) = message {
                    match proto::Packet::decode(Cursor::new(bytes)) {
                        Ok(mut packet) => {
                            packet.connection_id = connection_id.clone();
                            let hub_command_sender = hub_command_sender.clone();
                            handle_packet(packet, hub_command_sender).await;
                        }
                        Err(error) => {
                            warn!(
                                "proto decode error from {:?}: {:?}, close connect",
                                socket_addr, error
                            );
                            continue;
                        }
                    }
                }
            }
            Err(error) => {
                warn!("error from {:?}: {:?}, close connect", socket_addr, error);
                break;
            }
        }
    }
}

async fn client_writer_pump(
    connection_id: String,
    mut command_receiver: UnboundedReceiver<Command>,
    mut client_writer: SplitSink<WebSocketStream<TcpStream>, Message>,
) {
    while let Some(command) = command_receiver.recv().await {
        match command {
            Command::Hello => {
                let packet = proto::Packet {
                    connection_id: connection_id.clone(),
                    data: Some(proto::packet::Data::Hello(proto::Hello {})),
                };
                let bytes = packet.encode_to_vec();
                let message = Message::binary(bytes);
                let _ = client_writer.send(message).await;
            }
            Command::SendRawData { raw_data } => {
                let _ = client_writer.send(Message::binary(raw_data)).await;
            }
            _ => {
                warn!("ClientAgent unknow command: {:?}", command);
            }
        }
    }
}
