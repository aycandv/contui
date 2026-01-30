//! Exec viewer component - bottom panel style

use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::state::ExecViewState;

/// Height of the exec panel
pub const EXEC_PANEL_HEIGHT: u16 = 10;

/// Render the exec viewer as a bottom panel
pub fn render_exec_panel(frame: &mut Frame, area: Rect, state: &ExecViewState) {
    let title = format!(
        " Exec: {} [{}] {} ",
        state.container_name,
        if state.focus { "FOCUS" } else { "UI" },
        state.status
    );

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    let lines: Vec<Line> = if state.screen_lines.is_empty() {
        vec![Line::from("Starting exec...")]
    } else {
        state
            .screen_lines
            .iter()
            .take(inner_area.height as usize)
            .map(|l: &String| Line::from(l.as_str()))
            .collect()
    };

    let para = Paragraph::new(lines);
    frame.render_widget(para, inner_area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn renders_exec_header() {
        let backend = TestBackend::new(80, 8);
        let mut terminal = Terminal::new(backend).unwrap();
        let state = ExecViewState {
            container_id: "id".into(),
            container_name: "web".into(),
            focus: true,
            status: "Running".into(),
            screen_lines: vec!["hello".into()],
        };

        terminal
            .draw(|f| render_exec_panel(f, f.area(), &state))
            .unwrap();

        let buffer = terminal.backend().buffer();
        let cell = buffer.cell((0, 0)).expect("cell at 0,0");
        assert!(!cell.symbol().is_empty());
    }
}
