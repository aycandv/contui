//! Log viewer widget

use ratatui::{
    layout::Rect,
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

    // Build title with follow indicator
    let title = format!(
        " Logs: {} {} ",
        state.container_name,
        if state.follow { "[FOLLOW]" } else { "" }
    );

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner_area = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    // Calculate visible lines
    let visible_lines = inner_area.height as usize;
    let total_lines = state.logs.len();

    // Calculate start index based on scroll offset
    let start_idx = if total_lines > visible_lines {
        state.scroll_offset.saturating_sub(visible_lines / 2)
    } else {
        0
    };
    let start_idx = start_idx.min(total_lines.saturating_sub(visible_lines));

    // Build visible log lines
    let log_lines: Vec<Line> = state
        .logs
        .iter()
        .skip(start_idx)
        .take(visible_lines)
        .map(|entry| {
            let timestamp = entry
                .timestamp
                .map(|ts: chrono::DateTime<chrono::Utc>| ts.format("%H:%M:%S").to_string())
                .unwrap_or_else(|| "??:??:??".to_string());

            let style = if entry.is_stderr {
                Style::default().fg(Color::Red)
            } else {
                Style::default().fg(Color::White)
            };

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
            popup_area.right() - 6,
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
        };

        assert_eq!(state.container_name, "test-container");
        assert!(state.follow);
    }
}
