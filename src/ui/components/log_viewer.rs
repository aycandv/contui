//! Log viewer widget

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::state::LogViewState;

/// Render the log viewer overlay
pub fn render_log_viewer(frame: &mut Frame, area: Rect, state: &LogViewState) {
    // Use 90% of screen for log viewer
    let popup_area = centered_rect(90, 90, area);

    // Clear background
    frame.render_widget(Clear, popup_area);

    // Split area for logs and optional search bar
    let (content_area, search_area) = if state.show_search_input {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Length(3)])
            .split(popup_area);
        (layout[0], Some(layout[1]))
    } else {
        (popup_area, None)
    };

    // Build title with follow and search indicators
    let search_indicator = if state.search_pattern.is_some() {
        format!(" [SEARCH: {}]", state.search_pattern.as_ref().unwrap())
    } else {
        String::new()
    };
    let title = format!(
        " Logs: {} {}{} ",
        state.container_name,
        if state.follow { "[FOLLOW]" } else { "" },
        search_indicator
    );

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner_area = block.inner(content_area);
    frame.render_widget(block, content_area);

    // Render search input if active
    if let Some(search_area) = search_area {
        let match_text = if state.search_matches.is_empty() {
            "0/0".to_string()
        } else {
            format!("{}/{}", 
                state.current_match.map(|i| i + 1).unwrap_or(0),
                state.search_matches.len()
            )
        };
        let search_text = format!("/{}  {}", 
            state.search_pattern.as_ref().map(|s| s.as_str()).unwrap_or(""),
            match_text
        );
        let search_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));
        let search_para = Paragraph::new(search_text)
            .style(Style::default().fg(Color::Yellow))
            .block(search_block);
        frame.render_widget(search_para, search_area);
    }

    // Calculate visible lines
    let visible_lines = inner_area.height as usize;
    let total_lines = state.logs.len();

    // Show message if no logs
    if total_lines == 0 {
        let no_logs = Paragraph::new("Press 'r' to load logs | 'q' to close | 'f' to toggle follow | '/' to search")
            .style(Style::default().fg(Color::Yellow))
            .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(no_logs, inner_area);
        return;
    }

    // Calculate start index based on scroll offset
    let start_idx = if total_lines > visible_lines {
        state.scroll_offset.saturating_sub(visible_lines / 2)
    } else {
        0
    };
    let start_idx = start_idx.min(total_lines.saturating_sub(visible_lines));

    // Build visible log lines with search highlighting
    let end_idx = (start_idx + visible_lines).min(total_lines);
    let log_lines: Vec<Line> = (start_idx..end_idx)
        .map(|idx| {
            let entry = &state.logs[idx];
            let timestamp = entry
                .timestamp
                .map(|ts: chrono::DateTime<chrono::Utc>| ts.format("%H:%M:%S").to_string())
                .unwrap_or_else(|| "??:??:??".to_string());

            // Check if this line is a search match
            let is_match = state.search_matches.contains(&idx);
            let is_current_match = state.current_match
                .map(|current_idx| state.search_matches.get(current_idx) == Some(&idx))
                .unwrap_or(false);

            let mut style = if entry.is_stderr {
                Style::default().fg(Color::Red)
            } else {
                Style::default().fg(Color::White)
            };

            // Highlight search matches
            if is_current_match {
                style = style.bg(Color::Magenta).fg(Color::Black);
            } else if is_match {
                style = style.bg(Color::Yellow).fg(Color::Black);
            }

            Line::from(vec![
                Span::styled(
                    format!("{} ", timestamp),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(entry.message.clone(), style),
            ])
        })
        .collect();

    let paragraph = Paragraph::new(log_lines).wrap(Wrap { trim: false });

    frame.render_widget(paragraph, inner_area);

    // Render scrollbar indicator if needed
    if total_lines > visible_lines {
        let scroll_pct = (state.scroll_offset as f64 / (total_lines - 1) as f64 * 100.0) as u16;
        let scroll_indicator = format!("{}%", scroll_pct);
        let scroll_area = Rect::new(
            popup_area.right() - 7,
            popup_area.y + 1,
            5,
            1,
        );
        frame.render_widget(
            Paragraph::new(scroll_indicator).style(Style::default().fg(Color::DarkGray)),
            scroll_area,
        );
    }
}

/// Calculate centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            ratatui::layout::Constraint::Percentage((100 - percent_y) / 2),
            ratatui::layout::Constraint::Percentage(percent_y),
            ratatui::layout::Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            ratatui::layout::Constraint::Percentage((100 - percent_x) / 2),
            ratatui::layout::Constraint::Percentage(percent_x),
            ratatui::layout::Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::docker::LogEntry;

    #[test]
    fn test_log_viewer_creation() {
        let state = LogViewState {
            container_id: "abc123".to_string(),
            container_name: "test-container".to_string(),
            logs: vec![
                LogEntry {
                    timestamp: None,
                    message: "Test log line".to_string(),
                    is_stderr: false,
                },
            ],
            scroll_offset: 0,
            follow: true,
            max_lines: 1000,
            search_pattern: None,
            search_matches: vec![],
            current_match: None,
            show_search_input: false,
        };

        assert_eq!(state.container_name, "test-container");
        assert!(state.follow);
    }
}
