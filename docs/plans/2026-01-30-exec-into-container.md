# Exec Into Container Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add an embedded exec pane under the container list that opens an interactive shell inside a container with full TTY support, focus toggle (Ctrl+E), and clean exit handling.

**Architecture:** Keep exec runtime (PTY/VT parser + streams) in `App` where non-clone types can live, while `AppState` holds only displayable, cloneable data for rendering. Use Bollard exec APIs and a VT parser (e.g., `vt100`) to render full-screen apps in the pane.

**Tech Stack:** Rust, ratatui, crossterm, tokio, bollard, vt100.

---

### Task 1: Add exec command selection helpers + tests

**Files:**
- Create: `src/docker/exec.rs`
- Modify: `src/docker/mod.rs`
- Test: `src/docker/exec.rs`

**Step 1: Write the failing tests**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selects_shell_entrypoint_with_cmd() {
        let entrypoint = vec!["/bin/sh".to_string(), "-lc".to_string()];
        let cmd = vec!["echo".to_string(), "hi".to_string()];
        let selected = select_exec_command(&entrypoint, &cmd);
        assert_eq!(selected, vec!["/bin/sh", "-lc", "echo", "hi"]);
    }

    #[test]
    fn falls_back_to_sh_when_not_shell() {
        let entrypoint = vec!["/usr/bin/myapp".to_string()];
        let cmd = vec!["--port".to_string(), "8080".to_string()];
        let selected = select_exec_command(&entrypoint, &cmd);
        assert_eq!(selected, vec!["/bin/sh", "-lc"]);
    }

    #[test]
    fn detects_shell_by_basename() {
        assert!(looks_like_shell(&["/bin/bash".to_string()]));
        assert!(!looks_like_shell(&["/usr/bin/python".to_string()]));
    }
}
```

**Step 2: Run tests to verify they fail**
Run: `cargo test docker::exec::tests::selects_shell_entrypoint_with_cmd`  
Expected: FAIL (module or functions not found)

**Step 3: Implement minimal helpers**
```rust
pub fn looks_like_shell(cmd: &[String]) -> bool {
    if cmd.is_empty() {
        return false;
    }
    let last = cmd[0].rsplit('/').next().unwrap_or(&cmd[0]);
    matches!(last, "sh" | "bash" | "zsh" | "ash" | "dash") || cmd.iter().any(|c| c == "-lc" || c == "-c")
}

pub fn select_exec_command(entrypoint: &[String], cmd: &[String]) -> Vec<String> {
    if looks_like_shell(entrypoint) || looks_like_shell(cmd) {
        let mut out = Vec::new();
        out.extend_from_slice(entrypoint);
        out.extend_from_slice(cmd);
        if out.is_empty() {
            return vec!["/bin/sh".to_string(), "-lc".to_string()];
        }
        out
    } else {
        vec!["/bin/sh".to_string(), "-lc".to_string()]
    }
}
```

**Step 4: Run tests to verify they pass**
Run: `cargo test docker::exec::tests::selects_shell_entrypoint_with_cmd`  
Expected: PASS

**Step 5: Commit**
```bash
git add src/docker/exec.rs src/docker/mod.rs

git commit -m "feat(docker): add exec command selection helpers"
```

---

### Task 2: Add exec defaults + start/resize API wrappers

**Files:**
- Modify: `src/docker/exec.rs`
- Test: `src/docker/exec.rs`

**Step 1: Write the failing tests**
```rust
#[cfg(test)]
mod defaults_tests {
    use super::*;

    #[test]
    fn exec_defaults_struct_is_cloneable() {
        let d = ExecDefaults {
            container_id: "id".into(),
            container_name: "name".into(),
            entrypoint: vec!["/bin/sh".into()],
            cmd: vec!["-lc".into()],
            running: true,
        };
        let _ = d.clone();
    }
}
```

**Step 2: Run tests to verify they fail**
Run: `cargo test docker::exec::defaults_tests::exec_defaults_struct_is_cloneable`  
Expected: FAIL (ExecDefaults not found)

**Step 3: Implement wrappers**
```rust
use bollard::exec::{CreateExecOptions, ResizeExecOptions, StartExecOptions, StartExecResults};
use tokio::io::AsyncWrite;
use futures_core::Stream;
use std::pin::Pin;

