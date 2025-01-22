pub mod client_agent;
pub mod command;
pub mod db;
pub mod hub;
pub mod player;
pub mod proto;
pub mod proto_util;
pub mod spore;
pub mod util;

use nanoid::nanoid;
use std::{net::SocketAddr, sync::Arc};
use tokio::{net::TcpStream, sync::mpsc::UnboundedSender};
use tracing::{error, info};

pub async fn handle_tcp_stream(
    tcp_stream: TcpStream,
    socket_addr: SocketAddr,
    db: db::Db,
    hub_command_sender: UnboundedSender<command::Command>,
) {
    let ws_stream = match tokio_tungstenite::accept_async(tcp_stream).await {
        Ok(ws_stream) => ws_stream,
        Err(e) => {
            error!("tokio_tungstenite accept_async error: {:?}", e);
            return;
        }
    };
    info!("tokio_tungstenite accept_async: {:?}", ws_stream);

    let client_agent =
        match client_agent::ClientAgent::new(socket_addr, db, hub_command_sender.clone()).await {
            Some(client_agent) => client_agent,
            None => {
                error!("ClientAgent::new() None, return");
                return;
            }
        };

    let connection_id = client_agent.connection_id.clone();

    client_agent.run(ws_stream).await;

    let _ = hub_command_sender.send(command::Command::UnregisterClientAgent { connection_id });
}
