# DockMon UI Design Specification

## Design Philosophy

DockMon follows a **"terminal-first, keyboard-centric"** design philosophy:

1. **Efficiency First**: All common actions accessible via single keystrokes
2. **Visual Clarity**: Clear information hierarchy with consistent visual language
3. **Minimal Chrome**: Maximize content area, minimize decorative elements
4. **Responsive**: Adapt gracefully to different terminal sizes
5. **Discoverable**: Help always one keypress away (`?`)

## Layout System

### Main Layout Structure

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ Header (1 line)                                                              │
│ [Logo] [Status]                    [Context] [Docker Version]      [Time]    │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│ Sidebar      │  Main Content Area                                            │
│ (15-25 cols) │  (remaining width)                                            │
│              │                                                               │
│ ○ Containers │  ┌──────────────────────────────────────────────────────┐    │
│ ● Images     │  │ Tab Bar                                               │    │
│ ○ Volumes    │  │ [Containers] [Images] [Volumes] [Networks] [Compose]  │    │
│ ○ Networks   │  ├──────────────────────────────────────────────────────┤    │
│ ○ Compose    │  │                                                       │    │
│ ○ System     │  │  List Panel              │ Detail Panel              │    │
│              │  │  ┌─────────────────┐     │ ┌───────────────────────┐ │    │
│              │  │  │ ID   Name  Stat │     │ │ Container Details     │ │    │
│              │  │  │ ───  ────  ──── │     │ │ ───────────────────── │ │    │
│              │  │  │ abc  web   ▶    │     │ │ ID: abc123...         │ │    │
│              │  │  │ def  db    ⏸    │     │ │ Image: nginx:latest   │ │    │
│              │  │  │ ghi  cache ■    │     │ │ Status: Running       │ │    │
│              │  │  │                 │     │ │ CPU: 12% | Mem: 45MB  │ │    │
│              │  │  │                 │     │ │                       │ │    │
│              │  │  │                 │     │ │ [Logs] [Stats] [Top]  │ │    │
│              │  │  └─────────────────┘     │ └───────────────────────┘ │    │
│              │  │                           │                           │    │
│              │  └──────────────────────────────────────────────────────┘    │
│                                                                              │
├──────────────────────────────────────────────────────────────────────────────┤
│ Status Bar (1-2 lines)                                                       │
│ [Tab: Switch] [Enter: Select] [s: Start/Stop] [r: Restart] [?: Help]        │
│ ⚠️ 2 containers unhealthy | 📊 CPU: 45% | 💾 Mem: 2.1GB / 8GB                │
└──────────────────────────────────────────────────────────────────────────────┘
```

### Responsive Breakpoints

| Width | Layout Adaptation |
|-------|------------------|
| < 80 cols | Hide sidebar, use Tab for navigation |
| 80-120 cols | Compact sidebar (icons only), single panel view |
| 120-160 cols | Full sidebar, split view (list + detail) |
| > 160 cols | Full sidebar, three-pane view (list + detail + logs) |

### Height Considerations

| Height | Layout Adaptation |
|--------|------------------|
| < 20 rows | Single panel, minimal header, scrollable only |
| 20-30 rows | Standard layout with compact status bar |
| > 30 rows | Expanded layout with full status bar and notifications |

## Component Specifications

### 1. Header

**Height**: 1 line
**Content**:
- Left: Logo/brand ("🐳 DockMon")
- Center-left: Docker connection status (● Connected / ○ Disconnected)
- Center: Current Docker context
- Center-right: Docker version
- Right: Current time (optional)

**Colors**:
- Connected: Green (●)
- Disconnected: Red (○)
- Background: Dark blue/cyan highlight

### 2. Navigation Sidebar

**Width**: 15-25 columns (configurable)
**Content**:
```
Navigation
──────────
▶ Containers    12▶ 3⏸
  Images        45
  Volumes       8
  Networks      5
  Compose       2 projects
  System
```

**Indicators**:
- `▶` - Currently selected tab
- `●` - Active/Running items count
- `⏸` - Paused items count
- `⚠` - Warning/Error items count

**Keybindings**:
- `1-6` or `Ctrl+h/l` - Switch tabs
- `Tab` - Focus to main content

### 3. Tab Bar

**Height**: 1 line
**Style**: Underline active tab

```
[Containers] [Images] [Volumes] [Networks] [Compose] [System]
 ──────────
```

**Colors**:
- Active tab: White text on blue background
- Inactive tabs: Gray text
- Separator: Dark gray `│`

### 4. List Panel (Containers Example)

**Columns**:
| Column | Width | Description |
|--------|-------|-------------|
| Status | 2 | Icon indicator |
| ID | 12 | Short ID |
| Name | 20 | Container name |
| Image | 25 | Image name:tag |
| Status | 12 | Running/Exited/etc |
| CPU% | 6 | CPU percentage |
| Mem | 10 | Memory usage |
| Ports | 30 | Port mappings |

**Row Format**:
```
▶ abc123 nginx web ▶ Running  12.5%  45MB  0.0.0.0:80->80/tcp
  def456 postgres db ▏ Exited   0.0%   0B   5432/tcp
