mod proxy;
mod tui;
mod types;

use anyhow::Result;
use clap::Parser;
use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber;

#[derive(Parser, Debug)]
#[command(name = "charles")]
#[command(about = "A terminal-controlled reverse proxy with TUI", long_about = None)]
struct Args {
    /// Port to listen on
    #[arg(short, long, default_value = "8080")]
    port: u16,

    /// Target host to forward requests to
    #[arg(short, long, default_value = "localhost:3000")]
    target: String,

    /// Disable TUI and run in CLI mode
    #[arg(long)]
    no_tui: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .init();

    let args = Args::parse();

    info!("Charles Reverse Proxy starting...");
    info!("Listening on port: {}", args.port);
    info!("Forwarding to: {}", args.target);

    // Create shared state
    let state = Arc::new(types::ProxyState::new());

    // Start proxy server in background
    let proxy_state = state.clone();
    let proxy_handle = tokio::spawn(async move {
        proxy::start_proxy(args.port, args.target.clone(), proxy_state)
            .await
            .expect("Failed to start proxy server");
    });

    // Run TUI or wait for proxy
    if args.no_tui {
        info!("Running in CLI mode (no TUI)");
        proxy_handle.await?;
    } else {
        // Start TUI
        let tui_result = tui::run_tui(state.clone()).await;
        
        // TUI exited, shutdown proxy
        proxy_handle.abort();
        
        tui_result?;
    }

    Ok(())
}
