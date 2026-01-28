//! Main application coordinator

use anyhow::Result;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};

use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::time::Duration;
use tracing::{debug, error, info, warn};

use crate::config::Config;
use crate::core::{ConnectionInfo, NotificationLevel, Result as DockMonResult};
use crate::docker::{DockerClient, LogEntry};
use crate::state::AppState;
use crate::ui::{UiAction, UiApp};
use tokio::sync::mpsc;

/// Main application struct
pub struct App {
    #[allow(dead_code)]
    config: Config,
    state: AppState,
    docker_client: Option<DockerClient>,
    /// Channel receiver for log fetch results
    log_fetch_rx: Option<mpsc::Receiver<DockMonResult<Vec<LogEntry>>>>,
    /// Last time we auto-fetched logs (for follow mode)
    last_log_fetch: Option<std::time::Instant>,
}

impl App {
    /// Create a new application instance
    pub async fn new(config: Config) -> Result<Self> {
        info!("Creating new App instance");

        let mut state = AppState::new();

        // Try to connect to Docker
        let docker_client = match Self::connect_docker(&config).await {
            Ok((client, info)) => {
                state.set_docker_connected(true, info);
                Some(client)
            }
            Err(e) => {
                warn!("Could not connect to Docker: {}", e);
                state.set_docker_connected(false, ConnectionInfo::default());
                None
            }
        };

        Ok(Self {
            config,
            state,
            docker_client,
            log_fetch_rx: None,
            last_log_fetch: None,
        })
    }

    /// Connect to Docker
    async fn connect_docker(config: &Config) -> Result<(DockerClient, ConnectionInfo)> {
        let client = if let Some(host) = &config.docker.host {
            DockerClient::with_host(host).await?
        } else {
            DockerClient::from_env().await?
        };

        let info = client.connection_info().clone();
        Ok((client, info))
    }

    /// Run the main application loop
    pub async fn run(&mut self) -> Result<()> {
        info!("Starting main application loop");

        // Setup terminal
        let mut terminal = setup_terminal()?;

        // Initial data load
        self.refresh_data().await;

        // Main event loop
        let result = self.run_event_loop(&mut terminal).await;

        // Cleanup terminal
        restore_terminal(&mut terminal)?;

        result
    }

