use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("backend=info,tower_http=info")),
        )
        .init();
    let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:3000".to_owned());
    let db = match std::env::var("DATABASE_URL") {
        Ok(url) => backend::connect(&url).await?,
        Err(std::env::VarError::NotPresent) => {
            let home = std::env::var_os("HOME")
                .filter(|value| !value.is_empty())
                .ok_or_else(|| {
                    std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "HOME is required for the default database location",
                    )
                })?;
            backend::connect_default(std::path::Path::new(&home)).await?
        }
        Err(error) => return Err(error.into()),
    };
    backend::migrate(&db).await?;
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    tracing::info!(address = %listener.local_addr()?, "backend listening");
    axum::serve(listener, backend::app(db.clone()))
        .with_graceful_shutdown(shutdown())
        .await?;
    db.close().await;
    Ok(())
}

async fn shutdown() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("install Ctrl+C handler");
    };
    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("install termination handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! { _ = ctrl_c => {}, _ = terminate => {} }
}
