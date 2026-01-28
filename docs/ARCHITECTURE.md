# DockMon Architecture Design

## Overview

DockMon follows a layered architecture with clear separation of concerns:
- **Presentation Layer**: UI components and event handling
- **Application Layer**: State management and business logic
- **Infrastructure Layer**: Docker API client and external services

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              Presentation Layer                              │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │   Widgets   │  │   Layouts   │  │   Styles    │  │   Event Handlers    │ │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────────────┘ │
├─────────────────────────────────────────────────────────────────────────────┤
│                              Application Layer                               │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │    App      │  │   State     │  │   Actions   │  │    Controllers      │ │
│  │   State     │  │  Updates    │  │             │  │                     │ │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────────────┘ │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                        Background Tasks                                │ │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌─────────────┐ │ │
│  │  │ Stats Worker │  │  Log Worker  │  │ Event Worker │  │  Registry   │ │ │
│  │  │              │  │              │  │              │  │   Worker    │ │ │
│  │  └──────────────┘  └──────────────┘  └──────────────┘  └─────────────┘ │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────────────────┤
│                             Infrastructure Layer                             │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                      Docker API Client (Bollard)                       │ │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌─────────────┐ │ │
│  │  │  Containers  │  │    Images    │  │   Networks   │  │   Volumes   │ │ │
│  │  └──────────────┘  └──────────────┘  └──────────────┘  └─────────────┘ │ │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐                  │ │
│  │  │   Compose    │  │    System    │  │     Exec     │                  │ │
│  │  └──────────────┘  └──────────────┘  └──────────────┘                  │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                      Registry Clients                                  │ │
│  │  ┌──────────────┐  ┌──────────────┐                                    │ │
│  │  │  Docker Hub  │  │   Custom     │                                    │ │
│  │  │              │  │  Registries  │                                    │ │
│  │  └──────────────┘  └──────────────┘                                    │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────────────────┤
│                              Shared Components                               │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                      Configuration & Utilities                         │ │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌─────────────┐ │ │
│  │  │    Config    │  │    Errors    │  │    Logging   │  │   Metrics   │ │ │
│  │  │   Manager    │  │   Handler    │  │              │  │   Storage   │ │ │
│  │  └──────────────┘  └──────────────┘  └──────────────┘  └─────────────┘ │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Module Structure

```
src/
├── main.rs                    # Entry point
├── app.rs                     # Main application coordinator
├── config/
│   ├── mod.rs                 # Configuration module
│   ├── model.rs               # Config data structures
│   └── loader.rs              # Config file loading
├── core/
│   ├── mod.rs                 # Core types and traits
│   ├── errors.rs              # Error types
│   ├── events.rs              # Event system
│   └── types.rs               # Shared type definitions
├── docker/
│   ├── mod.rs                 # Docker client module
│   ├── client.rs              # Docker client wrapper
│   ├── containers.rs          # Container operations
│   ├── images.rs              # Image operations
│   ├── networks.rs            # Network operations
│   ├── volumes.rs             # Volume operations
│   ├── compose.rs             # Docker Compose support
│   ├── stats.rs               # Stats collection
│   └── logs.rs                # Log streaming
├── registry/
│   ├── mod.rs                 # Registry module
│   ├── dockerhub.rs           # Docker Hub client
│   └── custom.rs              # Custom registry support
├── ui/
│   ├── mod.rs                 # UI module
│   ├── app.rs                 # App-level UI coordinator
│   ├── layout.rs              # Layout definitions
│   ├── components/
│   │   ├── mod.rs
│   │   ├── header.rs          # Header component
│   │   ├── footer.rs          # Status bar
│   │   ├── sidebar.rs         # Navigation sidebar
│   │   ├── container_list.rs  # Container list widget
│   │   ├── image_list.rs      # Image list widget
│   │   ├── log_viewer.rs      # Log viewer widget
│   │   ├── stats_chart.rs     # Stats chart widget
│   │   ├── detail_panel.rs    # Detail/info panel
│   │   └── help_panel.rs      # Help overlay
│   ├── screens/
│   │   ├── mod.rs
│   │   ├── main.rs            # Main screen
│   │   ├── container_detail.rs
│   │   ├── image_detail.rs
│   │   └── compose_view.rs
│   └── widgets/
│       ├── mod.rs
│       ├── table.rs           # Enhanced table widget
│       ├── chart.rs           # Chart widget
│       ├── scrollable.rs      # Scrollable container
│       └── search_bar.rs      # Search input widget
├── state/
│   ├── mod.rs                 # State management
│   ├── app_state.rs           # Application state
│   ├── container_state.rs     # Container-specific state
│   ├── metrics_store.rs       # Metrics history storage
│   └── log_buffer.rs          # Log ring buffer
└── utils/
    ├── mod.rs
    ├── formatting.rs          # Text formatting utilities
    ├── converters.rs          # Unit converters
    └── validators.rs          # Input validators
```

## Component Interactions

### 1. Main Event Loop

