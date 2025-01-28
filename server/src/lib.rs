pub mod client_agent;
pub mod command;
pub mod db;
pub mod hub;
pub mod player;
pub mod proto;
pub mod proto_util;
pub mod spore;
pub mod util;

use anyhow::Result;
use nanoid::nanoid;
use std::{net::SocketAddr, sync::Arc};
use tokio::{net::TcpStream, sync::mpsc::UnboundedSender};
use tracing::info;

pub async fn handle_tcp_stream(
    tcp_stream: TcpStream,
    socket_addr: SocketAddr,
    db: db::Db,
    hub_command_sender: UnboundedSender<command::Command>,
) -> Result<()> {
    let ws_stream = tokio_tungstenite::accept_async(tcp_stream).await?;

    let client_agent =
        client_agent::ClientAgent::new(ws_stream, socket_addr, db, hub_command_sender).await?;

    client_agent.run().await;

    Ok(())
}
