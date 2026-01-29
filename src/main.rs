use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::{debug, info, warn};

use contui::app::App;
use contui::config::Config;
use contui::core::ConnectionInfo;
use contui::docker::DockerClient;
use contui::update::{
    check_for_updates, install_update, is_interactive, prompt_for_update, save_skip_version,
    UpdateCheckResult, UpdateDecision, UpdateInfo,
};

/// Contui - Advanced Docker TUI
#[derive(Parser, Debug)]
#[command(name = "contui")]
#[command(about = "A powerful terminal UI for Docker container management")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Path to configuration file
    #[arg(short, long, value_name = "FILE", global = true)]
    config: Option<std::path::PathBuf>,

    /// Docker host to connect to
    #[arg(short = 'H', long, value_name = "HOST", global = true)]
    host: Option<String>,

    /// Enable debug logging to file
    #[arg(short, long, global = true)]
    debug: bool,

    /// Log level (error, warn, info, debug, trace)
    #[arg(long, value_name = "LEVEL", default_value = "info", global = true)]
    log_level: String,

    /// Skip automatic update check on startup
    #[arg(long, global = true)]
    skip_update_check: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Start the TUI (default)
    #[command(alias = "tui")]
    Run,

    /// Update contui to the latest version
    Update {
        /// Only check for updates, don't install
        #[arg(long)]
        check: bool,
    },

    /// Uninstall contui from your system
    Uninstall {
        /// Also remove all configuration files
        #[arg(long)]
        purge: bool,
    },

    /// Show version information
    Version,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Update { check }) => {
            if check {
                return check_for_updates_cli().await;
            } else {
                return update_self().await;
            }
        }
        Some(Commands::Uninstall { purge }) => {
            return uninstall_self(purge).await;
        }
        Some(Commands::Version) => {
            print_version();
            return Ok(());
        }
        _ => run_tui(cli).await,
    }
}

fn print_version() {
    println!("contui {}", env!("CARGO_PKG_VERSION"));
    println!(
        "Platform: {} {}",
        std::env::consts::OS,
        std::env::consts::ARCH
    );
    println!("Repository: https://github.com/aycandv/contui");
}

async fn check_for_updates_cli() -> Result<()> {
    println!("Checking for updates...");

    let current_version = env!("CARGO_PKG_VERSION");

    match get_latest_version().await {
        Ok(latest_version) => {
            if latest_version == current_version {
                println!("✓ You're on the latest version (v{})", current_version);
            } else {
                println!("Current version: v{}", current_version);
                println!("Latest version: v{}", latest_version);
                println!("\nUpdate available! Run 'contui update' to install.");
            }
        }
        Err(e) => {
            eprintln!("✗ Failed to check for updates: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}

async fn update_self() -> Result<()> {
    use self_update::backends::github::Update;
    use self_update::cargo_crate_version;

    println!("Checking for updates...");

    let current_version = cargo_crate_version!();

    // Determine target identifier matching our release asset naming
    let target = self_update::get_target();
    println!("Platform: {}", target);

    let status = Update::configure()
        .repo_owner("aycandv")
        .repo_name("contui")
        .bin_name("contui")
        .target(target)
        .identifier("contui")
        .show_download_progress(true)
        .show_output(false)
        .no_confirm(false)
        .current_version(current_version)
        .build()?
        .update()?;

    if status.updated() {
        println!("\n✓ Successfully updated to v{}", status.version());
        println!("  Previous version: v{}", current_version);
    } else {
        println!(
            "\n✓ You're already on the latest version (v{})",
            current_version
        );
    }

    Ok(())
}

async fn uninstall_self(purge: bool) -> Result<()> {
    let exe_path = std::env::current_exe()?;

    println!("This will remove contui from your system.");
    println!("Binary location: {}", exe_path.display());

    if purge {
        let config_dir = directories::ProjectDirs::from("com", "contui", "contui")
            .map(|d| d.config_dir().to_path_buf())
            .or_else(|| {
                std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".config/contui"))
            });

        if let Some(ref dir) = config_dir {
            println!("Configuration directory: {}", dir.display());
        }
    }

    println!();
    print!("Are you sure? [y/N] ");
    use std::io::Write;
    std::io::stdout().flush()?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    if !input.trim().eq_ignore_ascii_case("y") {
        println!("Uninstall cancelled.");
        return Ok(());
    }

    // Remove binary
    println!("\nRemoving {}...", exe_path.display());
    if let Err(e) = std::fs::remove_file(&exe_path) {
        eprintln!("✗ Failed to remove binary: {}", e);

        // On Unix, if the binary is running, we might need to use a different approach
        #[cfg(unix)]
        {
            eprintln!("  The binary may be in use. Try running:");
            eprintln!("  rm '{}'", exe_path.display());
        }

        return Err(e.into());
    }

    // Remove config if purge
    if purge {
        if let Some(config_dir) = directories::ProjectDirs::from("com", "contui", "contui")
            .map(|d| d.config_dir().to_path_buf())
            .or_else(|| {
                std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".config/contui"))
            })
        {
            if config_dir.exists() {
                println!("Removing {}...", config_dir.display());
                if let Err(e) = std::fs::remove_dir_all(&config_dir) {
                    eprintln!("✗ Failed to remove config directory: {}", e);
                    eprintln!("  You can remove it manually:");
                    eprintln!("  rm -rf '{}'", config_dir.display());
                }
            }
        }
    }

    println!("\n✓ Successfully uninstalled contui.");

    if !purge {
        if let Some(config_dir) = directories::ProjectDirs::from("com", "contui", "contui")
            .map(|d| d.config_dir().to_path_buf())
            .or_else(|| {
                std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".config/contui"))
            })
        {
            println!("\nTo also remove configuration files:");
            println!("  rm -rf '{}'", config_dir.display());
        }
    }

    Ok(())
}