```

**Selection**:
- Highlighted row: Reverse video or background color
- Cursor: `▶` at row start

**Sorting**:
- Click column header or press number key (1-9)
- Secondary sort by name
- Indicator: `▼` descending, `▲` ascending

### 5. Detail Panel

**Structure**:
```
┌─ Container: web ─────────────────────────────┐
│ ID:           abc123def456                   │
│ Image:        nginx:latest                   │
│ Created:      2024-01-15 10:30:45            │
│ Status:       ▶ Running (2h 15m)             │
│ Health:       ✓ Healthy                      │
│                                              │
│ Resources:                                   │
│   CPU:    ████████████░░░░░░░░  45%          │
│   Memory: █████░░░░░░░░░░░░░░░  45MB/100MB   │
│   PIDs:   23                                 │
│                                              │
│ Ports:                                       │
│   0.0.0.0:80 -> 80/tcp                       │
│   0.0.0.0:443 -> 443/tcp                     │
│                                              │
│ [Actions: s:Stop r:Restart x:Exec d:Remove]  │
└──────────────────────────────────────────────┘
```

**Sub-panels** (switch with Tab or number keys):
- Info (default)
- Logs
- Stats (charts)
- Top (processes)
- Inspect (JSON)

### 6. Log Viewer

**Structure**:
```
┌─ Logs: web ──────────────────────────────────┐[Search: error_][🔍][×]
│ 2024-01-15 10:30:45 [INFO] Server started     │
│ 2024-01-15 10:30:46 [INFO] Listening on :80   │
│▌2024-01-15 10:31:12 [WARN] Slow request       │
│ 2024-01-15 10:31:15 [ERROR] Connection failed │
│ 2024-01-15 10:31:16 [INFO] Retry attempt 1    │
│                                              │
│ [Follow: ON] [Lines: 1000] [Level: All]      │
└──────────────────────────────────────────────┘
```

**Features**:
- Color-coded log levels (INFO: white, WARN: yellow, ERROR: red)
- Search bar at top (activated with `/`)
- Status bar at bottom
- Scrollbar indicator
- Timestamp toggle (`t`)
- Word wrap toggle (`w`)

**Log Level Colors**:
| Level | Color | BG |
|-------|-------|-----|
| TRACE | Gray | Default |
| DEBUG | Cyan | Default |
| INFO | White | Default |
| WARN | Yellow | Default |
| ERROR | Red | Default |
| FATAL | White | Red |

### 7. Stats Charts

**CPU Chart** (Braille patterns):
```
CPU Usage (last 5 minutes)
100%│        ⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀
 75%│    ⣀⡠⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⢄
 50%│⣀⡠⠊          ⢀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⠑
 25%│⠊                   ⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉
  0%└────────────────────────────────────────────────────────────────────────
    10:30    10:32    10:34    10:36    10:38    10:40
```

**Memory Chart** (Block characters):
```
Memory Usage
┌─────────────────────────────────────────────────────────────────────────┐
│████████████████████████████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░│ 56%
│████████████████████████████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░│
│██████████████████████████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░│
│██████████████████████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░│
│██████████████████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░│
└─────────────────────────────────────────────────────────────────────────┘
 0MB                                                                    512MB
```

**Network I/O** (Dual chart):
```
Network I/O
    ▲ RX                                    TX ▲
100K│                ╭─╮                   │100K│
 50K│     ╭─╮       │ │ ╭─╮      ╭─╮      │ 50K│
  0K├─────┴─┴───────┴─┴─┴─┴──────┴─┴──────┤  0K├
    └──────────────────────────────────────┘    └
```

### 8. Status Bar

**Format**:
```
[Keybindings] | [Status Icons] [System Stats]
```

**Example**:
```
[Tab:Switch ↑↓:Navigate Enter:Select s:Start/Stop r:Restart x:Exec d:Delete ?:Help] 
│ ⚠️ 2 unhealthy │ 📊 45% CPU │ 💾 2.1GB/8GB │ 🐳 v24.0.7 │ 14:32:18
```

**Status Icons**:
| Icon | Meaning |
|------|---------|
| ⚠️ | Containers with issues |
| 🔔 | Notifications |
| 📊 | System stats |
| 💾 | Memory usage |
| 🐳 | Docker version |
| ⏳ | Pending operations |

### 9. Help Overlay

**Layout**:
```
┌─ Help ──────────────────────────────────────────────────────────────────┐
│                                                                         │
│  Global                                                                  │
│    q         Quit application                                            │
│    ?         Toggle help                                                 │
│    1-6       Switch tabs                                                 │
│    Tab       Next panel                                                  │
│    Shift+Tab Previous panel                                              │
│                                                                         │
│  Containers                                                              │
│    s         Start/Stop container                                        │
│    r         Restart container                                           │
│    p         Pause/Unpause container                                     │
│    k         Kill container                                              │
│    d         Remove container                                            │
│    x         Exec into container                                         │
│    l         View logs                                                   │
│    /         Filter list                                                 │
│    Space     Select for bulk operation                                   │
│                                                                         │
│  Logs                                                                      │
│    /         Search                                                      │
│    n         Next match                                                  │
│    N         Previous match                                              │
│    t         Toggle timestamps                                           │
│    f         Follow logs (auto-scroll)                                   │
│    S         Save to file                                                │
│                                                                         │
│  Press any key to close                                                  │
└─────────────────────────────────────────────────────────────────────────┘
```

### 10. Confirmation Dialog

```
┌─ Confirm ──────────────────────┐
│                                │
│  ⚠️  Remove container "web"?    │
│                                │
│  This action cannot be undone. │
│                                │
│     [Cancel]  [Remove]         │
│          ↑                      │
│     (Tab to switch, Enter to   │
│      confirm)                   │
└────────────────────────────────┘
```

## Color Schemes

### Dark Theme (Default)

```toml
[colors]
background = "#1e1e1e"
foreground = "#d4d4d4"
selection = "#264f78"
border = "#3c3c3c"
accent = "#007acc"

