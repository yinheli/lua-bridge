use std::env;

use clap::Parser;
use tracing::info;

mod cli;
mod script;
mod server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "error,lua_bridge=debug");
    }

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false)
        .init();

    // Load .env file
    // This will load .env file from the current directory
    dotenv::dotenv().ok();

    let cli = cli::Cli::parse();

    info!("{}, ver: v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

    match cli.command {
        cli::Command::Serve(args) => server::serve(&args).await?,
    }

    Ok(())
}
