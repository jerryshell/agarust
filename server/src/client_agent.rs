use crate::*;
use anyhow::Result;
use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use prost::Message as _;
use sqlx::query_as;
use std::{io::Cursor, net::SocketAddr, sync::Arc, time::Duration};
use tokio::{
    net::TcpStream,
    sync::{
        mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel},
        oneshot,
    },
    time::interval,
};
use tokio_tungstenite::{WebSocketStream, tungstenite::Message};
use tracing::{error, info, warn};

#[derive(Debug)]
pub struct ClientAgent {
    pub ws_stream: WebSocketStream<TcpStream>,
    pub socket_addr: SocketAddr,
    pub connection_id: Arc<str>,
    pub db: db::Db,
    pub hub_command_sender: UnboundedSender<command::Command>,
    pub client_agent_command_sender: UnboundedSender<command::Command>,
    pub client_agent_command_receiver: UnboundedReceiver<command::Command>,
    pub db_player: Option<db::Player>,
}

impl ClientAgent {
    pub async fn new(
        ws_stream: WebSocketStream<TcpStream>,
        socket_addr: SocketAddr,
        db: db::Db,
        hub_command_sender: UnboundedSender<command::Command>,
    ) -> Result<Self> {
        let (client_agent_command_sender, client_agent_command_receiver) =
            unbounded_channel::<command::Command>();

        let connection_id = {
            let client_agent_command_sender = client_agent_command_sender.clone();
            let (response_sender, response_receiver) = oneshot::channel();
            hub_command_sender.send(command::Command::RegisterClientAgent {
                socket_addr,
                client_agent_command_sender,
                response_sender,
            })?;
            response_receiver.await?
        };

        let client_agent = Self {
            ws_stream,
            socket_addr,
            connection_id,
            db,
            hub_command_sender,
            client_agent_command_sender,
            client_agent_command_receiver,
            db_player: None,
        };

        Ok(client_agent)
    }

    pub async fn run(mut self) {
        let hello_packet = proto_util::hello_packet(self.connection_id.clone());
        self.send_packet(&hello_packet).await;

        loop {
            tokio::select! {
                ws_stream_next = self.ws_stream.next() => {
                    match ws_stream_next {
                        Some(ws_stream_next_result) => {
                            match ws_stream_next_result {
                                Ok(ws_stream_message) => {
                                    self.handle_ws_stream_message(ws_stream_message).await;
                                },
                                Err(e) => {
                                    warn!("ws_stream_next_result error, disconnect {:?}: {:?}", self.socket_addr, e);
                                    break;
                                },
                            }
                        },
                        None => {
                            warn!("ws_stream_next None, disconnect {:?}", self.socket_addr);
                            break;
                        },
                    }
                },
                command_recv = self.client_agent_command_receiver.recv() => {
                    match command_recv {
                        Some(command) => {
                            self.handle_command(command).await;
                        },
                        None => {
                            warn!("command_recv None, disconnect {:?}", self.socket_addr);
                            break;
                        },
                    }
                },
            };
        }

        let _ = self
            .hub_command_sender
            .send(command::Command::UnregisterClientAgent {
                connection_id: self.connection_id,
            });
    }

    async fn handle_ws_stream_message(&mut self, ws_stream_message: Message) {
        match ws_stream_message {
            Message::Binary(bytes) => match proto::Packet::decode(Cursor::new(bytes)) {
                Ok(packet) => {
                    self.handle_packet(&packet).await;
                }
                Err(e) => {
                    warn!("proto decode error {:?}: {:?}", self, e);
                }
            },
            Message::Close(close_frame) => {
                info!("client close_frame: {:?}", close_frame);
                let _ = self.ws_stream.close(None).await;
            }
            _ => {
                warn!("unkonwn message: {:?}", ws_stream_message);
            }
        }
    }

