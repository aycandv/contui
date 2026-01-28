# DockMon Technical Documentation

This directory contains comprehensive technical documentation for DockMon - an advanced Docker TUI written in Rust.

## Documents

### 1. PRD (Product Requirements Document)
**Location**: `/tasks/prd-docker-tui.md`

The main Product Requirements Document containing:
- Project introduction and goals
- 10 detailed User Stories with acceptance criteria
- 50+ Functional Requirements
- Non-Goals and scope boundaries
- Design considerations and UI layout
- Technical considerations
- Success metrics and milestones

### 2. Architecture Design
**Location**: `/docs/ARCHITECTURE.md`

High-level and detailed architecture covering:
- Three-layer architecture (Presentation, Application, Infrastructure)
- Module structure and file organization
- Component interaction diagrams
- Data flow diagrams
- State management patterns
- Async task architecture
- Error handling strategy
- Performance considerations
- Security considerations
- Testing architecture

### 3. Data Models
**Location**: `/docs/DATA_MODELS.md`

Complete data model specifications:
- Container types (summary, details, config, state)
- Image types (summary, details, history)
- Volume and Network types
- Docker Compose types (projects, services)
- Metrics and Statistics (CPU, memory, network, block I/O)
- Log types (LogLine, LogBuffer, SearchState)
- Registry types (Docker Hub, custom registries)
- Configuration types
- UI State types
- Type conversions from Bollard types
- Validation traits

### 4. UI Design Specification
**Location**: `/docs/UI_DESIGN.md`

User interface design covering:
- Design philosophy
- Layout system and responsive breakpoints
- Component specifications (Header, Sidebar, Tabs, Lists, Panels)
- Log viewer design
- Stats charts (braille patterns, block characters)
- Color schemes (Dark, Light, High Contrast)
- Animations and transitions
- Unicode icons reference
- Mouse support

### 5. API Integration Guide
**Location**: `/docs/API_INTEGRATION.md`

Docker API integration details:
- Docker client connection management
- Container operations (CRUD, lifecycle)
- Image operations (list, pull, push, build)
- Volume and Network operations
- Event streaming
- Docker Compose integration
- Registry integration (Docker Hub, custom)
- Error handling
- Rate limiting and caching

### 6. Testing Strategy
**Location**: `/docs/TESTING.md`

Comprehensive testing approach:
- Automated testing capabilities and limitations
- Unit test patterns with examples
- UI component testing with TestBackend
- Integration tests for Docker operations
- Mocking strategies for Docker client
- CI/CD workflow configuration
- Performance testing
- Coverage goals

### 6. Ralph PRD (Autonomous Agent Format)
**Location**: `/ralph/prd.json`

Ralph-compatible PRD with 25 user stories:
- Ordered by dependency (foundation → features)
- Each story sized for single context window completion
- Verifiable acceptance criteria
- Priority ordering for implementation

## Quick Start

1. **For understanding the project**: Start with `/tasks/prd-docker-tui.md`
2. **For implementation**: Use `/ralph/prd.json` with Ralph agent
3. **For architecture decisions**: Reference `/docs/ARCHITECTURE.md`
4. **For data structures**: Check `/docs/DATA_MODELS.md`
5. **For UI implementation**: Follow `/docs/UI_DESIGN.md`
6. **For API integration**: Use `/docs/API_INTEGRATION.md`
7. **For testing**: Reference `/docs/TESTING.md`

## Project Structure

```
docker-tui/
├── tasks/
│   └── prd-docker-tui.md       # Main PRD
├── docs/
│   ├── README.md               # This file
│   ├── ARCHITECTURE.md         # Architecture design
│   ├── DATA_MODELS.md          # Data model specifications
│   ├── UI_DESIGN.md            # UI design specification
│   └── API_INTEGRATION.md      # API integration guide
├── ralph/
│   └── prd.json                # Ralph agent format
└── src/                        # Implementation (to be created)
```

## Technology Stack

- **Language**: Rust
- **TUI Framework**: ratatui
- **Terminal Backend**: crossterm
- **Docker API**: bollard
- **Async Runtime**: tokio
- **Serialization**: serde + toml
- **Configuration**: TOML files

## Key Features (from PRD)

1. **Container Management**: Full lifecycle control with real-time updates
2. **Monitoring**: CPU, memory, network, disk I/O with historical charts
3. **Advanced Logging**: Multi-container aggregation, search, filtering
4. **Docker Compose**: Full stack management
5. **Registry Integration**: Docker Hub search and image pulling
6. **Customization**: Keybindings, themes, configuration files

## Implementation Order (Ralph)

The Ralph PRD contains 25 user stories ordered by dependency:

1. Project setup and dependencies (US-001)
2. Core types and error handling (US-002)
3. Docker client wrapper (US-003)
4. Container list and operations (US-004)
5. Basic TUI layout (US-005)
6. Container list widget (US-006)
7. Container detail panel (US-007)
8. Container actions (US-008)
9. Image management (US-009)
10. Volume and Network management (US-010)
11. Real-time log streaming (US-011)
12. Log search and filtering (US-012)
13. Container stats (US-013)
14. Docker Compose support (US-014)
15. Configuration system (US-015)
16. Customizable keybindings (US-016)
17. Theme customization (US-017)
18. Help panel (US-018)
19. Registry integration (US-019)
20. Exec into containers (US-020)
21. Filtering and search (US-021)
22. Notifications (US-022)
23. System dashboard (US-023)
24. Mouse support (US-024)
25. Release packaging (US-025)

## Contributing

When implementing:
1. Follow the Ralph PRD story order
2. Reference architecture documents for patterns
3. Use data models for type definitions
4. Follow UI design for consistent styling
5. Implement API patterns as shown in integration guide
6. **Follow git workflow** documented in `GIT_WORKFLOW.md`

## Git Workflow

See [`GIT_WORKFLOW.md`](GIT_WORKFLOW.md) for:
- Branch naming conventions (`ralph/US-XXX-description`)
- Commit message conventions (Conventional Commits)
- Pre-commit hooks setup
- Story completion checklist
- Release workflow

### Quick Start

```bash
# Initialize git
git init

# Create branch for first story
git checkout -b ralph/US-001-project-setup

# Work on story, then commit with conventional commit
git commit -m "chore(project): initial project setup

Set up Cargo.toml with dependencies and basic project structure.

Relates to: US-001"

# Merge to main when complete
git checkout main
git merge --no-ff ralph/US-001-project-setup
```

## License

TBD - Add appropriate license