#[derive(Debug, Clone)]
pub struct ExecDefaults {
    pub container_id: String,
    pub container_name: String,
    pub entrypoint: Vec<String>,
    pub cmd: Vec<String>,
    pub running: bool,
}

pub struct ExecStart {
    pub exec_id: String,
    pub output: Pin<Box<dyn Stream<Item = crate::core::Result<bollard::container::LogOutput>> + Send>>,
    pub input: Pin<Box<dyn AsyncWrite + Send>>,
}

impl DockerClient {
    pub async fn exec_defaults(&self, id: &str) -> Result<ExecDefaults> {
        let inspect = self.inner().inspect_container(id, None).await
            .map_err(|e| DockerError::Container(format!("Failed to inspect: {e}")))?;

        let config = inspect.config.unwrap_or_default();
        let entrypoint = config.entrypoint.unwrap_or_default().into_iter().map(Into::into).collect();
        let cmd = config.cmd.unwrap_or_default().into_iter().map(Into::into).collect();
        let name = inspect.name.unwrap_or_default().trim_start_matches('/').to_string();
        let running = inspect.state.and_then(|s| s.running).unwrap_or(false);

        Ok(ExecDefaults { container_id: id.to_string(), container_name: name, entrypoint, cmd, running })
    }

    pub async fn start_exec_session(
        &self,
        container_id: &str,
        cmd: Vec<String>,
        cols: u16,
        rows: u16,
    ) -> Result<ExecStart> {
        let create = CreateExecOptions {
            attach_stdin: Some(true),
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            tty: Some(true),
            cmd: Some(cmd),
            ..Default::default()
        };

        let exec = self.inner().create_exec(container_id, create).await
            .map_err(|e| DockerError::Container(format!("Failed to create exec: {e}")))?;

        // resize before start
        let _ = self.inner().resize_exec(&exec.id, ResizeExecOptions { width: cols, height: rows }).await;

        let started = self.inner().start_exec(&exec.id, Some(StartExecOptions { detach: false, tty: true, output_capacity: None })).await
            .map_err(|e| DockerError::Container(format!("Failed to start exec: {e}")))?;

        match started {
            StartExecResults::Attached { output, input } => Ok(ExecStart { exec_id: exec.id, output, input }),
            StartExecResults::Detached => Err(DockerError::Container("Exec detached unexpectedly".into()).into()),
        }
    }

    pub async fn resize_exec_session(&self, exec_id: &str, cols: u16, rows: u16) -> Result<()> {
        self.inner().resize_exec(exec_id, ResizeExecOptions { width: cols, height: rows }).await
            .map_err(|e| DockerError::Container(format!("Failed to resize exec: {e}")))?;
        Ok(())
    }
}
```

**Step 4: Run tests to verify they pass**
Run: `cargo test docker::exec::defaults_tests::exec_defaults_struct_is_cloneable`  
Expected: PASS

**Step 5: Commit**
```bash
git add src/docker/exec.rs

git commit -m "feat(docker): add exec defaults/start/resize helpers"
```

---

### Task 3: Add exec view state to AppState + tests

**Files:**
- Modify: `src/state/app_state.rs`
- Test: `src/state/app_state.rs`

**Step 1: Write failing tests**
```rust
#[cfg(test)]
mod exec_state_tests {
    use super::*;

