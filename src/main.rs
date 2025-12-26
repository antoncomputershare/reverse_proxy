mod config;
mod control;
mod proxy;
mod state;
mod tui;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser)]
#[command(name = "charles")]
#[command(about = "A terminal-controlled reverse proxy", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the proxy server
    Run {
        /// Path to configuration file
        #[arg(short, long, default_value = "config/charles.toml")]
        config: String,
    },
    /// Run the TUI interface
    Tui {
        /// Control API URL
        #[arg(short, long, default_value = "http://127.0.0.1:9000")]
        control: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Run { config } => {
            info!("Starting Charles proxy server");
            run_server(config).await?;
        }
        Commands::Tui { control } => {
            info!("Starting Charles TUI");
            run_tui(control).await?;
        }
    }

    Ok(())
}

async fn run_server(config_path: String) -> Result<()> {
    let config = config::Config::from_file(&config_path)?;
    info!("Loaded configuration from {}", config_path);

    let state = state::SharedState::new();

    // Start control server
    let control_server =
        control::ControlServer::new(config.control.listen.clone(), Arc::clone(&state));
    let control_handle = tokio::spawn(async move {
        if let Err(e) = control_server.run().await {
            tracing::error!("Control server error: {}", e);
        }
    });

    // Start proxy server
    let proxy_server = proxy::ProxyServer::new(config, Arc::clone(&state));
    let proxy_handle = tokio::spawn(async move {
        if let Err(e) = proxy_server.run().await {
            tracing::error!("Proxy server error: {}", e);
        }
    });

    // Wait for both servers
    tokio::try_join!(control_handle, proxy_handle)?;

    Ok(())
}

async fn run_tui(control_url: String) -> Result<()> {
    let mut app = tui::TuiApp::new(control_url);
    app.run().await?;
    Ok(())
}
