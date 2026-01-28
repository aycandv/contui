# PRD: DockMon - Advanced Docker TUI

## Introduction

DockMon is a comprehensive, Rust-based Terminal User Interface (TUI) for Docker management that combines the power of existing tools like lazydocker and oxker with advanced monitoring capabilities, intelligent log analysis, and seamless container registry integration. Designed for developers and DevOps engineers who demand both high-level oversight and low-level control, DockMon provides real-time visibility into containers, images, volumes, networks, and Docker Compose stacks—all from the terminal.

The tool bridges the gap between simple Docker CLIs and heavy web-based GUIs like Portainer, offering a keyboard-driven, resource-efficient interface that excels in local development, remote SSH sessions, and production troubleshooting scenarios.

## Goals

- **Comprehensive Management**: Provide full CRUD operations for containers, images, volumes, networks, and Docker Compose stacks
- **Real-time Monitoring**: Deliver real-time and historical resource metrics (CPU, memory, network, disk I/O) with visual charts
- **Intelligent Logging**: Offer advanced log aggregation, filtering, search, and multi-container log streaming
- **Registry Integration**: Enable browsing, searching, and pulling images directly from Docker Hub and custom registries
- **Developer Experience**: Achieve sub-100ms response times, vim-style keybindings, customizable configuration, and zero external dependencies for basic operation
- **Operational Excellence**: Support alerting on resource thresholds, export capabilities, and multi-context management

## User Stories

### US-001: Container Lifecycle Management
**Description:** As a developer, I want to view and manage all my Docker containers so that I can start, stop, restart, pause, unpause, kill, and remove containers without memorizing CLI commands.

**Acceptance Criteria:**
- [ ] Display all containers (running and stopped) in a sortable, filterable table
- [ ] Show container status, name, image, ports, uptime, and health status
- [ ] Support keyboard shortcuts: `s` (start/stop), `r` (restart), `p` (pause/unpause), `k` (kill), `d` (remove), `x` (exec)
- [ ] Bulk operations: select multiple containers with spacebar and perform actions
- [ ] Confirmation dialogs for destructive actions (remove, kill)
- [ ] View container details: ID, creation time, labels, environment variables, mounts
- [ ] Typecheck/lint passes

### US-002: Real-time Container Monitoring
**Description:** As a DevOps engineer, I want to see real-time resource usage for containers with historical data so that I can identify performance bottlenecks and resource hogs.

**Acceptance Criteria:**
- [ ] Display live CPU usage percentage per container with ASCII/Unicode bar charts
- [ ] Display live memory usage (used/total/limit) with visual indicators
- [ ] Display network I/O (RX/TX bytes and packets)
- [ ] Display block I/O (read/write bytes)
- [ ] Show PIDs count per container
- [ ] Historical data: Store last 1 hour of metrics in memory (configurable)
- [ ] Time-series charts using braille patterns or block characters
- [ ] Export metrics to CSV/JSON for external analysis
- [ ] Alert when containers exceed configurable CPU/memory thresholds
- [ ] Typecheck/lint passes

### US-003: Advanced Log Management
**Description:** As a developer debugging microservices, I want to view, search, and filter container logs across multiple containers so that I can correlate events and find issues faster.

**Acceptance Criteria:**
- [ ] Real-time log streaming with configurable tail lines (default: 1000)
- [ ] Search logs with `/` keybinding (regex support)
- [ ] Filter by log level (INFO, WARN, ERROR, DEBUG) when structured logs detected
- [ ] Multi-container log aggregation: view logs from selected containers interleaved
- [ ] Timestamp display with configurable timezone
- [ ] Color-coded log levels (auto-detect JSON structured logs)
- [ ] Export logs to file (`S` keybinding)
- [ ] Log following toggle (auto-scroll vs manual scroll)
- [ ] Highlight search matches in log view
- [ ] Typecheck/lint passes

### US-004: Docker Compose Stack Management
**Description:** As a developer using Docker Compose, I want to manage my entire stack including services, networks, and volumes defined in compose files so that I can orchestrate multi-container applications.

**Acceptance Criteria:**
- [ ] Auto-detect docker-compose.yml files in current directory and subdirectories
- [ ] Display stacks with their services, showing service state
- [ ] Support docker-compose operations: up, down, pull, build, restart
- [ ] View merged compose configuration (all extends/includes resolved)
- [ ] Scale services up/down (`+`/`-` keybindings)
- [ ] View service logs (individual and aggregated)
- [ ] Show stack-level resource usage aggregated by service
- [ ] Support multiple compose files (-f flag equivalent)
- [ ] Environment file (.env) parsing and display
- [ ] Typecheck/lint passes

