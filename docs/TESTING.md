# DockMon Testing Strategy

## Overview

This document outlines how to test DockMon, including automated testing capabilities and manual verification workflows.

## Testing Capabilities

### ✅ What I Can Do

1. **Build & Compile Verification**
   ```bash
   cargo check          # Fast syntax/type checking
   cargo build          # Full compilation
   cargo build --release # Release build
   ```

2. **Unit Tests**
   ```bash
   cargo test           # Run all unit tests
   cargo test <filter>  # Run specific tests
   ```

3. **Integration Tests**
   - Test Docker client operations
   - Verify state management
   - Test configuration loading

4. **Code Quality Checks**
   ```bash
   cargo fmt --check    # Format verification
   cargo clippy         # Linting
   cargo audit          # Security audit
   ```

5. **Basic Runtime Verification**
   - Start the application
   - Verify it doesn't crash immediately
   - Check connection to Docker daemon
   - Send basic keystrokes
   - Verify log output

### ⚠️ Limitations

1. **Visual UI Verification**
   - I cannot "see" the terminal UI visually
   - Cannot verify colors, layouts, or rendering artifacts
   - Cannot verify mouse interactions visually

2. **Interactive TUI Testing**
   - Can send keyboard input but verifying visual output is limited
   - Cannot easily test complex UI state transitions
   - Cannot verify chart rendering visually

3. **Docker Environment**
   - Depends on Docker being available in the test environment
   - Some tests may require actual Docker daemon

## Testing Strategy

### 1. Unit Tests (Always Automated)

```rust
// In src/docker/containers.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_container_state_from_docker() {
        let docker_state = bollard::models::ContainerState {
            status: Some(ContainerStateStatusEnum::RUNNING),
            ..Default::default()
        };
        
        let state: ContainerState = docker_state.into();
        assert_eq!(state, ContainerState::Running);
    }
    
    #[test]
    fn test_container_sorting() {
        let mut containers = vec![
            ContainerSummary { name: "zebra".to_string(), ..Default::default() },
            ContainerSummary { name: "alpha".to_string(), ..Default::default() },
        ];
        
        containers.sort_by(|a, b| a.name.cmp(&b.name));
        assert_eq!(containers[0].name, "alpha");
    }
}
```

### 2. UI Component Tests (Using Test Backend)

```rust
// In src/ui/components/container_list.rs
#[cfg(test)]
mod tests {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    
    #[test]
    fn test_container_list_rendering() {
        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        
        let containers = vec![
            ContainerSummary {
                id: "abc123".to_string(),
                names: vec!["web".to_string()],
                state: ContainerState::Running,
                ..Default::default()
            },
        ];
        
        let widget = ContainerList::new(containers);
        
        terminal.draw(|f| {
            f.render_widget(widget, f.area());
        }).unwrap();
        
        // Verify buffer contents
        let expected = Buffer::with_lines(vec![
            "▶ abc123 web    ▶ Running  ...",
            // ...
        ]);
        
        terminal.backend().assert_buffer(&expected);
    }
}
```

### 3. Integration Tests

```rust
// In tests/docker_integration.rs
#[tokio::test]
#[ignore = "requires Docker daemon"]
async fn test_list_containers() {
    let client = DockerClient::from_env().await.unwrap();
    let containers = client.list_containers(true).await.unwrap();
    
    // Just verify it doesn't error
    // Don't assert on contents since environment varies
}

#[tokio::test]
async fn test_config_loading() {
    let config = Config::load("tests/fixtures/test_config.toml").unwrap();
    assert_eq!(config.general.poll_interval_ms, 1000);
}
```

### 4. End-to-End Tests (Simulated)

```rust
// In tests/e2e.rs
#[test]
fn test_app_startup() {
    // This would spawn the actual app and verify it starts
    let mut app = std::process::Command::new("cargo")
        .args(&["run", "--", "--help"])
        .output()
        .expect("Failed to execute");
    
    assert!(app.status.success());
    assert!(String::from_utf8_lossy(&app.stdout).contains("DockMon"));
}
```