    #[test]
    fn exec_view_open_close_toggle_focus() {
        let mut state = AppState::new();
        state.open_exec_view("abc".into(), "web".into());
        assert!(state.exec_view.is_some());
        assert!(!state.exec_view.as_ref().unwrap().focus);
        state.toggle_exec_focus();
        assert!(state.exec_view.as_ref().unwrap().focus);
        state.close_exec_view();
        assert!(state.exec_view.is_none());
    }
}
```

**Step 2: Run tests to verify they fail**
Run: `cargo test state::app_state::exec_state_tests::exec_view_open_close_toggle_focus`  
Expected: FAIL (methods/struct not found)

**Step 3: Implement exec state**
```rust
#[derive(Debug, Clone)]
pub struct ExecViewState {
    pub container_id: String,
    pub container_name: String,
    pub focus: bool,
    pub status: String,
    pub screen_lines: Vec<String>,
}

impl AppState {
    pub fn open_exec_view(&mut self, container_id: String, container_name: String) {
        self.exec_view = Some(ExecViewState {
            container_id,
            container_name,
            focus: false,
            status: "Running".to_string(),
            screen_lines: vec![],
        });
        // Optional: close stats panel to avoid stacking
        self.stats_view = None;
    }

    pub fn close_exec_view(&mut self) { self.exec_view = None; }

    pub fn toggle_exec_focus(&mut self) {
        if let Some(ref mut v) = self.exec_view { v.focus = !v.focus; }
    }

    pub fn update_exec_screen(&mut self, lines: Vec<String>, status: Option<String>) {
        if let Some(ref mut v) = self.exec_view {
            v.screen_lines = lines;
            if let Some(s) = status { v.status = s; }
        }
    }
}
```

**Step 4: Run tests to verify they pass**
Run: `cargo test state::app_state::exec_state_tests::exec_view_open_close_toggle_focus`  
Expected: PASS

**Step 5: Commit**
```bash
git add src/state/app_state.rs

git commit -m "feat(state): add exec view state"
```

---

### Task 4: Add exec pane renderer + tests

**Files:**
- Create: `src/ui/components/exec_viewer.rs`
- Modify: `src/ui/components/mod.rs`
- Test: `src/ui/components/exec_viewer.rs`

**Step 1: Write failing tests**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use crate::state::ExecViewState;

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
        terminal.draw(|f| render_exec_panel(f, f.area(), &state)).unwrap();
        let buffer = terminal.backend().buffer();
        let title = buffer.get(1, 0).symbol.clone();
        assert!(!title.is_empty());
    }
}
```

**Step 2: Run tests to verify they fail**
Run: `cargo test ui::components::exec_viewer::tests::renders_exec_header`  
Expected: FAIL (module not found)

**Step 3: Implement renderer**
```rust
pub const EXEC_PANEL_HEIGHT: u16 = 10;

pub fn render_exec_panel(frame: &mut Frame, area: Rect, state: &ExecViewState) {
    let title = format!(
        " Exec: {} [{}] {} ",
        state.container_name,
        if state.focus { "FOCUS" } else { "UI" },
        state.status,
    );

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines: Vec<Line> = state
        .screen_lines
        .iter()
        .take(inner.height as usize)
        .map(|l| Line::from(l.as_str()))
        .collect();

    let para = Paragraph::new(lines);
    frame.render_widget(para, inner);
}
```

**Step 4: Run tests to verify they pass**
Run: `cargo test ui::components::exec_viewer::tests::renders_exec_header`  
Expected: PASS

**Step 5: Commit**
```bash
git add src/ui/components/exec_viewer.rs src/ui/components/mod.rs

git commit -m "feat(ui): add exec pane renderer"
```

---

### Task 5: Add key encoding helper + tests

**Files:**
- Create: `src/exec/input.rs`
- Modify: `src/exec/mod.rs`
- Test: `src/exec/input.rs`

