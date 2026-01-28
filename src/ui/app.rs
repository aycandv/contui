//! UI Application logic

use std::borrow::Cow;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;
use tracing::{debug, info};

use crate::core::{ConfirmAction, ContainerState, Tab, UiAction};
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

    /// Handle a terminal event and return any action to execute
    pub fn handle_event(&mut self, event: Event) -> UiAction {
        match event {
            Event::Key(key_event) => self.handle_key_event(key_event),
            Event::Resize(width, height) => {
                debug!("Terminal resized to {}x{}", width, height);
                self.state.terminal_size = (width, height);
                UiAction::None
            }
            _ => UiAction::None,
        }
    }

    /// Handle keyboard events and return any action to execute
    fn handle_key_event(&mut self, key: KeyEvent) -> UiAction {
        // Only handle key press events (not release or repeat)
        if key.kind != KeyEventKind::Press {
            return UiAction::None;
        }

        // If confirmation dialog is showing
        if self.state.confirm_dialog.is_some() {
            return self.handle_confirmation_key(key);
        }

        // If help is showing, any key closes it (except when toggling help)
        if self.state.show_help && key.code != KeyCode::Char('?') && key.code != KeyCode::Char('h') {
            self.state.show_help = false;
            return UiAction::None;
        }

        // Global key handlers
        match key.code {
            // Quit
            KeyCode::Char('q') if key.modifiers.is_empty() => {
                info!("Quit key pressed");
                self.should_quit = true;
                UiAction::Quit
            }
            KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => {
                info!("Ctrl+C pressed");
                self.should_quit = true;
                UiAction::Quit
            }

            // Tab switching with number keys
            KeyCode::Char('1') => { self.switch_tab(Tab::Containers); UiAction::None }
            KeyCode::Char('2') => { self.switch_tab(Tab::Images); UiAction::None }
            KeyCode::Char('3') => { self.switch_tab(Tab::Volumes); UiAction::None }
            KeyCode::Char('4') => { self.switch_tab(Tab::Networks); UiAction::None }
            KeyCode::Char('5') => { self.switch_tab(Tab::Compose); UiAction::None }
            KeyCode::Char('6') => { self.switch_tab(Tab::System); UiAction::None }

            // Tab switching with arrow keys
            KeyCode::Right => { self.next_tab(); UiAction::None }
            KeyCode::Left => { self.previous_tab(); UiAction::None }

            // Navigation between panels
            KeyCode::Tab => { self.next_panel(); UiAction::None }
            KeyCode::BackTab => { self.previous_panel(); UiAction::None }

            // Container actions (when on Containers tab) - must come before unguarded 'k'
            KeyCode::Char('s') if self.state.current_tab == Tab::Containers => {
                self.handle_start_stop_action()
            }
            KeyCode::Char('r') if self.state.current_tab == Tab::Containers => {
                self.handle_restart_action()
            }
            KeyCode::Char('p') if self.state.current_tab == Tab::Containers => {
                self.handle_pause_action()
            }
            KeyCode::Char('k') if self.state.current_tab == Tab::Containers && key.modifiers.is_empty() => {
                self.handle_kill_action()
            }
            KeyCode::Char('d') if self.state.current_tab == Tab::Containers => {
                self.handle_remove_action()
            }
            KeyCode::Char('l') if self.state.current_tab == Tab::Containers => {
                self.handle_logs_action()
            }

            // Image actions (when on Images tab)
            KeyCode::Char('d') if self.state.current_tab == Tab::Images => {
                self.handle_image_remove_action()
            }
            KeyCode::Char('p') if self.state.current_tab == Tab::Images => {
                self.handle_image_prune_action()
            }
            KeyCode::Char('i') if self.state.current_tab == Tab::Images => {
                self.handle_image_inspect_action()
            }

            // List navigation (when on Containers or Images tab)
            KeyCode::Char('j') => {
                match self.state.current_tab {
                    Tab::Containers => self.state.next_container(),
                    Tab::Images => self.state.next_image(),
                    _ => {}
                }
                UiAction::None
            }
            KeyCode::Char('k') => {
                match self.state.current_tab {
                    Tab::Containers => self.state.previous_container(),
                    Tab::Images => self.state.previous_image(),
                    _ => {}
                }
                UiAction::None
            }
            KeyCode::Up => {
                match self.state.current_tab {
                    Tab::Containers => self.state.previous_container(),
                    Tab::Images => self.state.previous_image(),
                    _ => self.previous_tab(),
                }
                UiAction::None
            }
            KeyCode::Down => {
                match self.state.current_tab {
                    Tab::Containers => self.state.next_container(),
                    Tab::Images => self.state.next_image(),
                    _ => self.next_tab(),
                }
                UiAction::None
            }

            // Help
            KeyCode::Char('?') | KeyCode::Char('h') if key.modifiers.is_empty() => {
                self.state.show_help = !self.state.show_help;
                UiAction::None
            }

            _ => {
                debug!("Unhandled key: {:?}", key);
                UiAction::None
            }
        }
    }

    /// Handle confirmation dialog keys
    fn handle_confirmation_key(&mut self, key: KeyEvent) -> UiAction {
        match key.code {
            KeyCode::Char('y') | KeyCode::Enter => {
                // Confirm action
                if let Some(confirm) = self.state.confirm_dialog.take() {
                    return confirm.action;
                }
                UiAction::None
            }
            KeyCode::Char('n') | KeyCode::Esc => {
                // Cancel action
                self.state.confirm_dialog = None;
                UiAction::None
            }
            _ => UiAction::None,
        }
    }

    /// Get the currently selected container ID
    fn selected_container_id(&self) -> Option<String> {
        self.state
            .containers
            .get(self.state.container_list_selected)
            .map(|c| c.id.clone())
    }

    /// Handle start/stop action
    fn handle_start_stop_action(&mut self) -> UiAction {
        if let Some(container) = self.state.containers.get(self.state.container_list_selected) {
            let id = container.id.clone();
            match container.state {
                ContainerState::Running => {
                    // Running -> Stop
                    UiAction::StopContainer(id)
                }
                ContainerState::Paused => {
                    // Paused -> Unpause
                    UiAction::UnpauseContainer(id)
                }
                _ => {
                    // Stopped/Exited -> Start
                    UiAction::StartContainer(id)
                }
            }
        } else {
            UiAction::None
        }
    }

    /// Handle restart action
    fn handle_restart_action(&mut self) -> UiAction {
        if let Some(id) = self.selected_container_id() {
            UiAction::RestartContainer(id)
        } else {
            UiAction::None
        }
    }

    /// Handle pause action
    fn handle_pause_action(&mut self) -> UiAction {
        if let Some(container) = self.state.containers.get(self.state.container_list_selected) {
            let id = container.id.clone();
            if container.state == ContainerState::Paused {
                UiAction::UnpauseContainer(id)
            } else {
                UiAction::PauseContainer(id)
            }
        } else {
            UiAction::None
        }
    }

    /// Handle kill action (with confirmation)
    fn handle_kill_action(&mut self) -> UiAction {
        if let Some(container) = self.state.containers.get(self.state.container_list_selected) {
            let name = container.names.first().cloned().unwrap_or_else(|| container.short_id.clone());
            let id = container.id.clone();
            
            self.state.confirm_dialog = Some(ConfirmAction {
                message: format!("Kill container '{}'?", name),
                action: UiAction::KillContainer(id),
            });
        }
        UiAction::None
    }

    /// Handle remove action (with confirmation)
    fn handle_remove_action(&mut self) -> UiAction {
        if let Some(container) = self.state.containers.get(self.state.container_list_selected) {
            let name = container.names.first().cloned().unwrap_or_else(|| container.short_id.clone());
            let id = container.id.clone();
            
            self.state.confirm_dialog = Some(ConfirmAction {
                message: format!("Remove container '{}'?", name),
                action: UiAction::RemoveContainer(id),
            });
        }
        UiAction::None
    }

    /// Handle logs action
    fn handle_logs_action(&mut self) -> UiAction {
        if let Some(id) = self.selected_container_id() {
            UiAction::ShowContainerLogs(id)
        } else {
            UiAction::None
        }
    }

    /// Handle image remove action (with confirmation)
    fn handle_image_remove_action(&mut self) -> UiAction {
        if let Some(image) = self.state.images.get(self.state.image_list_selected) {
            let name = if image.dangling {
                "<dangling>".to_string()
            } else {
                image.repo_tags.first().cloned().unwrap_or_else(|| image.short_id.clone())
            };
            let id = image.id.clone();
            
            self.state.confirm_dialog = Some(ConfirmAction {
                message: format!("Remove image '{}'?", name),
                action: UiAction::RemoveImage(id),
            });
        }
        UiAction::None
    }

    /// Handle image prune action (with confirmation)
    fn handle_image_prune_action(&mut self) -> UiAction {
        self.state.confirm_dialog = Some(ConfirmAction {
            message: "Remove all dangling images?".to_string(),
            action: UiAction::PruneImages,
        });
        UiAction::None
    }

    /// Handle image inspect action
    fn handle_image_inspect_action(&mut self) -> UiAction {
        if let Some(image) = self.state.images.get(self.state.image_list_selected) {
            let id = image.id.clone();
            UiAction::InspectImage(id)
        } else {
            UiAction::None
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

        // Render notification overlay (if any)
        if let Some(notif) = self.state.notifications.last() {
            self.render_notification(frame, area, notif);
        }

        // Render confirmation dialog if active
        if let Some(ref confirm) = self.state.confirm_dialog {
            self.render_confirmation_dialog(frame, area, confirm);
        }

        // Render help overlay if active
        if self.state.show_help {
            self.render_help_overlay(frame, area);
        }
    }

    /// Render confirmation dialog
    fn render_confirmation_dialog(&self, frame: &mut Frame, area: Rect, confirm: &ConfirmAction) {
        // Create a centered popup (50% width, 20% height, min 8 lines)
        let popup_area = Self::centered_rect(50, 20, area);
        let popup_area = popup_area.intersection(Rect {
            x: popup_area.x,
            y: popup_area.y,
            width: popup_area.width,
            height: popup_area.height.max(8),
        });

        // Clear the background
        frame.render_widget(Clear, popup_area);

        // Create layout for dialog content
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Min(1), // Message
                Constraint::Length(1), // Spacer
                Constraint::Length(1), // Buttons
            ])
            .split(popup_area);

        // Render dialog block with title
        let block = Block::default()
            .title(" ⚠️ Confirmation ")
            .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));
        frame.render_widget(block, popup_area);

        // Render message
        let message = Paragraph::new(confirm.message.as_str())
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: true })
            .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(message, layout[0]);

        // Render buttons hint
        let buttons = Line::from(vec![
            Span::styled("[", Style::default().fg(Color::Gray)),
            Span::styled("y", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled("] Yes   [", Style::default().fg(Color::Gray)),
            Span::styled("n", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled("] No", Style::default().fg(Color::Gray)),
        ]);
        let buttons_para = Paragraph::new(buttons)
            .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(buttons_para, layout[2]);
    }

    /// Render notification toast
    fn render_notification(&self, frame: &mut Frame, area: Rect, notif: &crate::state::Notification) {
        use ratatui::widgets::{Clear, Paragraph};
        
        let color = match notif.level {
            crate::core::NotificationLevel::Info => Color::Blue,
            crate::core::NotificationLevel::Success => Color::Green,
            crate::core::NotificationLevel::Warning => Color::Yellow,
            crate::core::NotificationLevel::Error => Color::Red,
        };
        
        let text = format!(" {} ", notif.message);
        let width = text.len() as u16 + 2;
        let height = 3;
        
        // Position at top-right of screen
        let x = area.width.saturating_sub(width + 2);
        let y = 1;
        
        if x > 0 && area.height > height + y {
            let notif_area = Rect::new(x, y, width.min(area.width - x), height);
            
            frame.render_widget(Clear, notif_area);
            
            let paragraph = Paragraph::new(text)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(color))
                )
                .style(Style::default().fg(color));
            
            frame.render_widget(paragraph, notif_area);
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
        
        // Render tab-specific content
        match self.state.current_tab {
            Tab::Containers if !self.state.containers.is_empty() => {
                self.render_containers_split_view(frame, content_layout[1]);
            }
            Tab::Images if !self.state.images.is_empty() => {
                self.render_images_view(frame, content_layout[1]);
            }
            _ => {
                self.render_main_panel(frame, content_layout[1]);
            }
        }
    }

    /// Render images view
    fn render_images_view(&self, frame: &mut Frame, area: Rect) {
        // Create image list widget
        let mut widget = crate::ui::components::ImageListWidget::new(self.state.images.clone());
        if !self.state.images.is_empty() {
            widget.set_selected(Some(self.state.image_list_selected));
        }
        let table = widget.build_table();
        let mut table_state = ratatui::widgets::TableState::default();
        table_state.select(Some(self.state.image_list_selected));
        frame.render_stateful_widget(table, area, &mut table_state);
    }

    /// Render containers in split view (list | detail)
    fn render_containers_split_view(&self, frame: &mut Frame, area: Rect) {
        use crate::ui::components::{ContainerDetailPanel, SplitLayout};

        // Split area: 60% for list, 40% for detail
        let (list_area, detail_area) = SplitLayout::horizontal_split(area, 60);

        // Create container list
        let mut widget = ContainerListWidget::new(self.state.containers.clone());
        if !self.state.containers.is_empty() {
            widget.set_selected(Some(self.state.container_list_selected));
        }
        let table = widget.build_table();
        let mut table_state = ratatui::widgets::TableState::default();
        table_state.select(Some(self.state.container_list_selected));
        frame.render_stateful_widget(table, list_area, &mut table_state);

        // Render detail panel for selected container
        if let Some(container) = self.state.containers.get(self.state.container_list_selected) {
            let detail = ContainerDetailPanel::draw(container);
            frame.render_widget(detail, detail_area);
        } else {
            // Empty state
            let empty = Paragraph::new("No container selected")
                .block(Block::default().borders(Borders::ALL));
            frame.render_widget(empty, detail_area);
        }
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
        // Containers tab is handled by render_containers_split_view
        // when there are containers, otherwise fall through to simple view
        if self.state.current_tab == Tab::Containers && !self.state.containers.is_empty() {
            return;
        }
        self.render_simple_tab(frame, area);
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
            Tab::Containers => {
                // This should only happen when containers list is empty
                "No containers found.\n\nDocker may not be running or you may not have permissions.".to_string()
            }
        };

        let paragraph = Paragraph::new(content)
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, inner_area);
    }

    /// Render the footer
    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        let help_text = if self.state.confirm_dialog.is_some() {
            Cow::Borrowed(" [y]Yes [n]No ")
        } else if self.state.show_help {
            Cow::Borrowed(" Press any key to close help ")
        } else if self.state.current_tab == Tab::Containers && !self.state.containers.is_empty() {
            Cow::Borrowed(" [↑/↓]Select [s]Start [p]Pause [r]Restart [k]Kill [d]Delete [l]Logs [?]Help [q]Quit ")
        } else if self.state.current_tab == Tab::Images && !self.state.images.is_empty() {
            Cow::Borrowed(" [↑/↓]Select [d]Delete [p]Prune [i]Inspect [?]Help [q]Quit ")
        } else {
            Cow::Borrowed(" [←/→ or 1-6]:Switch Tabs | [?]:Help | [q]:Quit ")
        };

        let style = if self.state.confirm_dialog.is_some() {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray).bg(Color::Black)
        };

        let footer = Paragraph::new(help_text).style(style);
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
  s                 Start/Stop selected container
  r                 Restart selected container
  p                 Pause/Unpause selected container
  k                 Kill selected container
  d                 Delete selected container
  l                 View container logs

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