### US-005: Image Management and Registry Integration
**Description:** As a developer, I want to browse, search, pull, and manage Docker images including integration with Docker Hub and custom registries so that I can manage my image inventory efficiently.

**Acceptance Criteria:**
- [ ] List local images with tags, size, creation date, and usage status
- [ ] Remove images (`d` keybinding) with layer dependency checking
- [ ] Prune unused images (`p` keybinding)
- [ ] Search Docker Hub from TUI (`/` keybinding)
- [ ] Pull images with progress indication
- [ ] Show image history/layers (like `docker history`)
- [ ] Show image vulnerabilities if Docker Scout is available
- [ ] Tag images (`t` keybinding)
- [ ] Push images to registries
- [ ] Display image size breakdown by layer
- [ ] Support private registries with authentication
- [ ] Typecheck/lint passes

### US-006: Volume and Network Management
**Description:** As a system administrator, I want to manage Docker volumes and networks so that I can inspect, clean up, and troubleshoot storage and connectivity issues.

**Acceptance Criteria:**
- [ ] List volumes with name, driver, size, and mountpoint
- [ ] Show which containers are using each volume
- [ ] Inspect volume contents (browse files if running as privileged)
- [ ] Remove volumes (`d` keybinding) with usage checking
- [ ] Prune unused volumes (`p` keybinding)
- [ ] List networks with driver, subnet, gateway, and scope
- [ ] Show connected containers per network
- [ ] Display network traffic statistics
- [ ] Create/remove networks
- [ ] Typecheck/lint passes

### US-007: Exec and Debugging
**Description:** As a developer debugging containers, I want to execute commands inside running containers and inspect their processes so that I can troubleshoot issues interactively.

**Acceptance Criteria:**
- [ ] Exec into container with custom command or default shell (`x` keybinding)
- [ ] Quick exec with predefined common commands (sh, bash, /bin/sh)
- [ ] View container processes (`Top` panel with ps-like output)
- [ ] View container resource limits and current usage
- [ ] Inspect container configuration (JSON view)
- [ ] View container filesystem changes (diff)
- [ ] Copy files to/from containers (cp command wrapper)
- [ ] Port mapping visualization
- [ ] Typecheck/lint passes

### US-008: Monitoring Dashboard and Alerts
**Description:** As an operations engineer, I want a system-wide dashboard with alerting capabilities so that I can monitor the health of my Docker host and receive notifications on issues.

**Acceptance Criteria:**
- [ ] System-wide dashboard showing: total containers (running/stopped), image count, volume usage, network count
- [ ] Host system metrics: Docker daemon info, storage driver, kernel version
- [ ] Disk usage visualization for Docker data directory
- [ ] Configurable alerts for:
  - Container exit/crash events
  - High resource usage (CPU > threshold, Memory > threshold)
  - Image vulnerability detection
  - Unhealthy container status
- [ ] Alert history log
- [ ] Export alert configuration
- [ ] Typecheck/lint passes

### US-009: Customization and Configuration
**Description:** As a power user, I want to customize keybindings, themes, and behavior so that DockMon fits my workflow and preferences.

**Acceptance Criteria:**
- [ ] Configuration file support (TOML/YAML) at `~/.config/dockmon/config.toml`
- [ ] Customizable keybindings for all actions
- [ ] Multiple color themes (dark, light, high-contrast, customizable)
- [ ] Configurable default views and panels
- [ ] Custom command templates (user-defined docker commands)
- [ ] Export/import configuration
- [ ] Per-project configuration files (`.dockmon.toml`)
- [ ] Typecheck/lint passes

### US-010: Search and Filtering
**Description:** As a user with many containers, I want powerful search and filtering capabilities so that I can quickly find specific resources.

**Acceptance Criteria:**
- [ ] Global search across all resource types (`/` keybinding)
- [ ] Filter containers by: status, image, name, label
- [ ] Filter images by: tag, size, age, dangling status
- [ ] Save and recall filter presets
- [ ] Fuzzy search matching
- [ ] Regular expression support for advanced filtering
- [ ] Typecheck/lint passes

## Functional Requirements

### Core Infrastructure

