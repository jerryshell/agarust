pub mod client_agent;
pub mod command;
pub mod hub;
pub mod proto;

use client_agent::ClientAgent;
use command::Command;
use nanoid::nanoid;
use std::net::SocketAddr;
use tokio::{net::TcpStream, sync::mpsc::UnboundedSender};
use tracing::info;

pub async fn handle_tcp_stream(
    tcp_stream: TcpStream,
    socket_addr: SocketAddr,
    hub_command_sender: UnboundedSender<Command>,
) {
    let ws_stream = tokio_tungstenite::accept_async(tcp_stream)
        .await
        .expect("Error during the websocket handshake occurred");
    info!("Accept ws_stream: {:?}", socket_addr);

    let connection_id = nanoid!();

    let client_agent_task = {
        let connection_id = connection_id.clone();
        let hub_command_sender = hub_command_sender.clone();
        let client_agent = ClientAgent {
            socket_addr,
            connection_id,
            hub_command_sender,
        };
        tokio::spawn(async move { client_agent.run(ws_stream).await })
    };

    let _ = client_agent_task.await;

    let unregister_command = Command::UnregisterClient { connection_id };
    let _ = hub_command_sender.send(unregister_command);
}
