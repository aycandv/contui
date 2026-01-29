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
cargo test --test integration -- --ignored  # Run integration tests (requires Docker)

# Lint & Format (CI checks these)
cargo fmt                      # Format code
cargo fmt -- --check           # Check formatting
cargo clippy                   # Run linter

# Run
cargo run                      # Run debug build
./target/release/contui        # Run release build
RUST_LOG=contui=debug cargo run  # Run with debug logging
```

## Architecture

### Layer Overview

```
main.rs          CLI parsing, update checks, launches App
    │
    ▼
app.rs (App)     Main coordinator: owns DockerClient, AppState, event loop
    │
    ├──► docker/     Docker API wrapper (bollard)
    │    └── DockerClient, containers, images, volumes, networks, logs, stats
    │
    ├──► state/      Application state (AppState)
    │    └── containers, images, volumes, networks, UI state (dialogs, views)
    │
    └──► ui/         UI layer (ratatui)
         ├── UiApp       Event handling, returns UiAction
         └── components/ Widgets (ContainerList, LogViewer, etc.)
```

### Key Patterns

**Event Loop (app.rs:99-212)**: The `run_event_loop` method:
1. Renders UI via `UiApp::draw()`
2. Polls terminal events with timeout
3. Creates fresh `UiApp` from state, handles event, gets `UiAction`
4. Applies state changes back, executes action via `handle_ui_action()`
5. Periodic tasks: log/stats refresh, data refresh (every 2s)

**State Cloning**: `AppState` is cloned into `UiApp` for each render/event cycle. After handling, state is cloned back. This keeps UI pure and state mutations explicit.

**Async Data Fetching**: Log and stats fetching use separate OS threads with their own tokio runtimes to prevent blocking the UI (bollard can block). Results come back via `mpsc` channels checked in the event loop.

**UiAction Pattern**: UI components don't execute Docker operations directly. They return `UiAction` variants (e.g., `StartContainer(id)`, `ShowContainerLogs(id)`) which the `App` executes.

### Module Responsibilities

- **core/**: Domain types (`ContainerSummary`, `ImageSummary`, `UiAction`, errors)
- **docker/**: Bollard wrapper, each submodule handles one Docker resource type
- **config/**: TOML configuration loading and model structs
- **state/**: `AppState` with all UI state (selections, views, dialogs)
- **ui/**: Ratatui rendering, event handling, reusable widgets
- **update/**: Self-update checking and installation

### Configuration

Config file: `~/.config/contui/config.toml` (Linux) or platform equivalent. Uses `directories` crate for paths.

Sections: `[general]`, `[ui]`, `[docker]`, `[keybindings]`, `[monitoring]`, `[logging]`, `[update]`

## Conventions

- Use conventional commits: `feat:`, `fix:`, `chore:`, etc.
- Uses `rustls` instead of `native-tls` for cross-compilation compatibility
- Integration tests that require Docker are marked `#[ignore = "requires Docker daemon"]`
- Application logs go to `/tmp/contui.log`, not stdout (to avoid polluting TUI)