**FR-1:** The application must be written in Rust using ratatui for the TUI and crossterm for terminal control.

**FR-2:** The application must communicate with Docker via the Bollard crate (official Docker API client).

**FR-3:** The application must support Docker API version 1.41+ (Docker Engine 20.10+).

**FR-4:** The application must gracefully handle Docker daemon connection loss and retry automatically.

**FR-5:** The application must support both Unix sockets (`/var/run/docker.sock`) and TCP connections to Docker daemon.

**FR-6:** The application must have a modular architecture separating UI, business logic, and Docker API communication.

### Container Management

**FR-7:** Display container list with columns: ID (short), Name, Image, Status, Ports, CPU %, Memory %, Uptime, Health.

**FR-8:** Sort containers by any column using number keys (1-9) or clicking headers.

**FR-9:** Auto-refresh container list every 1 second (configurable).

**FR-10:** Show container logs with configurable tail (100, 500, 1000, 5000, all).

**FR-11:** Support following logs (auto-scroll) and pausing follow.

**FR-12:** Container actions: Start, Stop, Restart, Pause, Unpause, Kill, Remove, Rename.

**FR-13:** Support force removal and volume removal flags.

**FR-14:** Display container inspection JSON in collapsible tree view.

### Image Management

**FR-15:** Display image list with: Repository, Tag, ID, Created, Size, Used By.

**FR-16:** Distinguish between dangling and used images visually.

**FR-17:** Show image layer history with cumulative sizes.

**FR-18:** Support docker build from Dockerfile with context selection.

**FR-19:** Integration with Docker Hub search API and custom registries.

**FR-20:** Pull images with streaming progress display.

**FR-21:** Tag and push images to registries.

### Docker Compose

**FR-22:** Parse docker-compose.yml and docker-compose.yaml files.

**FR-23:** Display services defined in compose files with their configurations.

**FR-24:** Execute compose commands: up, down, pull, build, config.

**FR-25:** Support service scaling operations.

**FR-26:** Display compose project networks and volumes.

**FR-27:** Show environment variable interpolation from .env files.

### Monitoring and Metrics

**FR-28:** Collect container stats every 1 second using Docker Stats API.

**FR-29:** Store historical metrics in memory with configurable retention (default: 1 hour).

**FR-30:** Render time-series charts using braille characters or Unicode blocks.

**FR-31:** Calculate and display rates (CPU %, network throughput).

**FR-32:** Support cumulative and instantaneous metrics views.

**FR-33:** Export metrics data to CSV and JSON formats.

### Log Management

**FR-34:** Stream logs from Docker Logs API with timestamps.

**FR-35:** Parse and colorize JSON-formatted logs automatically.

**FR-36:** Implement log search with regex support and result highlighting.

**FR-37:** Support multi-container log aggregation with container name prefix.

**FR-38:** Implement log filtering by time range.

**FR-39:** Save logs to file with configurable naming patterns.

### User Interface

**FR-40:** Use vim-style navigation (h/j/k/l or arrow keys).

**FR-41:** Support mouse interactions (click, scroll, resize).

**FR-42:** Implement tab-based navigation between: Containers, Images, Volumes, Networks, Compose, System.

**FR-43:** Provide context-aware help panel (? keybinding).

**FR-44:** Support split-pane views (e.g., container list + logs side by side).

**FR-45:** Implement responsive layout adapting to terminal size changes.

### Configuration

**FR-46:** Configuration file at `~/.config/dockmon/config.toml`.

**FR-47:** Support for theme customization including colors for different states.

**FR-48:** Customizable keybinding definitions.

**FR-49:** Per-project configuration override files.

**FR-50:** CLI arguments override config file settings.

## Non-Goals (Out of Scope)

The following features are explicitly out of scope for the initial release but may be considered for future versions:

- **Multi-host management**: Managing Docker Swarm or multiple remote hosts simultaneously
- **Kubernetes support**: This is a Docker-focused tool; use k9s for Kubernetes
- **Podman support**: Initial version targets Docker only
- **Web UI**: This is a terminal-first application
- **Container orchestration**: No built-in orchestration features beyond Docker Compose
- **Image building from scratch**: We support `docker build` but not custom build engines
- **Registry management**: Browse and pull only; no registry administration
- **Backup/restore**: No built-in volume backup capabilities (use dedicated tools)
- **CI/CD integration**: Not designed for pipeline usage
- **Windows native support**: Initial target is Linux/macOS; Windows via WSL