```rust
// Simplified main loop structure
loop {
    // 1. Handle terminal events (input)
    if event::poll(timeout)? {
        match event::read()? {
            Event::Key(key) => app.handle_key_event(key).await,
            Event::Mouse(mouse) => app.handle_mouse_event(mouse).await,
            Event::Resize(w, h) => app.handle_resize(w, h),
            _ => {}
        }
    }
    
    // 2. Process background task results
    while let Ok(msg) = message_rx.try_recv() {
        app.process_message(msg);
    }
    
    // 3. Render UI
    terminal.draw(|f| app.render(f))?;
}
```

### 2. Docker Event Streaming

```
┌─────────────┐    ┌──────────────────┐    ┌──────────────┐
│   Docker    │───▶│  Event Stream    │───▶│  App State   │
│   Daemon    │    │  (tokio task)    │    │   Update     │
└─────────────┘    └──────────────────┘    └──────────────┘
                          │
                          ▼
                   ┌──────────────┐
                   │   Channel    │
                   │  (broadcast) │
                   └──────────────┘
```

### 3. Stats Collection Flow

```
┌──────────────────┐     ┌───────────────┐     ┌──────────────┐
│  Stats Worker    │────▶│  Metrics      │────▶│  Time-Series │
│  (1s interval)   │     │  Processor    │     │  Storage     │
└──────────────────┘     └───────────────┘     └──────────────┘
         │                                              │
         │                                              ▼
         │                                       ┌──────────────┐
         └──────────────────────────────────────▶│   UI Chart   │
                                                 │   Update     │
                                                 └──────────────┘
```

### 4. Log Streaming Flow

```
┌──────────────┐    ┌───────────────┐    ┌──────────────┐    ┌──────────────┐
│   Log        │───▶│   Log         │───▶│   Ring       │───▶│   Log        │
│   Worker     │    │   Parser      │    │   Buffer     │    │   Viewer     │
│   (per       │    │   (JSON/      │    │   (circular) │    │   (render)   │
│   container) │    │    text)      │    │              │    │              │
└──────────────┘    └───────────────┘    └──────────────┘    └──────────────┘
```

## Data Flow Diagrams

### Container List Refresh

```
User presses 'r' to refresh
         │
         ▼
┌─────────────────┐
│  Key Handler    │
│  (containers)   │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Action::       │
│  RefreshList    │
└────────┬────────┘
         │
         ▼
┌─────────────────┐     ┌─────────────────┐
│  Docker Client  │────▶│  List Containers│
│  .containers()  │     │  API Call       │
└────────┬────────┘     └─────────────────┘
         │
         ▼
┌─────────────────┐
│  Update         │
│  ContainerState │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Trigger Render │
└─────────────────┘
```

### Log View with Search

```
User types "/" then search term
         │
         ▼
┌─────────────────┐
│  Search Mode    │
│  Activated      │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  User types     │
│  "error"        │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Regex Compile  │
│  (error)        │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Filter Buffer  │
│  (search index) │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Jump to Next   │
│  Match (n key)  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Highlight      │
│  Matches        │
└─────────────────┘
```

## State Management

### Centralized State Pattern

```rust
pub struct AppState {
    // Navigation
    pub current_tab: Tab,
    pub previous_tab: Option<Tab>,
    pub focused_panel: Panel,
    
    // Container data
    pub containers: Vec<ContainerSummary>,
    pub selected_container: Option<String>,
    pub container_filter: Filter,
    
    // Image data
    pub images: Vec<ImageSummary>,
    pub selected_image: Option<String>,
    
    // Metrics
    pub metrics: MetricsStore,
    
    // Logs
    pub log_buffer: LogBuffer,
    pub log_search: Option<SearchState>,
    
    // Compose
    pub compose_projects: Vec<ComposeProject>,
    
    // UI State
    pub show_help: bool,
    pub notifications: Vec<Notification>,
    pub popup: Option<Popup>,
    pub loading: bool,
    
    // Async results
    pub pending_operations: Vec<OperationId>,
}
```

### Immutable Updates

```rust
impl AppState {
    pub fn update_containers(&mut self, containers: Vec<ContainerSummary>) {
        self.containers = containers;
        // Preserve selection if still valid
        if let Some(ref id) = self.selected_container {
            if !self.containers.iter().any(|c| c.id == *id) {
                self.selected_container = self.containers.first().map(|c| c.id.clone());
            }
        }
    }
    
    pub fn with_filter(&self, filter: Filter) -> Self {
        let mut new = self.clone();
        new.container_filter = filter;
        new.apply_filter();
        new
    }
}
```

## Async Architecture

### Task Spawning Strategy

