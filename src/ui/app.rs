//! UI Application logic

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;
use tracing::{debug, info};

use crate::core::{NotificationLevel, Tab};
use crate::state::AppState;
use crate::ui::components::ContainerListWidget;

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

        // If help is showing, any key closes it (except when toggling help)
        if self.state.show_help && key.code != KeyCode::Char('?') && key.code != KeyCode::Char('h') {
            self.state.show_help = false;
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

            // Tab switching with number keys
            KeyCode::Char('1') => self.switch_tab(Tab::Containers),
            KeyCode::Char('2') => self.switch_tab(Tab::Images),
            KeyCode::Char('3') => self.switch_tab(Tab::Volumes),
            KeyCode::Char('4') => self.switch_tab(Tab::Networks),
            KeyCode::Char('5') => self.switch_tab(Tab::Compose),
            KeyCode::Char('6') => self.switch_tab(Tab::System),

            // Tab switching with arrow keys (or container nav when on Containers tab)
            KeyCode::Right => self.next_tab(),
            KeyCode::Left => self.previous_tab(),
            KeyCode::Down => {
                if self.state.current_tab == Tab::Containers {
                    self.state.next_container();
                } else {
                    self.next_tab();
                }
            }
            KeyCode::Up => {
                if self.state.current_tab == Tab::Containers {
                    self.state.previous_container();
                } else {
                    self.previous_tab();
                }
            }

            // Navigation between panels
            KeyCode::Tab => self.next_panel(),
            KeyCode::BackTab => self.previous_panel(),

            // Container list navigation (when on Containers tab)
            KeyCode::Char('j') => {
                if self.state.current_tab == Tab::Containers {
                    self.state.next_container();
                }
            }
            KeyCode::Char('k') => {
                if self.state.current_tab == Tab::Containers {
                    self.state.previous_container();
                }
            }

            // Help
            KeyCode::Char('?') | KeyCode::Char('h') if key.modifiers.is_empty() => {
                self.state.show_help = !self.state.show_help;
            }

            _ => {
                debug!("Unhandled key: {:?}", key);
            }
        }
    }

    /// Switch to a specific tab
    fn switch_tab(&mut self, tab: Tab) {
        if self.state.current_tab != tab {
            info!("Switching to tab: {:?}", tab);
            self.state.previous_tab = Some(self.state.current_tab);
            self.state.current_tab = tab;
        }
    }

    /// Move to next tab (circular)
    fn next_tab(&mut self) {
        let tabs = Tab::all();
        let current_idx = tabs
            .iter()
            .position(|t| *t == self.state.current_tab)
            .unwrap_or(0);
        let next_idx = (current_idx + 1) % tabs.len();
        self.switch_tab(tabs[next_idx]);
    }

    /// Move to previous tab (circular)
    fn previous_tab(&mut self) {
        let tabs = Tab::all();
        let current_idx = tabs
            .iter()
            .position(|t| *t == self.state.current_tab)
            .unwrap_or(0);
        let prev_idx = if current_idx == 0 {
            tabs.len() - 1
        } else {
            current_idx - 1
        };
        self.switch_tab(tabs[prev_idx]);
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
                Constraint::Min(3),    // Main content (at least 3 lines)
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
        let status_indicator = if self.state.docker_connected {
            ("●", Color::Green)
        } else {
            ("○", Color::Red)
        };

        let header_spans = vec![
            Span::styled(" 🐳 DockMon ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("v{} ", env!("CARGO_PKG_VERSION")),
                Style::default().fg(Color::Gray),
            ),
            Span::raw("| "),
            Span::styled(
                self.state.current_tab.name(),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" | "),
            Span::styled(status_indicator.0, Style::default().fg(status_indicator.1)),
            Span::styled(
                if self.state.docker_connected { " Connected " } else { " Disconnected " },
                Style::default().fg(status_indicator.1),
            ),
        ];

        let header = Line::from(header_spans);
        frame.render_widget(
            Paragraph::new(header).style(Style::default().bg(Color::Black)),
            area,
        );
    }

    /// Render the main content area
    fn render_main_content(&self, frame: &mut Frame, area: Rect) {
        // Create a layout for sidebar + main panel
        // Use min 12 chars for sidebar, max 20
        let sidebar_width = (area.width / 5).clamp(12, 20);
        
        let content_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(sidebar_width), // Sidebar
                Constraint::Min(0),                // Main panel
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
            let shortcut = tab.shortcut();
            let name = tab.name();
            
            let style = if is_selected {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };

            // Format: "▶ 1:Containers" or "  2:Images"
            let line_text = if is_selected {
                format!("▶ {}:{}", shortcut, name)
            } else {
                format!("  {}:{}", shortcut, name)
            };

            lines.push(Line::from(Span::styled(line_text, style)));
        }

        let sidebar = Paragraph::new(Text::from(lines))
            .block(Block::default().borders(Borders::RIGHT).border_style(Color::DarkGray));

        frame.render_widget(sidebar, area);
    }

    /// Render the main panel based on current tab
    fn render_main_panel(&self, frame: &mut Frame, area: Rect) {
        match self.state.current_tab {
            Tab::Containers => self.render_containers_tab(frame, area),
            _ => self.render_simple_tab(frame, area),
        }
    }

    /// Render the containers tab with table
    fn render_containers_tab(&self, frame: &mut Frame, area: Rect) {
        // Create a local TableState that we'll use for rendering
        let mut table_state = ratatui::widgets::TableState::default();
        if !self.state.containers.is_empty() {
            table_state.select(Some(self.state.container_list_selected));
        }
        
        let widget = ContainerListWidget::new(self.state.containers.clone());
        let table = widget.build_table();
        frame.render_stateful_widget(table, area, &mut table_state);
    }

    /// Render simple tabs (Images, Volumes, etc.)
    fn render_simple_tab(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(self.state.current_tab.name())
            .borders(Borders::ALL)
            .border_style(Color::DarkGray);

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        let content = match self.state.current_tab {
            Tab::Images => format!("Total: {}", self.state.images.len()),
            Tab::Volumes => format!("Total: {}", self.state.volumes.len()),
            Tab::Networks => format!("Total: {}", self.state.networks.len()),
            Tab::Compose => "Docker Compose\n\nNot yet implemented".to_string(),
            Tab::System => {
                if self.state.docker_connected {
                    format!(
                        "Docker Version: {}\nAPI Version: {}\nOS: {}\nArch: {}",
                        self.state.connection_info.version,
                        self.state.connection_info.api_version,
                        self.state.connection_info.os,
                        self.state.connection_info.arch
                    )
                } else {
                    "Not connected to Docker\n\nPlease check your Docker daemon.".to_string()
                }
            }
            Tab::Containers => unreachable!(),
        };

        let paragraph = Paragraph::new(content)
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, inner_area);
    }

    /// Render the footer
    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        let help_text = if self.state.current_tab == Tab::Containers && !self.state.containers.is_empty() {
            " [←/→]:Tabs | [↑/↓ or j/k]:Select | [?]:Help | [q]:Quit "
        } else {
            " [←/→ or 1-6]:Switch Tabs | [?]:Help | [q]:Quit "
        };

        let footer = Paragraph::new(help_text)
            .style(Style::default().fg(Color::Gray).bg(Color::Black));

        frame.render_widget(footer, area);
    }

    /// Render help overlay
    fn render_help_overlay(&self, frame: &mut Frame, area: Rect) {
        // Create a centered popup (60% width, 70% height)
        let popup_area = Self::centered_rect(60, 70, area);

        // Clear the background
        frame.render_widget(Clear, popup_area);

        let help_text = r#"Keyboard Shortcuts

Navigation:
  ← / → or ↑ / ↓    Switch between tabs (circular)
  1 - 6             Jump directly to tab (Containers, Images, etc.)
  Tab               Move to next panel
  Shift+Tab         Move to previous panel

Containers Tab:
  ↑ / ↓ or j / k    Select container in list

Global:
  q                 Quit application
  Ctrl+C            Force quit
  ? or h            Toggle this help screen

Press any key to close this help...
"#;

        let help = Paragraph::new(help_text)
            .block(
                Block::default()
                    .title(" Help (Press any key to close) ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow)),
            )
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: true });

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
    fn test_tab_switching_numbers() {
        let state = AppState::default();
        let mut app = UiApp::new(state);

        assert_eq!(app.state.current_tab, Tab::Containers);

        app.handle_key_event(KeyEvent::from(KeyCode::Char('2')));
        assert_eq!(app.state.current_tab, Tab::Images);

        app.handle_key_event(KeyEvent::from(KeyCode::Char('1')));
        assert_eq!(app.state.current_tab, Tab::Containers);
    }

    #[test]
    fn test_tab_switching_arrows() {
        let state = AppState::default();
        let mut app = UiApp::new(state);

        assert_eq!(app.state.current_tab, Tab::Containers);

        // Right arrow should go to Images
        app.handle_key_event(KeyEvent::from(KeyCode::Right));
        assert_eq!(app.state.current_tab, Tab::Images);

        // Right arrow should go to Volumes
        app.handle_key_event(KeyEvent::from(KeyCode::Right));
        assert_eq!(app.state.current_tab, Tab::Volumes);

        // Left arrow should go back to Images
        app.handle_key_event(KeyEvent::from(KeyCode::Left));
        assert_eq!(app.state.current_tab, Tab::Images);

        // Left arrow should go back to Containers
        app.handle_key_event(KeyEvent::from(KeyCode::Left));
        assert_eq!(app.state.current_tab, Tab::Containers);

        // Left arrow should wrap to System (last tab)
        app.handle_key_event(KeyEvent::from(KeyCode::Left));
        assert_eq!(app.state.current_tab, Tab::System);
    }

    #[test]
    fn test_help_toggle() {
        let state = AppState::default();
        let mut app = UiApp::new(state);

        assert!(!app.state.show_help);

        app.handle_key_event(KeyEvent::from(KeyCode::Char('?')));
        assert!(app.state.show_help);

        app.handle_key_event(KeyEvent::from(KeyCode::Char('?')));
        assert!(!app.state.show_help);
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
