use std::net::SocketAddr;

use maxt_relay::{Config, app};

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("relay failed: {error}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env()?;
    let bind: SocketAddr = std::env::var("RELAY_BIND")
        .unwrap_or_else(|_| "0.0.0.0:8080".into())
        .parse()?;
    let listener = tokio::net::TcpListener::bind(bind).await?;
    eprintln!("relay listening on {bind}");
    axum::serve(listener, app(config)).await?;
    Ok(())
}
