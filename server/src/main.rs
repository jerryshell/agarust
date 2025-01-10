#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let bind_addr = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());

    let tcp_listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .expect("Failed to bind");
    tracing::info!("TCP Listening on: {}", bind_addr);

    let mut hub = agarust_server::hub::Hub::new();
    let hub_command_sender = hub.command_sender.clone();
    tokio::spawn(async move { hub.run().await });

    while let Ok((tcp_stream, socket_addr)) = tcp_listener.accept().await {
        tracing::info!("Accept tcp_stream: {:?}", socket_addr);
        let hub_command_sender = hub_command_sender.clone();
        tokio::spawn(async move {
            agarust_server::handle_tcp_stream(tcp_stream, socket_addr, hub_command_sender).await
        });
    }
}
