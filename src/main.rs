use clap::Parser;
use log_watcher::cli::Args;
use log_watcher::config::Config;
use log_watcher::watcher::LogWatcher;
use std::process;
use tracing::{error, info};

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    // Build configuration from CLI args
    let config = match Config::from_args(&args) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Configuration error: {}", e);
            process::exit(2);
        }
    };

    // Create and run the log watcher
    let mut watcher = LogWatcher::new(config);

    match watcher.run().await {
        Ok(_) => {
            info!("LogWatcher completed successfully");
            process::exit(0);
        }
        Err(e) => {
            error!("LogWatcher failed: {}", e);
            process::exit(1);
        }
    }
}
