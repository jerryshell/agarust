use crate::*;

use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use prost::Message as _;
use sqlx::query_as;
use std::{io::Cursor, net::SocketAddr, sync::Arc, time::Duration};
use tokio::{
    net::TcpStream,
    sync::{
        mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
        Mutex, RwLock,
    },
};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
use tracing::{error, warn};

#[derive(Debug)]
pub struct ClientAgent {
    pub socket_addr: SocketAddr,
    pub connection_id: Arc<str>,
    pub db_pool: sqlx::Pool<sqlx::Sqlite>,
    pub hub_command_sender: UnboundedSender<command::Command>,
    pub client_agent_command_sender: UnboundedSender<command::Command>,
    pub client_agent_command_receiver: UnboundedReceiver<command::Command>,
    pub db_player: Arc<RwLock<Option<db::Player>>>,
}

impl ClientAgent {
    pub fn new(
        socket_addr: SocketAddr,
        connection_id: Arc<str>,
        db_pool: sqlx::Pool<sqlx::Sqlite>,
        hub_command_sender: UnboundedSender<command::Command>,
    ) -> Self {
        let (client_agent_command_sender, client_agent_command_receiver) =
            unbounded_channel::<command::Command>();
        let db_player = Arc::new(RwLock::new(None));
        Self {
            socket_addr,
            connection_id,
            db_pool,
            hub_command_sender,
            client_agent_command_sender,
            client_agent_command_receiver,
            db_player,
        }
    }

    pub async fn run(mut self, ws_stream: WebSocketStream<TcpStream>) {
        let (client_writer, mut client_reader) = ws_stream.split();
        let client_writer = Arc::new(Mutex::new(client_writer));
        loop {
            tokio::select! {
                client_reader_next = client_reader.next() => {
                    match client_reader_next {
                        Some(read_message_result) => {
                            match read_message_result {
                                Ok(client_reader_message) => {
                                    let client_writer = client_writer.clone();
                                    self.handle_client_reader_message(client_reader_message, client_writer).await;
                                },
                                Err(e) => {
                                    warn!("client_reader error, disconnect {:?}: {:?}", self.socket_addr, e);
                                    break;
                                },
                            }
                        },
                        None => {
                            warn!("client_reader next None, disconnect {:?}", self.socket_addr);
                            break;
                        },
                    }
                },
                command_recv = self.client_agent_command_receiver.recv() => {
                    match command_recv {
                        Some(command) => {
                            let client_writer = client_writer.clone();
                            self.handle_command(command, client_writer).await;
                        },
                        None => {
                            warn!("client_agent_command_receiver recv None, disconnect {:?}", self.socket_addr);
                            break;
                        },
                    }
                },
            };
        }
    }

    async fn handle_client_reader_message(
        &mut self,
        client_reader_message: Message,
        client_writer: Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>>,
    ) {
        match client_reader_message {
            Message::Binary(bytes) => match proto::Packet::decode(Cursor::new(bytes)) {
                Ok(packet) => {
                    self.handle_client_reader_packet(packet).await;
                }
                Err(e) => {
                    warn!("proto decode error {:?}: {:?}", self, e);
                }
            },
            Message::Close(close_frame) => {
                info!("client close_frame: {:?}", close_frame);
                let _ = client_writer.lock().await.close().await;
            }
            _ => {
                warn!("unkonwn message: {:?}", client_reader_message);
            }
        }
    }