## Design Considerations

### UI Layout

```
┌─────────────────────────────────────────────────────────────┐
│ Header: Docker Status | Host Info | Current Context          │
├──────────────────┬──────────────────────────────────────────┤
│                  │                                          │
│  Resource List   │     Details / Logs / Stats Panel         │
│  (Containers/    │                                          │
│   Images/etc)    │  - Log viewer with search                │
│                  │  - Real-time charts                      │
│  [Filter input]  │  - JSON inspector                        │
│                  │  - Process list                          │
│                  │                                          │
├──────────────────┴──────────────────────────────────────────┤
│ Status Bar: Help | Keybindings | Notifications               │
└─────────────────────────────────────────────────────────────┘
```

### Keybinding Philosophy

- Single-key actions for common operations (s, r, d, x)
- Modifier keys for destructive actions (Ctrl+d for force remove)
- Vim-style navigation (hjkl, gg, G)
- Consistent patterns across all panels
- Context-sensitive help always available

### Color Scheme

- **Running containers**: Green
- **Stopped containers**: Red/Gray
- **Paused containers**: Yellow
- **Healthy**: Green checkmark
- **Unhealthy**: Red X
- **Warning states**: Orange/Yellow
- **Information**: Blue/Cyan
- **Selection highlight**: Inverted/reverse

### Performance Targets

- Initial load time: < 500ms
- UI refresh rate: 60 FPS
- Stats collection interval: 1 second
- Log streaming latency: < 100ms
- Memory usage: < 100MB for 100 containers

## Technical Considerations

### Architecture

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│     UI      │────▶│  App State  │────▶│   Docker    │
│  (Ratatui)  │◀────│   (Model)   │◀────│   (Bollard) │
└─────────────┘     └─────────────┘     └─────────────┘
       ▲                    │
       │              ┌─────────────┐
       └──────────────│   Config    │
                      │   (TOML)    │
                      └─────────────┘
```

### Key Dependencies

- **ratatui**: TUI framework (widgets, layout, rendering)
- **crossterm**: Cross-platform terminal control (input, screen)
- **bollard**: Docker API client
- **tokio**: Async runtime
- **serde + toml**: Configuration serialization
- **chrono**: Date/time handling
- **regex**: Log searching and filtering
- **unicode-width**: Proper text width calculation

### Async Strategy

- Use tokio for all async operations
- Spawn separate tasks for:
  - Stats collection loop
  - Log streaming
  - Event handling (keyboard/mouse)
  - Docker API calls
- Use channels (mpsc) for communication between tasks
- Implement cancellation tokens for clean shutdown

### Data Flow

1. **Main Loop**: Handle user input, update app state
2. **Render Loop**: Draw UI based on current state (60 FPS target)
3. **Docker Events Loop**: Stream Docker events (container start/stop/die)
4. **Stats Loop**: Collect metrics periodically
5. **Log Loop**: Stream logs for selected containers

### Error Handling

- Graceful degradation when Docker is unavailable
- User-friendly error messages in notification area
- Retry logic with exponential backoff for API calls
- Log errors to file for debugging

## Success Metrics

- **Performance**: UI remains responsive with 100+ containers
- **Adoption**: Target feature parity with lazydocker within 6 months
- **Stability**: < 1 crash per 100 hours of usage
- **User Satisfaction**: Ability to perform 90% of daily Docker tasks without leaving the TUI
- **Resource Efficiency**: Memory footprint < 100MB under normal load

## Open Questions

1. Should we implement a plugin system for custom extensions?
2. How should we handle Docker contexts (docker context use)?
3. Should we support docker-in-docker scenarios?
4. What's the best approach for handling very large log streams (>100MB)?
5. Should we include a built-in tutorial/onboarding for first-time users?
6. How do we handle secrets (registry credentials) securely?

## Milestones

### MVP (v0.1.0)
- Container list and basic lifecycle management
- Basic log viewing
- Image list and removal
- Simple stats display

### v0.2.0
- Docker Compose support
- Advanced filtering and search
- Volume and network management
- Configuration file support

### v0.3.0
- Real-time charts and historical data
- Advanced log features (search, aggregation)
- Registry integration
- Custom keybindings

### v1.0.0
- Full feature parity with lazydocker
- Alerting system
- Theme customization
- Comprehensive documentation

---

*This PRD is a living document and will be updated as the project evolves.*