async fn get_latest_version() -> Result<String> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.github.com/repos/aycandv/contui/releases/latest")
        .header("User-Agent", "contui-update-checker")
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!("GitHub API returned {}", response.status()));
    }

    let release: serde_json::Value = response.json().await?;
    let tag = release["tag_name"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Could not parse version from GitHub response"))?;

    // Remove 'v' prefix if present
    Ok(tag.trim_start_matches('v').to_string())
}

async fn run_tui(cli: Cli) -> Result<()> {
    // Initialize logging (file only, not stdout to avoid polluting TUI)
    let log_level = if cli.debug { "debug" } else { &cli.log_level };

    // Write logs to file instead of stdout
    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/contui.log")
        .ok();

    if let Some(file) = log_file {
        tracing_subscriber::fmt()
            .with_env_filter(format!("contui={}", log_level))
            .with_writer(std::sync::Arc::new(file))
            .init();
    } else {
        // If can't open log file, disable logging
        tracing_subscriber::fmt().with_env_filter("off").init();
    }

    info!("Starting Contui v{}", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let config = match &cli.config {
        Some(path) => Config::load(path)?,
        None => Config::load_default().unwrap_or_default(),
    };

    // Override config with CLI arguments
    let config = apply_cli_overrides(config, &cli);

    info!("Configuration loaded successfully");

    // Check for updates before launching TUI (unless skipped via CLI)
    if !cli.skip_update_check {
        if let Some(should_exit) = handle_update_check(&config).await {
            if should_exit {
                return Ok(());
            }
        }
    } else {
        debug!("Update check skipped via CLI flag");
    }

    // Check Docker connection
    match check_docker_connection(&config).await {
        Ok(info) => {
            info!(
                "Connected to Docker: {} (API: {})",
                info.version, info.api_version
            );
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

    info!("Contui shutting down gracefully");
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

/// Handle automatic update check before TUI launch
///
/// Returns `Some(true)` if the app should exit (after installing update),
/// `Some(false)` if the app should continue (user declined or error),
/// or `None` if the check was skipped entirely.
async fn handle_update_check(config: &Config) -> Option<bool> {
    let result = check_for_updates(&config.update).await;

    match result {
        UpdateCheckResult::UpdateAvailable {
            current,
            latest,
            release_url,
        } => {
            info!("Update available: {} -> {}", current, latest);

            // Only prompt if configured and in interactive mode
            if !config.update.prompt_to_install || !is_interactive() {
                // Just log, don't prompt
                debug!("Update prompt disabled or non-interactive mode");
                return Some(false);
            }

            let info = UpdateInfo {
                current_version: current,
                latest_version: latest.clone(),
                release_url,
            };

            match prompt_for_update(&info) {
                Ok(UpdateDecision::Install) => {
                    if let Err(e) = install_update() {
                        eprintln!("\n  ✗ Failed to install update: {}", e);
                        eprintln!("    You can try manually: contui update\n");
                        Some(false)
                    } else {
                        // Exit after successful update so user restarts with new version
                        Some(true)
                    }
                }
                Ok(UpdateDecision::Skip) => {
                    debug!("User chose to skip update");
                    Some(false)
                }
                Ok(UpdateDecision::SkipVersion) => {
                    debug!("User chose to skip version {}", latest);
                    if let Err(e) = save_skip_version(&latest) {
                        warn!("Could not save skip_version to config: {}", e);
                    }
                    Some(false)
                }
                Err(e) => {
                    warn!("Failed to read user input: {}", e);
                    Some(false)
                }
            }
        }
        UpdateCheckResult::UpToDate => {
            debug!("Already on latest version");
            None
        }
        UpdateCheckResult::Skipped { reason } => {
            debug!("Update check skipped: {}", reason);
            None
        }
        UpdateCheckResult::Failed { error } => {
            debug!("Update check failed (continuing anyway): {}", error);
            None
        }
    }
}