    async fn handle_packet(&mut self, packet: &proto::Packet) {
        let data = match &packet.data {
            Some(data) => data,
            None => {
                warn!("packet has no data");
                return;
            }
        };
        match data {
            proto::packet::Data::Ping(_) => {
                self.send_packet(packet).await;
            }
            proto::packet::Data::Login(login) => {
                let auth = match self.db.auth_get_one_by_username(&login.username).await {
                    Ok(auth) => auth,
                    Err(e) => {
                        warn!("auth query error: {:?}", e);
                        let packet =
                            proto_util::login_err_packet("incorrect username or password".into());
                        self.send_packet(&packet).await;
                        return;
                    }
                };

                match bcrypt::verify(&login.password, &auth.password) {
                    Ok(valid) => {
                        if !valid {
                            warn!("bcrypt valid false");
                            let packet = proto_util::login_err_packet(
                                "incorrect username or password".into(),
                            );
                            self.send_packet(&packet).await;
                            return;
                        }
                    }
                    Err(e) => {
                        warn!("bcrypt verify error: {:?}", e);
                        let packet =
                            proto_util::login_err_packet("incorrect username or password".into());
                        self.send_packet(&packet).await;
                        return;
                    }
                }

                let player = match self.db.player_get_one_by_auth_id(auth.id).await {
                    Ok(player) => player,
                    Err(e) => {
                        warn!("player query error: {:?}", e);
                        let packet =
                            proto_util::login_err_packet("incorrect username or password".into());
                        self.send_packet(&packet).await;
                        return;
                    }
                };

                self.db_player = Some(player);

                let packet = proto_util::login_ok_packet();
                self.send_packet(&packet).await;
            }
            proto::packet::Data::Register(register) => {
                let mut transaction = match self.db.db_pool.begin().await {
                    Ok(transaction) => transaction,
                    Err(e) => {
                        warn!("transaction begin error: {:?}", e);
                        let packet =
                            proto_util::register_err_packet("transaction begin error".into());
                        self.send_packet(&packet).await;
                        return;
                    }
                };

                if register.username.is_empty() {
                    let packet = proto_util::register_err_packet("username is empty".into());
                    self.send_packet(&packet).await;
                    return;
                }
                if register.username.len() > 16 {
                    let packet = proto_util::register_err_packet("username too long".into());
                    self.send_packet(&packet).await;
                    return;
                }

                let query_result = query_as!(
                    db::Auth,
                    r#"SELECT * FROM auth WHERE username = ? LIMIT 1"#,
                    register.username
                )
                .fetch_one(&mut *transaction)
                .await;

                if query_result.is_ok() {
                    let packet = proto_util::register_err_packet("username already exists".into());
                    self.send_packet(&packet).await;
                    return;
                }

                let password = match bcrypt::hash(&register.password, bcrypt::DEFAULT_COST) {
                    Ok(password) => password,
                    Err(_) => {
                        let packet = proto_util::register_err_packet("password hash error".into());
                        self.send_packet(&packet).await;
                        return;
                    }
                };

                let query_result = query_as!(
                    db::Auth,
                    r#"INSERT INTO auth ( username, password ) VALUES ( ?, ? )"#,
                    register.username,
                    password,
                )
                .execute(&mut *transaction)
                .await;

                let auth_id = match query_result {
                    Ok(query_result) => query_result.last_insert_rowid(),
                    Err(e) => {
                        warn!("auth insert error: {:?}", e);
                        let packet = proto_util::register_err_packet("auth insert error".into());
                        self.send_packet(&packet).await;
                        return;
                    }
                };

                // force alpha 0xFFFF
                let color = register.color | 0xFFFF;
                let query_result = query_as!(
                    db::Player,
                    r#"INSERT INTO player ( auth_id, nickname, color ) VALUES ( ?, ?, ? )"#,
                    auth_id,
                    register.username,
                    color,
                )
                .execute(&mut *transaction)
                .await;

                if let Err(e) = query_result {
                    warn!("player insert error: {:?}", e);
                    let packet = proto_util::register_err_packet("player insert error".into());
                    self.send_packet(&packet).await;
                    return;
                }

                if let Err(e) = transaction.commit().await {
                    warn!("transaction commit error: {:?}", e);
                    let packet = proto_util::register_err_packet("transaction commit error".into());
                    self.send_packet(&packet).await;
                    return;
                }

                let packet = proto_util::register_ok_packet();
                self.send_packet(&packet).await;
            }
            proto::packet::Data::Join(_) => {
                let db_player = match self.db_player.as_ref() {
                    Some(db_player) => db_player,
                    None => {
                        warn!("join without login");
                        return;
                    }
                };
                let _ = self.hub_command_sender.send(command::Command::Join {
                    connection_id: self.connection_id.clone(),
                    player_db_id: db_player.id,
                    nickname: db_player.nickname.clone(),
                    color: db_player.color,
                });
            }
            proto::packet::Data::Chat(chat) => {
                let _ = self.hub_command_sender.send(command::Command::Chat {
                    connection_id: self.connection_id.clone(),
                    msg: chat.msg.as_str().into(),
                });
            }
            proto::packet::Data::UpdatePlayerDirectionAngle(update_player_direction_angle) => {
                let _ =
                    self.hub_command_sender
                        .send(command::Command::UpdatePlayerDirectionAngle {
                            connection_id: self.connection_id.clone(),
                            direction_angle: update_player_direction_angle.direction_angle,
                        });
            }
            proto::packet::Data::ConsumeSpore(consume_spore) => {
                let _ = self
                    .hub_command_sender
                    .send(command::Command::ConsumeSpore {
                        connection_id: self.connection_id.clone(),
                        spore_id: consume_spore.spore_id.as_str().into(),
                    });
            }
            proto::packet::Data::ConsumePlayer(consume_player) => {
                let _ = self
                    .hub_command_sender
                    .send(command::Command::ConsumePlayer {
                        connection_id: self.connection_id.clone(),
                        victim_connection_id: consume_player.victim_connection_id.as_str().into(),
                    });
            }
            proto::packet::Data::Rush(_) => {
                let _ = self.hub_command_sender.send(command::Command::Rush {
                    connection_id: self.connection_id.clone(),
                });
            }
            proto::packet::Data::Disconnect(_) => {
                let _ = self
                    .client_agent_command_sender
                    .send(command::Command::DisconnectClinet);
            }
            proto::packet::Data::LeaderboardRequest(_) => {
                let leaderboard_entry_list = match self.db.player_get_list(100).await {
                    Ok(player_list) => player_list
                        .iter()
                        .enumerate()
                        .map(|(index, player)| command::LeaderboardEntry {
                            rank: (index + 1) as u64,
                            player_nickname: player.nickname.clone(),
                            score: player.best_score as u64,
                        })
                        .collect::<Vec<_>>(),
                    Err(e) => {
                        error!("fetch leaderboard error: {:?}", e);
                        return;
                    }
                };

                let packet = proto_util::leaderboard_response(&leaderboard_entry_list);
                self.send_packet(&packet).await;
            }
            _ => {
                warn!("unknown packet data: {:?}", data);
            }
        }
    }