# Status colors
success = "#4ec9b0"
warning = "#cca700"
error = "#f48771"
info = "#75beff"

# Container states
running = "#4ec9b0"
stopped = "#808080"
paused = "#cca700"
restarting = "#75beff"
unhealthy = "#f48771"
healthy = "#4ec9b0"

# Resource usage
low = "#4ec9b0"
medium = "#cca700"
high = "#f48771"
```

### Light Theme

```toml
[colors]
background = "#ffffff"
foreground = "#3c3c3c"
selection = "#add6ff"
border = "#e5e5e5"
accent = "#0078d4"

# Status colors (adjusted for light bg)
success = "#107c10"
warning = "#ffc107"
error = "#e81123"
info = "#0078d4"
```

### High Contrast Theme

```toml
[colors]
background = "#000000"
foreground = "#ffffff"
selection = "#008000"
border = "#ffffff"
accent = "#ffff00"

# Maximum contrast for accessibility
success = "#00ff00"
warning = "#ffff00"
error = "#ff0000"
info = "#00ffff"
```

## Animations & Transitions

### Smooth Scrolling
- Use terminal's native smooth scroll when available
- Fallback to line-by-line scrolling

### Progress Indicators
```
Loading... [⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏]  (spinner)
Pulling image... [████████░░░░░░░░░░░░] 45%  (progress bar)
```

### Notification Toast
```
┌────────────────────────┐
│ ✅ Container started   │
│    web is now running  │
└────────────────────────┘
```
Appears in bottom-right, auto-dismisses after 3 seconds.

## Responsive Layout Examples

### Full Width (>160 cols)
```
┌────┬─────────────┬──────────────────────────────────────────────────────────┐
│Nav │  List       │  Detail                    │  Logs                       │
│    │             │                            │                             │
└────┴─────────────┴────────────────────────────┴─────────────────────────────┘
```

### Medium Width (120-160 cols)
```
┌────┬────────────────────────────────────────────────────────────────────────┐
│Nav │  List                       │  Detail/Logs (toggle)                    │
│    │                             │                                          │
└────┴─────────────────────────────┴──────────────────────────────────────────┘
```

### Narrow Width (80-120 cols)
```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Tabs: [Containers] [Images] [Volumes]...                                   │
│                                                                             │
│  List Panel                                                                 │
│  (detail in popup/modal)                                                    │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Mobile/Small (<80 cols)
```
┌─────────────────────────────────────────────────┐
│ [◀] Containers [▶]                             │
│                                                 │
│ ▶ web                    Running               │
│   db                     Exited                │
│                                                 │
│ [Select] [Start] [Logs] [Remove]               │
└─────────────────────────────────────────────────┘
```

## Mouse Support

### Click Actions
| Element | Action |
|---------|--------|
| Sidebar item | Switch to that tab |
| Tab | Switch to that view |
| Column header | Sort by column |
| List row | Select item |
| Button | Trigger action |
| Scrollbar | Scroll view |
| Split divider | Resize panels (if draggable) |

### Right-Click Context Menu
```
┌─ Actions ──────────────┐
│ View Logs              │
│ View Stats             │
│ Restart                │
│ Stop                   │
│ ─────────────────────  │
│ Copy ID                │
│ Inspect                │
└────────────────────────┘
```

## Unicode Icons Reference

| Symbol | Unicode | Usage |
|--------|---------|-------|
| ▶ | U+25B6 | Running / Selected / Start |
| ⏸ | U+23F8 | Paused |
| ■ | U+25A0 | Stopped |
| ● | U+25CF | Active/Healthy |
| ○ | U+25CB | Inactive |
| ⚠ | U+26A0 | Warning |
| ✓ | U+2713 | Healthy/Success |
| ✗ | U+2717 | Unhealthy/Error |
| 🐳 | U+1F433 | Docker/Brand |
| 📊 | U+1F4CA | Stats |
| 💾 | U+1F4BE | Storage |
| 🔔 | U+1F514 | Notification |
