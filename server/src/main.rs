const DEFAULT_BIND_ADDR: &str = "127.0.0.1:8080";
const DEFAULT_DATABASE_URL: &str = "sqlite:agarust_db.sqlite";

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let bind_addr = match std::env::var("BIND_ADDR") {
        Ok(bind_addr) => bind_addr,
        Err(_) => DEFAULT_BIND_ADDR.to_string(),
    };
    tracing::info!("bind_addr: {:?}", bind_addr);

    let tcp_listener = match tokio::net::TcpListener::bind(&bind_addr).await {
        Ok(tcp_listener) => tcp_listener,
        Err(error) => {
            tracing::error!("TcpListener bind error: {:?}", error);
            return;
        }
    };

    let database_url = match std::env::var("DATABASE_URL") {
        Ok(database_url) => database_url,
        Err(_) => DEFAULT_DATABASE_URL.to_string(),
    };
    tracing::info!("database_url: {:?}", database_url);

    let db_pool = match sqlx::sqlite::SqlitePool::connect(&database_url).await {
        Ok(pool) => pool,
        Err(error) => {
            tracing::error!("SqlitePool connect error: {:?}", error);
            return;
        }
    };

    let mut hub = agarust_server::Hub::new(db_pool);
    let hub_command_sender = hub.command_sender.clone();
    let hub_task = tokio::spawn(async move { hub.run().await });

    while let Ok((tcp_stream, socket_addr)) = tcp_listener.accept().await {
        tracing::info!("tcp_listener accept: {:?}", socket_addr);
        let hub_command_sender = hub_command_sender.clone();
        tokio::spawn(async move {
            if let Err(error) =
                agarust_server::handle_tcp_stream(tcp_stream, socket_addr, hub_command_sender).await
            {
                tracing::error!("handle_tcp_stream error: {:?}", error);
            }
        });
    }

    hub_task.abort();
}