**Step 1: Write failing tests**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn encodes_basic_keys() {
        assert_eq!(encode_key_event(KeyEvent::from(KeyCode::Enter)).unwrap(), b"\r".to_vec());
        assert_eq!(encode_key_event(KeyEvent::from(KeyCode::Char('a'))).unwrap(), b"a".to_vec());
    }

    #[test]
    fn encodes_arrow_keys() {
        let up = encode_key_event(KeyEvent::from(KeyCode::Up)).unwrap();
        assert_eq!(up, b"\x1b[A".to_vec());
    }

    #[test]
    fn ctrl_e_is_none_for_exec() {
        let key = KeyEvent::new(KeyCode::Char('e'), KeyModifiers::CONTROL);
        assert!(encode_key_event(key).is_none());
    }
}
```

**Step 2: Run tests to verify they fail**
Run: `cargo test exec::input::tests::encodes_basic_keys`  
Expected: FAIL (module not found)

**Step 3: Implement encoder**
```rust
pub fn encode_key_event(key: KeyEvent) -> Option<Vec<u8>> {
    if key.code == KeyCode::Char('e') && key.modifiers.contains(KeyModifiers::CONTROL) {
        return None;
    }
    let bytes = match key.code {
        KeyCode::Enter => b"\r".to_vec(),
        KeyCode::Backspace => b"\x7f".to_vec(),
        KeyCode::Tab => b"\t".to_vec(),
        KeyCode::Esc => b"\x1b".to_vec(),
        KeyCode::Up => b"\x1b[A".to_vec(),
        KeyCode::Down => b"\x1b[B".to_vec(),
        KeyCode::Right => b"\x1b[C".to_vec(),
        KeyCode::Left => b"\x1b[D".to_vec(),
        KeyCode::Home => b"\x1b[H".to_vec(),
        KeyCode::End => b"\x1b[F".to_vec(),
        KeyCode::PageUp => b"\x1b[5~".to_vec(),
        KeyCode::PageDown => b"\x1b[6~".to_vec(),
        KeyCode::Delete => b"\x1b[3~".to_vec(),
        KeyCode::Insert => b"\x1b[2~".to_vec(),
        KeyCode::Char(c) => {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                let v = (c as u8) & 0x1f;
                vec![v]
            } else {
                c.to_string().into_bytes()
            }
        }
        _ => return None,
    };
    Some(bytes)
}
```

**Step 4: Run tests to verify they pass**
Run: `cargo test exec::input::tests::encodes_basic_keys`  
Expected: PASS

**Step 5: Commit**
```bash
git add src/exec/input.rs src/exec/mod.rs

git commit -m "feat(exec): add key encoding helper"
```

---

### Task 6: Wire UI actions + exec pane layout

**Files:**
- Modify: `src/core/mod.rs` (add UiAction variants)
- Modify: `src/ui/app.rs`
- Test: `src/ui/app.rs`

**Step 1: Write failing tests**
```rust
#[test]
fn exec_key_triggers_action() {
    let mut app = UiApp::new(AppState::default());
    app.state.current_tab = Tab::Containers;
    app.state.containers = vec![crate::core::ContainerSummary { state: crate::core::ContainerState::Running, ..Default::default() }];
    let action = app.handle_event(crossterm::event::Event::Key(
        crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Char('x')),
    ));
    matches!(action, UiAction::ExecContainer(_));
}
```

**Step 2: Run tests to verify they fail**
Run: `cargo test ui::app::tests::exec_key_triggers_action`  
Expected: FAIL (UiAction missing)

**Step 3: Implement UI wiring**
- Add `UiAction::ExecContainer(String)` and `UiAction::ExecInput(Vec<u8>)` to `core/mod.rs`.
- In `UiApp::handle_key_event`:
  - If `exec_view` exists **and** `focus == true`, route keys through `encode_key_event` and return `UiAction::ExecInput(bytes)` (unless Ctrl+E, which toggles focus).
  - If `x` on Containers tab: open confirm dialog if container not running (use `UiAction::StartContainerAndExec`), otherwise return `UiAction::ExecContainer(id)`.
  - Ctrl+E toggles focus if exec_view exists.
- In `render_containers_split_view`, add exec panel below list when exec_view exists. Use `EXEC_PANEL_HEIGHT` and close stats panel when opening exec.
- Update footer + help overlay to include `x` and `Ctrl+E`.

**Step 4: Run tests to verify they pass**
Run: `cargo test ui::app::tests::exec_key_triggers_action`  
Expected: PASS

**Step 5: Commit**
```bash
git add src/core/mod.rs src/ui/app.rs

