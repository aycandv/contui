# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Contui is a terminal user interface (TUI) for Docker container management built with Rust. It provides real-time monitoring, log streaming, resource statistics, and full container lifecycle management.

## Build & Development Commands

```bash
# Build
cargo build                    # Debug build
cargo build --release          # Release build (optimized)

# Test
cargo test                     # Run all tests
cargo test <test_name>         # Run single test
cargo test --lib               # Unit tests only (no Docker required)
cargo test --test integration -- --ignored  # Integration tests (requires Docker)
cargo test -- --nocapture      # Show test output

# Lint & Format (CI checks these - run before committing)
cargo fmt                      # Format code
cargo fmt -- --check           # Check formatting
cargo clippy                   # Run linter

# Run
cargo run                      # Run debug build
./target/release/contui        # Run release build
RUST_LOG=contui=debug cargo run  # Run with debug logging
```

## Git Workflow

### Branching Strategy

- `main` - Production-ready code
- Feature branches: `feature/<short-description>` or `ralph/US-XXX-description` for PRD stories

### Commit Convention

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]

Relates to: US-XXX (if applicable)
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

Examples:
- `feat(docker): add container pause/unpause`
- `fix(ui): resolve log viewer scroll issue`
- `chore(deps): update ratatui to 0.29`

### Pre-Commit Checklist

Before committing, always run:
```bash
cargo check && cargo test && cargo fmt --check && cargo clippy
```

## Architecture

### Three-Layer Design

```
┌─────────────────────────────────────────────────────────────┐
│  Presentation Layer (ui/)                                    │
│  - Widgets, layouts, event handlers, ratatui rendering       │
├─────────────────────────────────────────────────────────────┤
│  Application Layer (app.rs, state/)                          │
│  - App coordinator, AppState, UiAction dispatch              │
│  - Background tasks: stats worker, log worker                │
├─────────────────────────────────────────────────────────────┤
│  Infrastructure Layer (docker/, config/, registry/)          │
│  - Docker API client (bollard), config loading               │
└─────────────────────────────────────────────────────────────┘
```

### Key Patterns

**Event Loop (app.rs)**:
1. Render UI → Poll terminal events → Handle event → Get `UiAction`
2. Execute action via `handle_ui_action()` → Periodic refresh (every 2s)

**State Cloning**: `AppState` is cloned into `UiApp` for each render cycle. Keeps UI pure, state mutations explicit.

**Async Data Fetching**: Log/stats use separate OS threads with own tokio runtimes to prevent blocking UI. Results via `mpsc` channels.

**UiAction Pattern**: UI returns action variants (`StartContainer(id)`, `ShowLogs(id)`) - `App` executes them. UI never calls Docker directly.

### Module Responsibilities

| Module | Purpose |
|--------|---------|
| `core/` | Domain types (`ContainerSummary`, `UiAction`, errors) |
| `docker/` | Bollard wrapper, one submodule per resource type |
| `config/` | TOML config loading, model structs |
| `state/` | `AppState` with UI state (selections, views, dialogs) |
| `ui/` | Ratatui rendering, event handling, widgets |
| `update/` | Self-update checking and installation |

### Configuration

Config file: `~/.config/contui/config.toml` (Linux) or platform equivalent.

Sections: `[general]`, `[ui]`, `[docker]`, `[keybindings]`, `[monitoring]`, `[logging]`, `[update]`

## Testing Strategy

### Test Types

| Type | Location | Docker Required |
|------|----------|-----------------|
| Unit tests | `src/**/*.rs` (`#[cfg(test)]`) | No |
| Integration tests | `tests/integration/` | Yes (marked `#[ignore]`) |

### UI Component Testing

Use `ratatui::backend::TestBackend` for widget tests:
```rust
let backend = TestBackend::new(80, 20);
let mut terminal = Terminal::new(backend).unwrap();
// Render widget, verify buffer contents
```

### Mocking Docker

Use `mockall` crate for tests without Docker daemon.

## Conventions

- **Commits**: Conventional commits (`feat:`, `fix:`, `chore:`, etc.)
- **TLS**: Uses `rustls` instead of `native-tls` for cross-compilation
- **Tests**: Docker-required tests marked `#[ignore = "requires Docker daemon"]`
- **Logging**: App logs to `/tmp/contui.log`, not stdout (avoids TUI pollution)
- **Errors**: Use `anyhow::Result` with `.context()` for error chains

## Documentation

Detailed docs in `docs/`:
- `ARCHITECTURE.md` - Full architecture design
- `DATA_MODELS.md` - Type definitions
- `UI_DESIGN.md` - UI specifications
- `API_INTEGRATION.md` - Docker API patterns
- `TESTING.md` - Testing strategy
- `GIT_WORKFLOW.md` - Git conventions
