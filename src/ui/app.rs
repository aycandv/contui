//! UI Application logic

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::layout::{Constraint, Direction, Layout, Margin, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;
use tracing::{debug, error, info, warn};

use crate::core::{NotificationLevel, Tab};
use crate::state::AppState;

/// UI Application controller
pub struct UiApp {
    pub state: AppState,
    pub should_quit: bool,
}

impl UiApp {
    /// Create a new UI app
    pub fn new(state: AppState) -> Self {
        Self {
            state,
            should_quit: false,
        }
    }

    /// Handle a terminal event
    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::Key(key_event) => self.handle_key_event(key_event),
            Event::Resize(width, height) => {
                debug!("Terminal resized to {}x{}", width, height);
                self.state.terminal_size = (width, height);
            }
            _ => {}
        }
    }

    /// Handle keyboard events
    fn handle_key_event(&mut self, key: KeyEvent) {
        // Only handle key press events (not release or repeat)
        if key.kind != KeyEventKind::Press {
            return;
        }

        // Global key handlers
        match key.code {
            // Quit
            KeyCode::Char('q') if key.modifiers.is_empty() => {
                info!("Quit key pressed");
                self.should_quit = true;
            }
            KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => {
                info!("Ctrl+C pressed");
                self.should_quit = true;
            }

            // Tab switching (1-6)
            KeyCode::Char('1') => self.switch_tab(Tab::Containers),
            KeyCode::Char('2') => self.switch_tab(Tab::Images),
            KeyCode::Char('3') => self.switch_tab(Tab::Volumes),
            KeyCode::Char('4') => self.switch_tab(Tab::Networks),
            KeyCode::Char('5') => self.switch_tab(Tab::Compose),
            KeyCode::Char('6') => self.switch_tab(Tab::System),

            // Navigation
            KeyCode::Tab => self.next_panel(),
            KeyCode::BackTab => self.previous_panel(),

            // Help
            KeyCode::Char('?') | KeyCode::Char('h') if key.modifiers.is_empty() => {
                self.state.show_help = !self.state.show_help;
            }

            _ => {
                debug!("Unhandled key: {:?}", key);
            }
        }
    }

    /// Switch to a different tab
    fn switch_tab(&mut self, tab: Tab) {
        info!("Switching to tab: {:?}", tab);
        self.state.previous_tab = Some(self.state.current_tab);
        self.state.current_tab = tab;
        self.state.add_notification(
            format!("Switched to {}", tab.name()),
            NotificationLevel::Info,
        );
    }

    /// Move focus to next panel
    fn next_panel(&mut self) {
        debug!("Moving to next panel");
        // Will be implemented with panel management
    }

    /// Move focus to previous panel
    fn previous_panel(&mut self) {
        debug!("Moving to previous panel");
        // Will be implemented with panel management
    }

    /// Render the UI
    pub fn draw(&self, frame: &mut Frame) {
        let area = frame.area();

        // Create the main layout
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Header
                Constraint::Min(0),    // Main content
                Constraint::Length(1), // Footer
            ])
            .split(area);

        // Render components
        self.render_header(frame, main_layout[0]);
        self.render_main_content(frame, main_layout[1]);
        self.render_footer(frame, main_layout[2]);

        // Render modal overlays if active
        if self.state.show_help {
            self.render_help_overlay(frame, area);
        }
    }

    /// Render the header
    fn render_header(&self, frame: &mut Frame, area: Rect) {
        let header_text = format!(
            " 🐳 DockMon {} | {} | {} ",
            env!("CARGO_PKG_VERSION"),
            self.state.current_tab.name(),
            if self.state.docker_connected {
                "● Connected"
            } else {
                "○ Disconnected"
            }
        );

        let header = Paragraph::new(header_text)
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));

        frame.render_widget(header, area);
    }

    /// Render the main content area
    fn render_main_content(&self, frame: &mut Frame, area: Rect) {
        // Create a layout for sidebar + main panel
        let content_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(15), // Sidebar
                Constraint::Min(0),     // Main panel
            ])
            .split(area);

        self.render_sidebar(frame, content_layout[0]);
        self.render_main_panel(frame, content_layout[1]);
    }

    /// Render the sidebar with tabs
    fn render_sidebar(&self, frame: &mut Frame, area: Rect) {
        let mut lines = vec![];

        for tab in Tab::all() {
            let is_selected = self.state.current_tab == *tab;
            let prefix = if is_selected { "▶ " } else { "  " };
            let style = if is_selected {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };

            lines.push(Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(tab.name(), style),
            ]));
        }

        let sidebar = Paragraph::new(Text::from(lines))
            .block(Block::default().borders(Borders::RIGHT));

        frame.render_widget(sidebar, area);
    }

    /// Render the main panel based on current tab
    fn render_main_panel(&self, frame: &mut Frame, area: Rect) {
        let content = match self.state.current_tab {
            Tab::Containers => {
                format!(
                    "Containers\n\nTotal: {}\nRunning: {}\nStopped: {}",
                    self.state.containers.len(),
                    self.state
                        .containers
                        .iter()
                        .filter(|c| c.state == crate::core::ContainerState::Running)
                        .count(),
                    self.state
                        .containers
                        .iter()
                        .filter(|c| c.state != crate::core::ContainerState::Running)
                        .count(),
                )
            }
            Tab::Images => format!("Images\n\nTotal: {}", self.state.images.len()),
            Tab::Volumes => format!("Volumes\n\nTotal: {}", self.state.volumes.len()),
            Tab::Networks => format!("Networks\n\nTotal: {}", self.state.networks.len()),
            Tab::Compose => "Docker Compose\n\nNot yet implemented".to_string(),
            Tab::System => format!(
                "System\n\nDocker: {}\nAPI: {}",
                self.state.connection_info.version,
                self.state.connection_info.api_version
            ),
        };

        let paragraph = Paragraph::new(content)
            .block(
                Block::default()
                    .title(self.state.current_tab.name())
                    .borders(Borders::ALL),
            )
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, area);
    }

    /// Render the footer
    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        let help_text = "Tab: Switch | ?: Help | q: Quit";

        let footer = Paragraph::new(help_text)
            .style(Style::default().fg(Color::Gray));

        frame.render_widget(footer, area);
    }

    /// Render help overlay
    fn render_help_overlay(&self, frame: &mut Frame, area: Rect) {
        // Create a centered popup
        let popup_area = Self::centered_rect(60, 70, area);

        // Clear the background
        frame.render_widget(Clear, popup_area);

        let help_text = r#"
Keyboard Shortcuts

Global:
  q, Ctrl+C    Quit application
  Tab          Next panel
  Shift+Tab    Previous panel
  ?            Toggle this help

Navigation:
  1            Containers tab
  2            Images tab
  3            Volumes tab
  4            Networks tab
  5            Compose tab
  6            System tab

Press any key to close...
"#;

        let help = Paragraph::new(help_text)
            .block(
                Block::default()
                    .title("Help")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow)),
            )
            .style(Style::default().fg(Color::White));

        frame.render_widget(help, popup_area);
    }

    /// Calculate centered rectangle for popups
    fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn test_ui_app_creation() {
        let state = AppState::default();
        let app = UiApp::new(state);
        assert!(!app.should_quit);
    }

    #[test]
    fn test_quit_key() {
        let state = AppState::default();
        let mut app = UiApp::new(state);

        app.handle_key_event(KeyEvent::from(KeyCode::Char('q')));
        assert!(app.should_quit);
    }

    #[test]
    fn test_ctrl_c() {
        let state = AppState::default();
        let mut app = UiApp::new(state);

        app.handle_key_event(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
        assert!(app.should_quit);
    }

    #[test]
    fn test_tab_switching() {
        let state = AppState::default();
        let mut app = UiApp::new(state);

        assert_eq!(app.state.current_tab, Tab::Containers);

        app.handle_key_event(KeyEvent::from(KeyCode::Char('2')));
        assert_eq!(app.state.current_tab, Tab::Images);

        app.handle_key_event(KeyEvent::from(KeyCode::Char('1')));
        assert_eq!(app.state.current_tab, Tab::Containers);
    }

    #[test]
    fn test_rendering() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        let state = AppState::default();
        let app = UiApp::new(state);

        terminal
            .draw(|f| {
                app.draw(f);
            })
            .unwrap();

        // Just verify it doesn't panic
    }
}