git commit -m "feat(ui): add exec actions and pane layout"
```

---

### Task 7: Add exec runtime in App + Docker integration

**Files:**
- Modify: `src/app.rs`
- Modify: `src/core/mod.rs` (add `StartContainerAndExec` if needed)
- Test: `src/app.rs` (unit tests for layout helpers)

**Step 1: Write failing tests**
```rust
#[cfg(test)]
mod exec_layout_tests {
    use super::*;

    #[test]
    fn computes_exec_pane_size() {
        let (cols, rows) = compute_exec_pane_size(120, 30);
        assert!(cols > 0);
        assert!(rows > 0);
    }
}
```

**Step 2: Run tests to verify they fail**
Run: `cargo test app::tests::exec_layout_tests::computes_exec_pane_size`  
Expected: FAIL (helper not found)

**Step 3: Implement runtime + handlers**
- Add `exec_runtime: Option<ExecRuntime>` to `App`.
- Add `ExecRuntime` struct (non-clone):
  - `exec_id: String`
  - `container_id: String`
  - `input: Pin<Box<dyn AsyncWrite + Send>>`
  - `parser: vt100::Parser`
  - `output_rx: mpsc::Receiver<Vec<u8>>`
- Add `start_exec_session` in `App`:
  - call `docker_client.exec_defaults` → build command with `select_exec_command`
  - call `docker_client.start_exec_session` with pane size
  - spawn `tokio::spawn` to read `output` stream, extract bytes from `LogOutput`, and `send` to channel
  - create parser with pane size; update `state.open_exec_view`
- Add `check_exec_output` in event loop tick:
  - drain `output_rx`, feed parser, update `state.exec_view.screen_lines`
  - when stream ends, close exec view + clear runtime
- Add `handle_exec_input` in `handle_ui_action`:
  - write bytes to `exec_runtime.input` via `write_all`
- Add `compute_exec_pane_size(term_w, term_h)` helper that mirrors layout constants used in UI.
- On `Resize` events, if exec is running, call `docker_client.resize_exec_session` and resize parser.

**Step 4: Run tests to verify they pass**
Run: `cargo test app::tests::exec_layout_tests::computes_exec_pane_size`  
Expected: PASS

**Step 5: Commit**
```bash
git add src/app.rs src/core/mod.rs

git commit -m "feat(app): add exec runtime wiring"
```

---

### Task 8: Add dependency + integration notes

**Files:**
- Modify: `Cargo.toml`
- Modify: `docs/TESTING.md`

**Step 1: Write failing test (compile gate)**
No new unit test; rely on `cargo test` to catch missing dependency.

**Step 2: Run tests to verify they fail**
Run: `cargo test`  
Expected: FAIL (missing vt100)

**Step 3: Add dependency + docs note**
```toml
# Cargo.toml
vt100 = "0.15"
```

```markdown
# docs/TESTING.md
Add manual steps for exec pane (open shell, run top, resize, exit).
```

**Step 4: Run tests to verify they pass**
Run: `cargo test`  
Expected: PASS

**Step 5: Commit**
```bash
git add Cargo.toml docs/TESTING.md

git commit -m "chore(exec): add vt100 dependency and testing notes"
```

---

## Execution Handoff
Plan complete and saved to `docs/plans/2026-01-30-exec-into-container.md`.

Two execution options:

1) **Subagent-Driven (this session)** – I dispatch a fresh subagent per task, review between tasks, fast iteration.

2) **Parallel Session (separate)** – Open new session in the worktree and execute with @executing-plans.

Which approach?