## Manual Testing Checklist

### Basic Functionality

- [ ] Application starts without errors
- [ ] Connects to Docker daemon
- [ ] Shows container list
- [ ] Shows image list
- [ ] Navigation works (arrow keys, hjkl)
- [ ] Tab switching works
- [ ] Help panel displays (`?`)
- [ ] Quit works (`q`)

### Container Operations

- [ ] Start container (`s`)
- [ ] Stop container (`s`)
- [ ] Restart container (`r`)
- [ ] Pause container (`p`)
- [ ] Kill container (`k`)
- [ ] Remove container (`d`)

### Log Viewing

- [ ] Open logs (`l`)
- [ ] Scroll logs (up/down)
- [ ] Toggle follow (`f`)
- [ ] Search (`/`)
- [ ] Next/previous match (`n`/`N`)

### Stats Viewing

- [ ] Open stats (`c`)
- [ ] CPU chart displays
- [ ] Memory chart displays
- [ ] Network stats show

### Configuration

- [ ] Config file loads from `~/.config/dockmon/config.toml`
- [ ] Custom keybindings work
- [ ] Theme changes apply

## Continuous Integration

### GitHub Actions Workflow

```yaml
# .github/workflows/ci.yml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          components: rustfmt, clippy
      
      - name: Check formatting
        run: cargo fmt -- --check
      
      - name: Run clippy
        run: cargo clippy -- -D warnings
      
      - name: Run tests
        run: cargo test --lib  # Unit tests only
      
      - name: Build
        run: cargo build --release
```

## Test Coverage Goals

| Module | Target Coverage |
|--------|----------------|
| Core types | 90% |
| Docker client | 70% (limited by Docker requirement) |
| UI components | 60% (visual testing limited) |
| State management | 80% |
| Utils | 90% |

## Running Tests

```bash
# All tests
cargo test

# Unit tests only (no Docker required)
cargo test --lib

# Integration tests (requires Docker)
cargo test --test '*'

# Specific test
cargo test test_container_list

# With output
cargo test -- --nocapture

# Coverage (requires tarpaulin)
cargo tarpaulin --out Html
```

## Mocking Docker

For tests without Docker:

```rust
#[cfg(test)]
mockall::mock! {
    pub DockerClient {}
    
    #[async_trait]
    impl DockerClientTrait for DockerClient {
        async fn list_containers(&self, all: bool) -> Result<Vec<ContainerSummary>>;
        async fn start_container(&self, id: &str) -> Result<()>;
        // ...
    }
}

#[tokio::test]
async fn test_with_mock() {
    let mut mock = MockDockerClient::new();
    mock.expect_list_containers()
        .returning(|_| Ok(vec![ContainerSummary::default()]));
    
    let app = App::with_client(mock);
    let containers = app.list_containers().await.unwrap();
    assert_eq!(containers.len(), 1);
}
```

## Debugging Failed Tests

```bash
# Run with backtrace
RUST_BACKTRACE=1 cargo test

# Run single test with output
cargo test test_name -- --nocapture

# Run with logging
RUST_LOG=debug cargo test

# Check test binary
cargo test --no-run
```

## Performance Testing

```rust
#[test]
fn test_render_performance() {
    let backend = TestBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).unwrap();
    
    let start = std::time::Instant::now();
    
    for _ in 0..1000 {
        terminal.draw(|f| {
            // Render full UI
        }).unwrap();
    }
    
    let elapsed = start.elapsed();
    assert!(elapsed.as_millis() < 1000, "Rendering too slow: {:?}", elapsed);
}
```

## Conclusion

While I cannot visually verify the TUI like a human, I can:
- Ensure code compiles and passes all tests
- Verify logic through unit tests
- Test Docker operations through integration tests
- Verify UI components render expected characters
- Ensure the application starts and responds to input
- Check performance metrics

For visual verification, manual testing is required. The code includes comprehensive test infrastructure to catch most issues automatically.
