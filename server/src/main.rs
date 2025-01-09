pub mod proto;

use futures_util::{future, StreamExt, TryStreamExt};
use prost::Message;
use std::env;
use tokio::net::{TcpListener, TcpStream};
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let packet = proto::Packet {
        connection_id: "123".to_owned(),
        data: Some(proto::packet::Data::Hello(proto::Hello {
            connection_id: "123".to_owned(),
        })),
    };
    info!("{:?}", packet);

    let v = packet.encode_to_vec();
    info!("{:?}", v);

    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());

    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    info!("Listening on: {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(accept_connection(stream));
    }
}

async fn accept_connection(stream: TcpStream) {
    let addr = stream
        .peer_addr()
        .expect("connected streams should have a peer address");
    info!("Peer address: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");

    info!("New WebSocket connection: {}", addr);

    let (write, read) = ws_stream.split();
    read.try_filter(|msg| future::ready(msg.is_text() || msg.is_binary()))
        .forward(write)
        .await
        .expect("Failed to forward messages")
}