    /// Run the event loop
    async fn run_event_loop(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> Result<()> {
        let mut last_tick = std::time::Instant::now();
        let mut last_data_refresh = std::time::Instant::now();
        let tick_rate = Duration::from_millis(250);
        let data_refresh_rate = Duration::from_secs(2); // Refresh data every 2 seconds
        let mut should_quit;

        loop {
            // Render the UI
            let ui_app = UiApp::new(self.state.clone());
            terminal.draw(|f| ui_app.draw(f))?;

            // Handle events with timeout
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if crossterm::event::poll(timeout)? {
                let event = crossterm::event::read()?;

                // Create a mutable UI app for handling events
                let mut ui_app = UiApp::new(self.state.clone());
                let action = ui_app.handle_event(event);

                // Apply UI-driven state changes first (navigation, dialogs, log view close/scroll, etc.)
                // and then execute the requested action.
                should_quit = ui_app.should_quit;
                self.state = ui_app.state;
                self.handle_ui_action(action).await;

                // Check if should quit
                if should_quit {
                    info!("Quit signal received, exiting event loop");
                    break;
                }
            }

            // Periodic tasks (every 250ms)
            if last_tick.elapsed() >= tick_rate {
                // Check for completed log fetches
                self.check_log_fetch().await;
                
                // Auto-refresh logs when in follow mode (every 2 seconds)
                if let Some(ref log_view) = self.state.log_view {
                    if log_view.follow && self.log_fetch_rx.is_none() {
                        let should_fetch = self.last_log_fetch
                            .map(|t| t.elapsed() >= Duration::from_secs(2))
                            .unwrap_or(true);
                        if should_fetch {
                            let id = log_view.container_id.clone();
                            self.start_log_streaming(id);
                            self.last_log_fetch = Some(std::time::Instant::now());
                        }
                    }
                }
                
                // Refresh data periodically (every 2 seconds)
                if last_data_refresh.elapsed() >= data_refresh_rate {
                    self.refresh_data().await;
                    last_data_refresh = std::time::Instant::now();
                }
                // Clear old notifications (older than 3 seconds)
                self.state.clear_old_notifications(3);
                last_tick = std::time::Instant::now();
            }
        }

        Ok(())
    }

    /// Handle UI action
    async fn handle_ui_action(&mut self, action: UiAction) {
        match action {
            UiAction::None => {}
            UiAction::Quit => {
                // Handled in event loop
            }
            UiAction::StartContainer(id) => {
                self.start_container(&id).await;
            }
            UiAction::StopContainer(id) => {
                self.stop_container(&id).await;
            }
            UiAction::RestartContainer(id) => {
                self.restart_container(&id).await;
            }
            UiAction::PauseContainer(id) => {
                self.pause_container(&id).await;
            }
            UiAction::UnpauseContainer(id) => {
                self.unpause_container(&id).await;
            }
            UiAction::KillContainer(id) => {
                self.kill_container(&id).await;
            }
            UiAction::RemoveContainer(id) => {
                self.remove_container(&id).await;
            }
            UiAction::ShowContainerLogs(id) => {
                let name = self
                    .state
                    .containers
                    .iter()
                    .find(|c| c.id == id)
                    .and_then(|c| c.names.first())
                    .cloned()
                    .unwrap_or_else(|| id.chars().take(12).collect::<String>());

                let needs_open = self
                    .state
                    .log_view
                    .as_ref()
                    .map(|lv| lv.container_id != id)
                    .unwrap_or(true);

                if needs_open {
                    self.state.open_log_view(id.clone(), name);
                } else if let Some(lv) = &mut self.state.log_view {
                    lv.logs.clear();
                    lv.scroll_offset = 0;
                    lv.follow = true;
                }

                self.state.add_notification("Loading logs...", NotificationLevel::Info);
                self.start_log_streaming(id);
                self.last_log_fetch = Some(std::time::Instant::now());
            }
            UiAction::RemoveImage(id) => {
                self.remove_image(&id).await;
            }
            UiAction::PruneImages => {
                self.prune_images().await;
            }
            UiAction::InspectImage(_id) => {
                // TODO: Implement image inspect view in future story
                info!("Image inspect not yet implemented");
                self.state.add_notification("Image inspect not yet implemented", NotificationLevel::Warning);
            }
            UiAction::RemoveVolume(name) => {
                self.remove_volume(&name).await;
            }
            UiAction::PruneVolumes => {
                self.prune_volumes().await;
            }
            UiAction::RemoveNetwork(id) => {
                self.remove_network(&id).await;
            }
            UiAction::PruneNetworks => {
                self.prune_networks().await;
            }
        }
    }

    /// Start a container
    async fn start_container(&mut self, id: &str) {
        if let Some(client) = &self.docker_client {
            info!("Starting container {}", id);
            match client.start_container(id).await {
                Ok(_) => {
                    info!("Container {} started", id);
                    self.state.add_notification("Container started", NotificationLevel::Success);
                    self.refresh_data().await;
                }
                Err(e) => {
                    error!("Failed to start container {}: {}", id, e);
                    self.state.add_notification(format!("Failed to start: {}", e), NotificationLevel::Error);
                }
            }
        }
    }

    /// Stop a container
    async fn stop_container(&mut self, id: &str) {
        if let Some(client) = &self.docker_client {
            info!("Stopping container {}", id);
            match client.stop_container(id, Some(10)).await {
                Ok(_) => {
                    info!("Container {} stopped", id);
                    self.state.add_notification("Container stopped", NotificationLevel::Success);
                    self.refresh_data().await;
                }
                Err(e) => {
                    error!("Failed to stop container {}: {}", id, e);
                    self.state.add_notification(format!("Failed to stop: {}", e), NotificationLevel::Error);
                }
            }
        }
    }

    /// Restart a container
    async fn restart_container(&mut self, id: &str) {
        if let Some(client) = &self.docker_client {
            info!("Restarting container {}", id);
            match client.restart_container(id, Some(10)).await {
                Ok(_) => {
                    info!("Container {} restarted", id);
                    self.state.add_notification("Container restarted", NotificationLevel::Success);
                    self.refresh_data().await;
                }
                Err(e) => {
                    error!("Failed to restart container {}: {}", id, e);
                    self.state.add_notification(format!("Failed to restart: {}", e), NotificationLevel::Error);
                }
            }
        }
    }

    /// Pause a container
    async fn pause_container(&mut self, id: &str) {
        if let Some(client) = &self.docker_client {
            info!("Pausing container {}", id);
            match client.pause_container(id).await {
                Ok(_) => {
                    info!("Container {} paused", id);
                    self.state.add_notification("Container paused", NotificationLevel::Success);
                    self.refresh_data().await;
                }
                Err(e) => {
                    error!("Failed to pause container {}: {}", id, e);
                    self.state.add_notification(format!("Failed to pause: {}", e), NotificationLevel::Error);
                }
            }
        }
    }

    /// Unpause a container
    async fn unpause_container(&mut self, id: &str) {
        if let Some(client) = &self.docker_client {
            info!("Unpausing container {}", id);
            match client.unpause_container(id).await {
                Ok(_) => {
                    info!("Container {} unpaused", id);
                    self.state.add_notification("Container unpaused", NotificationLevel::Success);
                    self.refresh_data().await;
                }
                Err(e) => {
                    error!("Failed to unpause container {}: {}", id, e);
                    self.state.add_notification(format!("Failed to unpause: {}", e), NotificationLevel::Error);
                }
            }
        }
    }

    /// Kill a container
    async fn kill_container(&mut self, id: &str) {
        if let Some(client) = &self.docker_client {
            info!("Killing container {}", id);
            match client.kill_container(id, None).await {
                Ok(_) => {
                    info!("Container {} killed", id);
                    self.state.add_notification("Container killed", NotificationLevel::Success);
                    self.refresh_data().await;
                }
                Err(e) => {
                    error!("Failed to kill container {}: {}", id, e);
                    self.state.add_notification(format!("Failed to kill: {}", e), NotificationLevel::Error);
                }
            }
        }
    }

    /// Remove a container
    async fn remove_container(&mut self, id: &str) {
        if let Some(client) = &self.docker_client {
            info!("Removing container {}", id);
            match client.remove_container(id, false, false).await {
                Ok(_) => {
                    info!("Container {} removed", id);
                    self.state.add_notification("Container removed", NotificationLevel::Success);
                    self.refresh_data().await;
                }
                Err(e) => {
                    error!("Failed to remove container {}: {}", id, e);
                    self.state.add_notification(format!("Failed to remove: {}", e), NotificationLevel::Error);
                }
            }
        }
    }

    /// Refresh all data from Docker
    async fn refresh_data(&mut self) {
        if let Some(client) = &self.docker_client {
            debug!("Refreshing data from Docker");

            // Fetch containers
            match client.list_containers(true).await {
                Ok(containers) => {
                    self.state.update_containers(containers);
                    debug!("Loaded {} containers", self.state.containers.len());
                }
                Err(e) => {
                    warn!("Failed to load containers: {}", e);
                }
            }

            // Fetch images
            match client.list_images(true).await {
                Ok(images) => {
                    self.state.update_images(images);
                    debug!("Loaded {} images", self.state.images.len());
                }
                Err(e) => {
                    warn!("Failed to load images: {}", e);
                }
            }

            // Fetch volumes
            match client.list_volumes().await {
                Ok(volumes) => {
                    self.state.update_volumes(volumes);
                    debug!("Loaded {} volumes", self.state.volumes.len());
                }
                Err(e) => {
                    warn!("Failed to load volumes: {}", e);
                }
            }

            // Fetch networks
            match client.list_networks().await {
                Ok(networks) => {
                    self.state.update_networks(networks);
                    debug!("Loaded {} networks", self.state.networks.len());
                }
                Err(e) => {
                    warn!("Failed to load networks: {}", e);
                }
            }
        }
    }

    /// Remove an image
    async fn remove_image(&mut self, id: &str) {
        if let Some(client) = &self.docker_client {
            info!("Removing image {}", id);
            match client.remove_image(id, false).await {
                Ok(_) => {
                    info!("Image {} removed", id);
                    self.state.add_notification("Image removed", NotificationLevel::Success);
                    self.refresh_data().await;
                }
                Err(e) => {
                    error!("Failed to remove image {}: {}", id, e);
                    self.state.add_notification(format!("Failed to remove image: {}", e), NotificationLevel::Error);
                }
            }
        }
    }

    /// Prune dangling images
    async fn prune_images(&mut self) {
        if let Some(client) = &self.docker_client {
            info!("Pruning dangling images");
            match client.prune_images().await {
                Ok(reclaimed) => {
                    let size_str = format_size(reclaimed);
                    info!("Pruned images, reclaimed {}", size_str);
                    self.state.add_notification(format!("Pruned images, reclaimed {}", size_str), NotificationLevel::Success);
                    self.refresh_data().await;
                }
                Err(e) => {
                    error!("Failed to prune images: {}", e);
                    self.state.add_notification(format!("Failed to prune images: {}", e), NotificationLevel::Error);
                }
            }
        }
    }

    /// Remove a volume
    async fn remove_volume(&mut self, name: &str) {
        if let Some(client) = &self.docker_client {
            info!("Removing volume {}", name);
            match client.remove_volume(name, false).await {
                Ok(_) => {
                    info!("Volume {} removed", name);
                    self.state.add_notification("Volume removed", NotificationLevel::Success);
                    self.refresh_data().await;
                }
                Err(e) => {
                    error!("Failed to remove volume {}: {}", name, e);
                    self.state.add_notification(format!("Failed to remove volume: {}", e), NotificationLevel::Error);
                }
            }
        }
    }

    /// Prune unused volumes
    async fn prune_volumes(&mut self) {
        if let Some(client) = &self.docker_client {
            info!("Pruning volumes");
            match client.prune_volumes().await {
                Ok(reclaimed) => {
                    let size_str = format_size(reclaimed);
                    info!("Pruned volumes, reclaimed {}", size_str);
                    self.state.add_notification(format!("Pruned volumes, reclaimed {}", size_str), NotificationLevel::Success);
                    self.refresh_data().await;
                }
                Err(e) => {
                    error!("Failed to prune volumes: {}", e);
                    self.state.add_notification(format!("Failed to prune volumes: {}", e), NotificationLevel::Error);
                }
            }
        }
    }

    /// Remove a network
    async fn remove_network(&mut self, id: &str) {
        if let Some(client) = &self.docker_client {
            info!("Removing network {}", id);
            match client.remove_network(id).await {
                Ok(_) => {
                    info!("Network {} removed", id);
                    self.state.add_notification("Network removed", NotificationLevel::Success);
                    self.refresh_data().await;
                }
                Err(e) => {
                    error!("Failed to remove network {}: {}", id, e);
                    self.state.add_notification(format!("Failed to remove network: {}", e), NotificationLevel::Error);
                }
            }
        }
    }

    /// Prune unused networks
    async fn prune_networks(&mut self) {
        if let Some(client) = &self.docker_client {
            info!("Pruning networks");
            match client.prune_networks().await {
                Ok(count) => {
                    info!("Pruned {} networks", count);
                    self.state.add_notification(format!("Pruned {} networks", count), NotificationLevel::Success);
                    self.refresh_data().await;
                }
                Err(e) => {
                    error!("Failed to prune networks: {}", e);
                    self.state.add_notification(format!("Failed to prune networks: {}", e), NotificationLevel::Error);
                }
            }
        }
    }

    /// Start fetching logs from a container (non-blocking, uses channel)
    fn start_log_streaming(&mut self, container_id: String) {
        if let Some(client) = &self.docker_client {
            // Cancel any existing pending fetch by dropping the receiver
            if self.log_fetch_rx.is_some() {
                info!("Cancelling previous log fetch");
                self.log_fetch_rx = None;
            }
            
            info!("Starting log fetch for container '{}'", container_id);
            self.state.add_notification("Fetching logs...", NotificationLevel::Info);
            
            // Create channel for results
            let (tx, rx) = mpsc::channel(1);
            self.log_fetch_rx = Some(rx);
            
            // Clone client for the thread
            let client = client.clone();
            
            // Spawn a completely separate OS thread with its own runtime
            // This is the only way to ensure bollard's blocking operations don't freeze our UI
            std::thread::spawn(move || {
                // Create a new runtime just for this thread
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("Failed to create runtime");
                
                rt.block_on(async move {
                    // fetch_logs has per-item timeouts
                    let result = client.fetch_logs(&container_id, 100).await;
                    
                    // Send result back via channel (ignore send errors if receiver dropped)
                    let _ = tx.send(result).await;
                });
            });
        } else {
            self.state.add_notification("Not connected to Docker", NotificationLevel::Error);
        }
    }
    
    /// Check if log fetch has completed and update state
    async fn check_log_fetch(&mut self) {
        if let Some(rx) = &mut self.log_fetch_rx {
            // Try to receive with a very short timeout (non-blocking check)
            match tokio::time::timeout(Duration::from_millis(1), rx.recv()).await {
                Ok(Some(Ok(entries))) => {
                    self.log_fetch_rx = None; // Clear the receiver
                    let count = entries.len();
                    info!("Fetched {} log entries", count);
                    if count > 0 {
                        for entry in entries {
                            self.state.add_log_entry(entry);
                        }
                        // Don't show notification for auto-refreshes in follow mode
                        // (only manual 'r' press shows notification)
                    }
                }
                Ok(Some(Err(e))) => {
                    self.log_fetch_rx = None;
                    warn!("Failed to fetch logs: {}", e);
                    self.state.add_notification(format!("Failed to fetch logs: {}", e), NotificationLevel::Error);
                }
                Ok(None) => {
                    // Channel closed
                    self.log_fetch_rx = None;
                }
                Err(_) => {
                    // Timeout - still waiting, that's fine
                }
            }
        }
    }
}

/// Format size in human readable format
fn format_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    if size == 0 {
        return "0 B".to_string();
    }
    let size = size as f64;
    let exp = (size.ln() / 1024_f64.ln()).min(UNITS.len() as f64 - 1.0) as usize;
    let size = size / 1024_f64.powi(exp as i32);
    format!("{:.1} {}", size, UNITS[exp])
}

/// Setup the terminal for TUI
fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    info!("Setting up terminal");

    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;

    info!("Terminal setup complete");
    Ok(terminal)
}

/// Restore terminal to original state
fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    info!("Restoring terminal");

    terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    info!("Terminal restored");
    Ok(())
}

#[cfg(test)]
mod tests {
    // Note: Most tests would require async runtime and Docker
}
