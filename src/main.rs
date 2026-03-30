mod config;
mod engines;
mod fetchers;
mod tools;
mod types;
mod utils;

use anyhow::Result;
use rmcp::{ServiceExt, transport::io::stdio};
use tracing_subscriber::{EnvFilter, fmt};

use config::CONFIG;
use tools::WebSearchHandler;

#[tokio::main]
async fn main() -> Result<()> {
    fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let mode = std::env::var("MODE")
        .unwrap_or_else(|_| "stdio".into())
        .to_lowercase();

    tracing::info!("ferris-search starting...");
    tracing::info!("Default engine: {}", CONFIG.default_search_engine);
    tracing::info!("Mode: {}", mode);

    if mode == "stdio" || mode == "both" {
        tracing::info!("Starting STDIO transport...");
        let service = WebSearchHandler::new()
            .serve(stdio())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to start stdio transport: {}", e))?;
        service.waiting().await?;
    }

    Ok(())
}
