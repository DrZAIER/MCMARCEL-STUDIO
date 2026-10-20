use std::net::SocketAddr;

use mcmarcel_server::{serve, ServerConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mcmarcel_server=info,tower_http=info".into()),
        )
        .init();

    let config = ServerConfig::from_env();
    if !config.ui_dir.join("index.html").exists() {
        tracing::warn!(
            "UI build not found at {}; API-only mode",
            config.ui_dir.display()
        );
    }
    let server = serve(config).await?;
    let address: SocketAddr = server.addr();
    tracing::info!("McMarcel Studio listening on http://{address}");
    server.wait().await;
    Ok(())
}
