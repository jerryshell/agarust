use crate::{
    command::{ClientRegisterEntry, Command},
    proto, proto_util,
};
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use prost::Message as _;
use std::{io::Cursor, net::SocketAddr, sync::Arc, time::Duration};
use tokio::{
    net::TcpStream,
    sync::{
        mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
        Mutex,
    },
};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
use tracing::{error, warn};

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

        let command_send_result = self.hub_command_sender.send(Command::RegisterClient {
            client_register_entry: ClientRegisterEntry {
                socket_addr: self.socket_addr,
                connection_id: self.connection_id.clone(),
                command_sender,
            },
        });
        if let Err(error) = command_send_result {
            error!("send Command::RegisterClient error: {:?}", error);
            return;
        }

        tokio::select! {
            _ = (&mut client_reader_task) => client_writer_task.abort(),
            _ = (&mut client_writer_task) => client_reader_task.abort(),
        };
    }
}

fn handle_packet(
    connection_id: String,
    packet: proto::Packet,
    hub_command_sender: UnboundedSender<Command>,
) {
    if let Some(data) = packet.clone().data {
        match data {
            proto::packet::Data::Chat(_chat) => {
                let _ = hub_command_sender.send(Command::BroadcastPacket { packet });
            }
            proto::packet::Data::UpdatePlayerDirectionAngle(update_player_direction_angle) => {
                let direction_angle = update_player_direction_angle.direction_angle;
                let _ = hub_command_sender.send(Command::UpdatePlayerDirectionAngle {
                    connection_id,
                    direction_angle,
                });
            }
            proto::packet::Data::ConsumeSpore(consume_spore) => {
                let _ = hub_command_sender.send(Command::ConsumeSpore {
                    connection_id,
                    spore_id: consume_spore.spore_id,
                });
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
                if let Message::Binary(bytes) = message {
                    match proto::Packet::decode(Cursor::new(bytes)) {
                        Ok(packet) => {
                            let hub_command_sender = hub_command_sender.clone();
                            handle_packet(connection_id.clone(), packet, hub_command_sender);
                        }
                        Err(error) => {
                            warn!("proto decode error {:?}: {:?}", socket_addr, error);
                            continue;
                        }
                    }
                }
            }
            Err(error) => {
                warn!("client_reader error {:?}: {:?}", socket_addr, error);
                break;
            }
        }
    }
}

async fn client_writer_pump(
    connection_id: String,
    mut command_receiver: UnboundedReceiver<Command>,
    client_writer: SplitSink<WebSocketStream<TcpStream>, Message>,
) {
    let client_writer = Arc::new(Mutex::new(client_writer));
    while let Some(command) = command_receiver.recv().await {
        let client_writer = client_writer.clone();
        match command {
            Command::Hello => {
                let packet = proto_util::hello_packet(connection_id.clone());
                let raw_data = packet.encode_to_vec();
                let message = Message::binary(raw_data);
                let mut client_writer = client_writer.lock().await;
                let _ = client_writer.send(message).await;
            }
            Command::SendPacket { packet } => {
                let raw_data = packet.encode_to_vec();
                let mut client_writer = client_writer.lock().await;
                let _ = client_writer.send(Message::binary(raw_data)).await;
            }
            Command::SendRawData { raw_data } => {
                let mut client_writer = client_writer.lock().await;
                let _ = client_writer.send(Message::binary(raw_data)).await;
            }
            Command::UpdateSporeBatch { spore_batch } => {
                tokio::spawn(async move {
                    for spore_window in spore_batch.windows(32) {
                        let packet = proto_util::update_spore_batch_packet(spore_window);
                        let raw_data = packet.encode_to_vec();
                        {
                            let mut client_writer = client_writer.lock().await;
                            let _ = client_writer.send(Message::binary(raw_data)).await;
                        }
                        tokio::time::sleep(Duration::from_millis(50)).await;
                    }
                });
            }
            _ => {
                warn!("unknow command: {:?}", command);
            }
        }
    }
}
