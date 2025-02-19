const DEFAULT_LOG_DIRECTORY: &str = "./";
const DEFAULT_LOG_FILE_NAME_PREFIX: &str = "agarust_server.log";
const DEFAULT_BIND_ADDR: &str = "127.0.0.1:8080";
const DEFAULT_DATABASE_URL: &str = "sqlite:agarust_db.sqlite";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    let _tracing_guard = init_tracing();

    let bind_addr = std::env::var("BIND_ADDR").unwrap_or(DEFAULT_BIND_ADDR.to_string());
    tracing::info!("bind_addr: {:?}", bind_addr);

    let tcp_listener = tokio::net::TcpListener::bind(&bind_addr).await?;

    let database_url = std::env::var("DATABASE_URL").unwrap_or(DEFAULT_DATABASE_URL.to_string());
    tracing::info!("database_url: {:?}", database_url);

    let db = agarust_server::db::Db::new(&database_url).await?;

    let hub = agarust_server::hub::Hub::new(db.clone());
    let hub_command_sender = hub.command_sender.clone();

    let hub_run_future = hub.run();
    tokio::spawn(hub_run_future);

    while let Ok((tcp_stream, socket_addr)) = tcp_listener.accept().await {
        tracing::info!("tcp_listener accept: {:?}", socket_addr);
        let tcp_stream_future = agarust_server::handle_tcp_stream(
            tcp_stream,
            socket_addr,
            db.clone(),
            hub_command_sender.clone(),
        );
        tokio::spawn(async move {
            let tcp_stream_result = tcp_stream_future.await;
            tracing::info!(
                "{:?} tcp_stream_result: {:?}",
                socket_addr,
                tcp_stream_result
            )
        });
    }

    Ok(())
}

fn init_tracing() -> tracing_appender::non_blocking::WorkerGuard {
    let log_directory = std::env::var("LOG_DIRECTORY").unwrap_or(DEFAULT_LOG_DIRECTORY.to_string());
    println!("log_directory: {:?}", log_directory);

    let log_file_name_prefix =
        std::env::var("LOG_FILE_NAME_PREFIX").unwrap_or(DEFAULT_LOG_FILE_NAME_PREFIX.to_string());
    println!("log_file_name_prefix: {:?}", log_file_name_prefix);

    let file_appender = tracing_appender::rolling::daily(log_directory, log_file_name_prefix);
    let (non_blocking_writer, worker_guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt()
        .compact()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(false)
        .with_writer(non_blocking_writer)
        .with_ansi(false)
        .init();

    worker_guard
}