    async fn handle_client_reader_packet(&mut self, packet: proto::Packet) {
        if let Some(data) = packet.data {
            match data {
                proto::packet::Data::Ping(ping) => {
                    let packet = proto::Packet {
                        data: Some(proto::packet::Data::Ping(ping)),
                    };
                    let _ = self
                        .client_agent_command_sender
                        .send(command::Command::SendPacket { packet });
                }
                proto::packet::Data::Login(login) => {
                    let username = login.username;
                    let password = login.password;

                    let query_result = query_as!(
                        db::Auth,
                        r#"SELECT * FROM auth WHERE username = ? LIMIT 1"#,
                        username
                    )
                    .fetch_one(&self.db_pool)
                    .await;

                    let auth = match query_result {
                        Ok(auth) => auth,
                        Err(e) => {
                            warn!("auth query error: {:?}", e);
                            let packet = proto_util::login_err_packet(
                                "incorrect username or password".into(),
                            );
                            let _ = self
                                .client_agent_command_sender
                                .send(command::Command::SendPacket { packet });
                            return;
                        }
                    };

                    match bcrypt::verify(password, &auth.password) {
                        Ok(valid) => {
                            if !valid {
                                warn!("bcrypt valid false");
                                let packet = proto_util::login_err_packet(
                                    "incorrect username or password".into(),
                                );
                                let _ = self
                                    .client_agent_command_sender
                                    .send(command::Command::SendPacket { packet });
                                return;
                            }
                        }
                        Err(e) => {
                            warn!("bcrypt verify error: {:?}", e);
                            let packet = proto_util::login_err_packet(
                                "incorrect username or password".into(),
                            );
                            let _ = self
                                .client_agent_command_sender
                                .send(command::Command::SendPacket { packet });
                            return;
                        }
                    }

                    let query_result = query_as!(
                        db::Player,
                        r#"SELECT * FROM player WHERE auth_id = ? LIMIT 1"#,
                        auth.id
                    )
                    .fetch_one(&self.db_pool)
                    .await;

                    let player = match query_result {
                        Ok(player) => player,
                        Err(e) => {
                            warn!("player query error: {:?}", e);
                            let packet = proto_util::login_err_packet(
                                "incorrect username or password".into(),
                            );
                            let _ = self
                                .client_agent_command_sender
                                .send(command::Command::SendPacket { packet });
                            return;
                        }
                    };

                    {
                        let mut db_player = self.db_player.write().await;
                        *db_player = Some(player);
                    }

                    let packet = proto_util::login_ok_packet();
                    let _ = self
                        .client_agent_command_sender
                        .send(command::Command::SendPacket { packet });
                }
                proto::packet::Data::Register(register) => {
                    let username = register.username;
                    let password = register.password;
                    let color = register.color;

                    let mut transaction = match self.db_pool.begin().await {
                        Ok(transaction) => transaction,
                        Err(e) => {
                            warn!("transaction begin error: {:?}", e);
                            let packet =
                                proto_util::register_err_packet("transaction begin error".into());
                            let _ = self
                                .client_agent_command_sender
                                .send(command::Command::SendPacket { packet });
                            return;
                        }
                    };

                    if username.is_empty() {
                        warn!("username is empty: {:?}", username);
                        let packet = proto_util::register_err_packet("username is empty".into());
                        let _ = self
                            .client_agent_command_sender
                            .send(command::Command::SendPacket { packet });
                        return;
                    }

                    if username.len() > 16 {
                        warn!("username too long: {:?}", username);
                        let packet = proto_util::register_err_packet("username too long".into());
                        let _ = self
                            .client_agent_command_sender
                            .send(command::Command::SendPacket { packet });
                        return;
                    }

                    let query_result = query_as!(
                        db::Auth,
                        r#"SELECT * FROM auth WHERE username = ? LIMIT 1"#,
                        username
                    )
                    .fetch_one(&mut *transaction)
                    .await;

                    if query_result.is_ok() {
                        warn!("auth already exists: {:?}", username);
                        let packet =
                            proto_util::register_err_packet("username already exists".into());
                        let _ = self
                            .client_agent_command_sender
                            .send(command::Command::SendPacket { packet });
                        return;
                    }

                    let password = match bcrypt::hash(password, bcrypt::DEFAULT_COST) {
                        Ok(password) => password,
                        Err(e) => {
                            warn!("password hash error: {:?}", e);
                            let packet =
                                proto_util::register_err_packet("password hash error".into());
                            let _ = self
                                .client_agent_command_sender
                                .send(command::Command::SendPacket { packet });
                            return;
                        }
                    };

                    let query_result = query_as!(
                        db::Auth,
                        r#"INSERT INTO auth ( username, password ) VALUES ( ?, ? )"#,
                        username,
                        password,
                    )
                    .execute(&mut *transaction)
                    .await;

                    let auth_id = match query_result {
                        Ok(query_result) => query_result.last_insert_rowid(),
                        Err(e) => {
                            warn!("auth insert error: {:?}", e);
                            let packet =
                                proto_util::register_err_packet("auth insert error".into());
                            let _ = self
                                .client_agent_command_sender
                                .send(command::Command::SendPacket { packet });
                            return;
                        }
                    };

                    let query_result = query_as!(
                        db::Player,
                        r#"INSERT INTO player ( auth_id, nickname, color ) VALUES ( ?, ?, ? )"#,
                        auth_id,
                        username,
                        color,
                    )
                    .execute(&mut *transaction)
                    .await;

                    if let Err(e) = query_result {
                        warn!("player insert error: {:?}", e);
                        let packet = proto_util::register_err_packet("player insert error".into());
                        let _ = self
                            .client_agent_command_sender
                            .send(command::Command::SendPacket { packet });
                        return;
                    }

                    if let Err(e) = transaction.commit().await {
                        warn!("transaction commit error: {:?}", e);
                        let packet =
                            proto_util::register_err_packet("transaction commit error".into());
                        let _ = self
                            .client_agent_command_sender
                            .send(command::Command::SendPacket { packet });
                        return;
                    }

                    let packet = proto_util::register_ok_packet();
                    let _ = self
                        .client_agent_command_sender
                        .send(command::Command::SendPacket { packet });
                }
                proto::packet::Data::Join(_) => {
                    let db_player = self.db_player.read().await;
                    let db_player = match &*db_player {
                        Some(db_player) => db_player,
                        None => {
                            warn!("join without login");
                            let packet =
                                proto_util::register_err_packet("transaction commit error".into());
                            let _ = self
                                .client_agent_command_sender
                                .send(command::Command::SendPacket { packet });
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
                        msg: chat.msg.into(),
                    });
                }
                proto::packet::Data::UpdatePlayerDirectionAngle(update_player_direction_angle) => {
                    let _ = self.hub_command_sender.send(
                        command::Command::UpdatePlayerDirectionAngle {
                            connection_id: self.connection_id.clone(),
                            direction_angle: update_player_direction_angle.direction_angle,
                        },
                    );
                }
                proto::packet::Data::ConsumeSpore(consume_spore) => {
                    let _ = self
                        .hub_command_sender
                        .send(command::Command::ConsumeSpore {
                            connection_id: self.connection_id.clone(),
                            spore_id: consume_spore.spore_id.into(),
                        });
                }
                proto::packet::Data::ConsumePlayer(consume_player) => {
                    let _ = self
                        .hub_command_sender
                        .send(command::Command::ConsumePlayer {
                            connection_id: self.connection_id.clone(),
                            victim_connection_id: consume_player.victim_connection_id.into(),
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
                    let query_result = query_as!(
                        db::Player,
                        r#"SELECT * FROM player ORDER BY best_score DESC LIMIT ?"#,
                        100,
                    )
                    .fetch_all(&self.db_pool)
                    .await;

                    let leaderboard_entry_list = match query_result {
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
                    let _ = self
                        .client_agent_command_sender
                        .send(command::Command::SendPacket { packet });
                }
                _ => {
                    warn!("unknown packet data: {:?}", data);
                }
            }
        }
    }

    async fn handle_command(
        &self,
        command: command::Command,
        client_writer: Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>>,
    ) {
        match command {
            command::Command::SendPacket { packet } => {
                let raw_data = packet.encode_to_vec();
                let mut client_writer = client_writer.lock().await;
                let _ = client_writer.send(Message::binary(raw_data)).await;
            }
            command::Command::SendRawData { raw_data } => {
                let mut client_writer = client_writer.lock().await;
                let _ = client_writer.send(Message::binary(raw_data)).await;
            }
            command::Command::UpdateSporeBatch { spore_batch } => {
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
            command::Command::SyncPlayerBestScore { current_score } => {
                let db_player_id = {
                    let mut db_player = self.db_player.write().await;
                    let db_player = match &mut *db_player {
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

                let query_result = query_as!(
                    db::Player,
                    r#"UPDATE player SET best_score = ? WHERE id = ?"#,
                    current_score,
                    db_player_id,
                )
                .execute(&self.db_pool)
                .await;

                if let Err(e) = query_result {
                    warn!("UPDATE player SET best_score error: {:?}", e);
                }
            }
            command::Command::DisconnectClinet => {
                warn!("Command::DisconnectClinet");
                let mut client_writer = client_writer.lock().await;
                let _ = client_writer.close().await;
            }
            _ => {
                warn!("unknown command: {:?}", command);
            }
        }
    }
}
