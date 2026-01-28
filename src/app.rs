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

/// Main application struct
pub struct App {
    #[allow(dead_code)]
    config: Config,
    state: AppState,
    docker_client: Option<DockerClient>,
    /// Pending log fetch task (container_id, join_handle)
    pending_log_fetch: Option<(String, tokio::task::JoinHandle<DockMonResult<Vec<LogEntry>>>)>,
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
            pending_log_fetch: None,
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
                
                // Check if should quit before we move ui_app
                should_quit = ui_app.should_quit;
                
                // Handle the action FIRST (this may modify self.state.containers and self.state.notifications)
                self.handle_ui_action(action).await;
                
                // Then sync UI state changes (confirm_dialog, show_help, etc.) to self.state
                // We need to preserve: containers, notifications (from action handler)
                // We need to sync from UI: confirm_dialog, show_help, current_tab, log_view, etc.
                let mut ui_state = ui_app.state;
                
                // Preserve data that action handler may have updated
                ui_state.containers = self.state.containers.clone();
                ui_state.notifications = self.state.notifications.clone();
                ui_state.docker_connected = self.state.docker_connected;
                ui_state.connection_info = self.state.connection_info.clone();
                ui_state.images = self.state.images.clone();
                ui_state.volumes = self.state.volumes.clone();
                ui_state.networks = self.state.networks.clone();
                
                // Sync log_view: prefer App's state (which may have been updated by action handler)
                // If App has log_view, use it (logs were fetched)
                // If App's log_view is None but UI has it, user just closed it - keep None
                if self.state.log_view.is_some() {
                    ui_state.log_view = self.state.log_view.clone();
                } else if ui_state.log_view.is_some() {
                    // App doesn't have log_view but UI does - user just closed it
                    ui_state.log_view = None;
                }
                
                self.state = ui_state;

                // Check if should quit
                if should_quit {
                    info!("Quit signal received, exiting event loop");
                    break;
                }
            }

            // Periodic tasks (every 250ms)
            if last_tick.elapsed() >= tick_rate {
                // Check for completed log fetches (only if there's a pending one)
                if self.pending_log_fetch.is_some() {
                    self.check_pending_log_fetch().await;
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
                // Open log view if not already open
                if self.state.log_view.is_none() {
                    if let Some(container) = self.state.containers.get(self.state.container_list_selected) {
                        let name = container.names.first().cloned().unwrap_or_else(|| container.short_id.clone());
                        self.state.open_log_view(id.clone(), name);
                        self.state.add_notification("Loading logs...", NotificationLevel::Info);
                    }
                }
                // Start log streaming (non-blocking)
                self.start_log_streaming(id);
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

    /// Start fetching logs from a container (non-blocking, spawns background task)
    fn start_log_streaming(&mut self, container_id: String) {
        if let Some(client) = &self.docker_client {
            // Cancel any existing pending fetch
            if let Some((id, handle)) = self.pending_log_fetch.take() {
                handle.abort();
                info!("Cancelled pending log fetch for container '{}'", id);
            }
            
            info!("Starting log fetch for container '{}'", container_id);
            self.state.add_notification("Fetching logs...", NotificationLevel::Info);
            
            // Clone for the async task
            let client = client.clone();
            let container_id_clone = container_id.clone();
            
            // Spawn async task with overall timeout
            let handle = tokio::spawn(async move {
                tokio::time::timeout(
                    std::time::Duration::from_secs(5),
                    client.fetch_logs(&container_id_clone, 100)
                ).await.map_err(|_| crate::core::DockMonError::Other("Log fetch timeout".to_string()))?
            });
            
            self.pending_log_fetch = Some((container_id, handle));
        } else {
            self.state.add_notification("Not connected to Docker", NotificationLevel::Error);
        }
    }
    
    /// Check if pending log fetch has completed and update state
    async fn check_pending_log_fetch(&mut self) {
        if let Some((container_id, handle)) = self.pending_log_fetch.take() {
            if handle.is_finished() {
                info!("Log fetch task finished for container '{}'", container_id);
                // Task completed, get the result
                match handle.await {
                    Ok(Ok(entries)) => {
                        let count = entries.len();
                        info!("Processing {} log entries for container {}", count, container_id);
                        if count == 0 {
                            self.state.add_notification(
                                format!("No logs found for container {}", &container_id[..12.min(container_id.len())]), 
                                NotificationLevel::Warning
                            );
                        } else {
                            for entry in entries {
                                self.state.add_log_entry(entry);
                            }
                            self.state.add_notification(format!("Loaded {} log lines", count), NotificationLevel::Success);
                        }
                    }
                    Ok(Err(e)) => {
                        warn!("Failed to fetch logs for container {}: {}", container_id, e);
                        self.state.add_notification(format!("Failed to fetch logs: {}", e), NotificationLevel::Error);
                    }
                    Err(e) => {
                        warn!("Log fetch task failed for container {}: {}", container_id, e);
                        self.state.add_notification("Log fetch failed", NotificationLevel::Error);
                    }
                }
            } else {
                // Task still running, put it back
                self.pending_log_fetch = Some((container_id, handle));
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
