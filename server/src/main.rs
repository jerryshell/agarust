#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let bind_addr = match std::env::args().nth(1) {
        Some(bind_addr) => bind_addr,
        None => "127.0.0.1:8080".to_string(),
    };
    tracing::info!("Bind addr: {:?}", bind_addr);

    let tcp_listener = match tokio::net::TcpListener::bind(&bind_addr).await {
        Ok(tcp_listener) => tcp_listener,
        Err(error) => {
            tracing::error!("Bind error: {:?}", error);
            return;
        }
    };
    tracing::info!("TCP listen: {:?}", tcp_listener);

    let mut hub = agarust_server::hub::Hub::new();
    let hub_command_sender = hub.command_sender.clone();
    let hub_task = tokio::spawn(async move { hub.run().await });

    while let Ok((tcp_stream, socket_addr)) = tcp_listener.accept().await {
        tracing::info!("Accept TCP stream: {:?}", socket_addr);
        let hub_command_sender = hub_command_sender.clone();
        tokio::spawn(async move {
            if let Err(error) =
                agarust_server::handle_tcp_stream(tcp_stream, socket_addr, hub_command_sender).await
            {
                tracing::error!("Handle TCP stream error: {:?}", error);
            }
        });
    }

    hub_task.abort();
}
