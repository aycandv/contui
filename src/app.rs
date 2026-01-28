//! Main application coordinator

use anyhow::Result;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::time::Duration;
use tracing::{debug, error, info, warn};

use crate::config::Config;
use crate::core::ConnectionInfo;
use crate::docker::DockerClient;
use crate::state::AppState;
use crate::ui::UiApp;

/// Main application struct
pub struct App {
    config: Config,
    state: AppState,
    docker_client: Option<DockerClient>,
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

        // Create UI app
        let mut ui_app = UiApp::new(self.state.clone());

        // Main event loop
        let result = self.run_event_loop(&mut terminal, &mut ui_app).await;

        // Cleanup terminal
        restore_terminal(&mut terminal)?;

        result
    }

    /// Run the event loop
    async fn run_event_loop(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        ui_app: &mut UiApp,
    ) -> Result<()> {
        let mut last_tick = std::time::Instant::now();
        let tick_rate = Duration::from_millis(250);

        loop {
            // Render the UI
            terminal.draw(|f| ui_app.draw(f))?;

            // Handle events with timeout
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if crossterm::event::poll(timeout)? {
                let event = crossterm::event::read()?;

                match event {
                    Event::Key(key) => {
                        // Handle key events
                        if key.kind == KeyEventKind::Press {
                            // Close help on any key if showing
                            if ui_app.state.show_help {
                                ui_app.state.show_help = false;
                                continue;
                            }
                        }
                        ui_app.handle_event(event);
                    }
                    _ => {
                        ui_app.handle_event(event);
                    }
                }
            }

            // Check if should quit
            if ui_app.should_quit {
                info!("Quit signal received, exiting event loop");
                break;
            }

            // Periodic tasks
            if last_tick.elapsed() >= tick_rate {
                self.on_tick().await;
                last_tick = std::time::Instant::now();
            }

            // Sync state back from UI app
            self.state = ui_app.state.clone();
        }

        Ok(())
    }

    /// Handle periodic tasks
    async fn on_tick(&mut self) {
        // Refresh data periodically
        // In future: refresh container lists, stats, etc.
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

            // TODO: Load images, volumes, networks (future stories)
        }
    }
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
    use super::*;

    // Note: Most tests would require async runtime and Docker

    #[test]
    fn test_terminal_setup_restore() {
        // This test would need to be integration test with proper terminal handling
        // For now just verify the functions exist
    }
}
