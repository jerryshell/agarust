pub mod proto;

use futures_util::{
    stream::{SplitSink, SplitStream},
    StreamExt,
};
use prost::Message;
use std::{env, io::Cursor, net::SocketAddr};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::WebSocketStream;
use tracing::{info, warn};

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

    while let Ok((tcp_stream, _)) = listener.accept().await {
        tokio::spawn(async move {
            let (addr, ws_stream) = accept_connection(tcp_stream).await;
            let (client_writer, client_reader) = ws_stream.split();
            let mut client_agent = ClientAgent {
                addr,
                client_writer,
                client_reader,
            };
            client_agent.run().await;
        });
    }
}

async fn accept_connection(tcp_stream: TcpStream) -> (SocketAddr, WebSocketStream<TcpStream>) {
    let addr = tcp_stream
        .peer_addr()
        .expect("connected streams should have a peer address");
    info!("Peer address: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(tcp_stream)
        .await
        .expect("Error during the websocket handshake occurred");

    info!("New WebSocket connection: {}", addr);

    (addr, ws_stream)
}

struct ClientAgent {
    addr: SocketAddr,
    client_writer: SplitSink<WebSocketStream<TcpStream>, tungstenite::protocol::Message>,
    client_reader: SplitStream<WebSocketStream<TcpStream>>,
}

impl ClientAgent {
    async fn run(&mut self) {
        loop {
            match self.client_reader.next().await {
                Some(r) => match r {
                    Ok(message) => {
                        info!("message {:?}", message);
                        if let tungstenite::Message::Binary(bytes) = message {
                            match proto::Packet::decode(Cursor::new(bytes)) {
                                Ok(mut packet) => {
                                    packet.connection_id = self.addr.to_string();
                                    info!("packet {:?}", packet);
                                }
                                Err(error) => {
                                    warn!(
                                        "proto decode error from {:?}: {:?}, close connect",
                                        self.addr, error
                                    );
                                    continue;
                                }
                            }
                        }
                    }
                    Err(error) => {
                        warn!("error from {:?}: {:?}, close connect", self.addr, error);
                        break;
                    }
                },
                None => {
                    warn!("read None from {:?}, close connect", self.addr);
                    break;
                }
            }
        }
    }
}