```rust
pub struct BackgroundTasks {
    stats_handle: JoinHandle<()>,
    events_handle: JoinHandle<()>,
    logs_handle: JoinHandle<()>,
    cancellation_token: CancellationToken,
}

impl BackgroundTasks {
    pub async fn spawn(docker: DockerClient, tx: mpsc::Sender<Message>) -> Self {
        let token = CancellationToken::new();
        
        let stats = spawn_stats_worker(
            docker.clone(), 
            tx.clone(), 
            token.child_token()
        );
        
        let events = spawn_event_worker(
            docker.clone(), 
            tx.clone(), 
            token.child_token()
        );
        
        let logs = spawn_log_worker(
            docker.clone(), 
            tx.clone(), 
            token.child_token()
        );
        
        Self {
            stats_handle: stats,
            events_handle: events,
            logs_handle: logs,
            cancellation_token: token,
        }
    }
    
    pub async fn shutdown(self) {
        self.cancellation_token.cancel();
        let _ = tokio::join!(
            self.stats_handle,
            self.events_handle,
            self.logs_handle
        );
    }
}
```

### Message Types

```rust
pub enum Message {
    // Docker events
    ContainerEvent(ContainerEvent),
    ImageEvent(ImageEvent),
    VolumeEvent(VolumeEvent),
    NetworkEvent(NetworkEvent),
    
    // Data updates
    ContainersUpdated(Vec<ContainerSummary>),
    ImagesUpdated(Vec<ImageSummary>),
    StatsUpdated(ContainerId, ContainerStats),
    LogLine(ContainerId, LogLine),
    
    // Operations
    OperationStarted(OperationId, String),
    OperationCompleted(OperationId, Result<(), DockerError>),
    
    // UI
    Notification(Notification),
    Error(String),
}
```

## Error Handling Strategy

### Error Types Hierarchy

```
DockMonError
├── DockerError
│   ├── ConnectionError
│   ├── ApiError { code, message }
│   ├── NotFound { resource }
│   └── Timeout { operation }
├── UiError
│   ├── RenderError
│   ├── InputError
│   └── LayoutError
├── ConfigError
│   ├── ParseError
│   ├── ValidationError
│   └── NotFound
└── IoError
```

### Error Handling Pattern

```rust
pub type Result<T> = std::result::Result<T, DockMonError>;

// In application code
async fn restart_container(&self, id: &str) -> Result<()> {
    self.docker
        .restart_container(id, Some(RestartContainerOptions {
            t: Some(10),
        }))
        .await
        .map_err(|e| match e {
            BollardError::DockerResponseNotFoundError { message, .. } => {
                DockerError::NotFound {
                    resource: format!("container {}", id),
                }.into()
            }
            BollardError::DockerResponseServerError { code, message, .. } => {
                DockerError::ApiError { code, message }.into()
            }
            _ => DockerError::ConnectionError(e.to_string()).into(),
        })?;
    
    Ok(())
}

// UI display
fn show_error(state: &mut AppState, error: DockMonError) {
    let msg = match error {
        DockerError::NotFound { resource } => {
            format!("❌ {} not found", resource)
        }
        DockerError::ApiError { code, message } => {
            format!("❌ Docker error {}: {}", code, message)
        }
        _ => format!("❌ {}", error),
    };
    state.notifications.push(Notification::error(msg));
}
```

## Performance Considerations

### 1. Rendering Optimization

- **Dirty checking**: Only re-render changed widgets
- **Frame skipping**: Skip renders if UI hasn't changed
- **Virtual scrolling**: Render only visible rows for large lists
- **Caching**: Cache formatted strings and computed values

### 2. Memory Management

- **Ring buffers**: Fixed-size buffers for logs and metrics
- **Streaming**: Don't load entire log files into memory
- **Lazy loading**: Load container details on demand
- **Pooling**: Reuse buffer allocations

### 3. Network Optimization

- **Batched requests**: Group API calls when possible
- **Selective updates**: Only fetch changed data
- **Compression**: Use gzip for large responses
- **Connection pooling**: Reuse HTTP connections

## Security Considerations

### 1. Docker Socket Access

- Require user in `docker` group or root (standard Docker security model)
- Support DOCKER_HOST environment variable for remote access
- Never hardcode credentials

### 2. Registry Credentials

- Use Docker's credential store
- Support credential helpers
- Never log sensitive information

### 3. Exec Operations

- Warn before running exec commands
- Log all exec operations for audit
- Support read-only mode

## Testing Architecture

```
tests/
├── unit/
│   ├── state_tests.rs
│   ├── config_tests.rs
│   └── utils_tests.rs
├── integration/
│   ├── docker_client_tests.rs
│   └── ui_tests.rs
├── fixtures/
│   ├── containers.json
│   ├── images.json
│   └── docker-compose.yml
└── mocks/
    ├── mock_docker.rs
    └── mock_terminal.rs
```

## Deployment

### Binary Distribution

```
dockmon/
├── dockmon              # Main binary
├── config/
│   └── default.toml     # Default configuration
└── docs/
    └── README.md
```

### Docker Image

```dockerfile
FROM alpine:latest
RUN apk add --no-cache ca-certificates
COPY dockmon /usr/local/bin/
ENTRYPOINT ["dockmon"]
```

Usage:
```bash
docker run --rm -it \
  -v /var/run/docker.sock:/var/run/docker.sock:ro \
  dockmon:latest
```