    async fn handle_command(&mut self, command: command::Command) {
        match command {
            command::Command::SendPacket { packet } => {
                self.send_packet(&packet).await;
            }
            command::Command::SendBytes { bytes } => {
                self.send_bytes(bytes).await;
            }
            command::Command::UpdateSporeBatch { spore_batch } => {
                const SEND_INTERNAL_DURATION: Duration = Duration::from_millis(20);
                const SPORE_CHUNKS: usize = 20;
                let client_agent_command_sender = self.client_agent_command_sender.clone();
                tokio::spawn(async move {
                    let mut send_interval = interval(SEND_INTERNAL_DURATION);
                    for spore_chunk in spore_batch.chunks(SPORE_CHUNKS) {
                        let packet = proto_util::update_spore_batch_packet(spore_chunk);
                        let bytes = packet.encode_to_vec().into();
                        let _ =
                            client_agent_command_sender.send(command::Command::SendBytes { bytes });
                        send_interval.tick().await;
                    }
                });
            }
            command::Command::SyncPlayerBestScore { current_score } => {
                let db_player_id = {
                    let db_player = match self.db_player.as_mut() {
                        Some(db_player) => db_player,
                        None => {
                            warn!("sync player best score without login");
                            return;
                        }
                    };
                    if db_player.best_score > current_score {
                        return;
                    }

                    db_player.best_score = current_score;

                    db_player.id
                };

                if let Err(e) = self
                    .db
                    .player_update_best_score_by_id(current_score, db_player_id)
                    .await
                {
                    error!("UPDATE player SET best_score error: {:?}", e);
                }
            }
            command::Command::DisconnectClinet => {
                warn!("Command::DisconnectClinet");
                let _ = self.ws_stream.close(None).await;
            }
            _ => {
                warn!("unknown command: {:?}", command);
            }
        }
    }

    async fn send_packet(&mut self, packet: &proto::Packet) {
        let bytes = packet.encode_to_vec().into();
        self.send_bytes(bytes).await;
    }

    async fn send_bytes(&mut self, bytes: Bytes) {
        let _ = self.ws_stream.send(Message::binary(bytes)).await;
    }
}
