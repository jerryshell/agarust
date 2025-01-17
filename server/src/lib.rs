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
    db_pool: sqlx::Pool<sqlx::Sqlite>,
    hub_command_sender: UnboundedSender<command::Command>,
) {
    let ws_stream = match tokio_tungstenite::accept_async(tcp_stream).await {
        Ok(ws_stream) => ws_stream,
        Err(e) => {
            error!("tokio_tungstenite accept_async error {:?}", e);
            return;
        }
    };
    info!("Accept WebSocket stream: {:?}", ws_stream);

    let connection_id: Arc<str> = nanoid!().into();

    let (client_agent, client_agent_command_receiver) = client_agent::ClientAgent::new(
        socket_addr,
        connection_id.clone(),
        db_pool,
        hub_command_sender.clone(),
    );

    let client_agent = Arc::new(client_agent);

    let client_agent_register_rsult =
        hub_command_sender.send(command::Command::RegisterClientAgent {
            client_agent: client_agent.clone(),
        });
    if let Err(error) = client_agent_register_rsult {
        error!("client_agent_register_rsult error: {:?}", error);
        return;
    }

    client_agent::run(client_agent, client_agent_command_receiver, ws_stream).await;

    let unregister_command = command::Command::UnregisterClientAgent { connection_id };
    let _ = hub_command_sender.send(unregister_command);
}
