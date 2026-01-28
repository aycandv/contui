use anyhow::Result;
use clap::Parser;
use tracing::{info, warn};

use dockmon::app::App;
use dockmon::config::Config;
use dockmon::core::ConnectionInfo;
use dockmon::docker::DockerClient;

/// DockMon - Advanced Docker TUI
#[derive(Parser, Debug)]
#[command(name = "dockmon")]
#[command(about = "A terminal UI for Docker management")]
#[command(version)]
struct Cli {
    /// Path to configuration file
    #[arg(short, long, value_name = "FILE")]
    config: Option<std::path::PathBuf>,

    /// Docker host to connect to
    #[arg(short = 'H', long, value_name = "HOST")]
    host: Option<String>,

    /// Enable debug logging to file
    #[arg(short, long)]
    debug: bool,

    /// Log level (error, warn, info, debug, trace)
    #[arg(long, value_name = "LEVEL", default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging (file only, not stdout to avoid polluting TUI)
    let log_level = if cli.debug {
        "debug"
    } else {
        &cli.log_level
    };
    
    // Write logs to file instead of stdout
    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/dockmon.log")
        .ok();
    
    if let Some(file) = log_file {
        tracing_subscriber::fmt()
            .with_env_filter(format!("dockmon={}", log_level))
            .with_writer(std::sync::Arc::new(file))
            .init();
    } else {
        // If can't open log file, disable logging
        tracing_subscriber::fmt()
            .with_env_filter("off")
            .init();
    }

    info!("Starting DockMon v{}", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let config = match &cli.config {
        Some(path) => Config::load(path)?,
        None => Config::load_default().unwrap_or_default(),
    };

    // Override config with CLI arguments
    let config = apply_cli_overrides(config, &cli);

    info!("Configuration loaded successfully");

    // Check Docker connection
    match check_docker_connection(&config).await {
        Ok(info) => {
            info!("Connected to Docker: {} (API: {})", info.version, info.api_version);
        }
        Err(e) => {
            warn!("Could not connect to Docker: {}", e);
            eprintln!("⚠️  Warning: Could not connect to Docker daemon.");
            eprintln!("   Please ensure Docker is running and you have permissions.");
            eprintln!("   Error: {}", e);
        }
    }

    // Run the TUI application
    let mut app = App::new(config).await?;
    app.run().await?;

    info!("DockMon shutting down gracefully");
    Ok(())
}

fn apply_cli_overrides(mut config: Config, cli: &Cli) -> Config {
    if let Some(host) = &cli.host {
        config.docker.host = Some(host.clone());
    }
    config
}

async fn check_docker_connection(config: &Config) -> anyhow::Result<ConnectionInfo> {
    let client = if let Some(host) = &config.docker.host {
        DockerClient::with_host(host).await?
    } else {
        DockerClient::from_env().await?
    };
    
    client.ping().await?;
    Ok(client.connection_info().clone())
}
