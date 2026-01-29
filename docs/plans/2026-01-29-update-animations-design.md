# Update & Install Animations Design

## Overview

Add terminal animations to the update check, download/install, and first-time install script flows. Goal: make the experience feel polished, informative, and playful.

## Style Guidelines

- **Spinner**: Braille dots (`⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏`) at ~80ms per frame
- **Colors**: Cyan for in-progress, green for success, yellow for warnings, red for errors
- **Delays**: 500ms between phases for visibility
- **Progress bar**: Show percentage, bytes transferred, speed, ETA
- **Borders**: Box-drawing characters for important messages

## Phase 1: Update Check (`src/update/mod.rs`)

### During Check
```
  ⠋ Checking for updates...    (cyan)
```

### Update Available
```
  ✨ Update found!                              (green, bold)

  ┌───────────────────────────────────────────┐
  │                                           │
  │   v0.5.0  ━━━━━━━━━━▶  v0.5.1            │
  │                                           │
  │   📦 What's new:                          │
  │   github.com/aycandv/contui/releases      │
  │                                           │
  └───────────────────────────────────────────┘

  🚀 Install now? [Y/n/s] _
```

### Already Up to Date
```
  ✅ Already on latest (v0.5.1)                (green)
```

### Network Error
```
  ⚠️  Update check skipped (offline?)          (dim yellow)
```

## Phase 2: Download & Install

### Downloading
```
  📥 Downloading v0.5.1...                     (cyan)

  [████████████░░░░░░░░░░░░░░] 45%             (green bar)
   3.2 MB / 7.1 MB   •   1.8 MB/s   •   ETA 2s (dim)
```

### Download Complete
```
  ✓ Downloaded v0.5.1 (7.1 MB)                 (green)
```

### Installing (staged spinners with 500ms delays)
```
  ⠋ Extracting archive...                      (yellow)
  ✓ Extracted

  ⠋ Replacing binary...                        (yellow)
  ✓ Replaced

  ⠋ Verifying installation...                  (yellow)
  ✓ Verified
```

### Success
```
  ┌───────────────────────────────────────────┐
  │                                           │
  │   ✅ Successfully updated to v0.5.1!      │
  │                                           │
  │   v0.5.0 → v0.5.1                         │
  │                                           │
  │   🎉 Restart contui to use new version    │
  │                                           │
  └───────────────────────────────────────────┘
```

### Failure
```
  ❌ Update failed                             (red, bold)

     Could not replace binary: Permission denied

     Try: sudo contui update                   (dim)
```

## Phase 3: Install Script (`scripts/install.sh`)

### Banner
```
   ┌─────────────────────────────────────┐
   │                                     │
   │      ██████╗ ██████╗ ███╗   ██╗    │
   │     ██╔════╝██╔═══██╗████╗  ██║    │
   │     ██║     ██║   ██║██╔██╗ ██║    │
   │     ██║     ██║   ██║██║╚██╗██║    │
   │     ╚██████╗╚██████╔╝██║ ╚████║    │
   │      ╚═════╝ ╚═════╝ ╚═╝  ╚═══╝    │
   │           CONTUI INSTALLER          │
   │                                     │
   └─────────────────────────────────────┘
```

### Stages (with spinners and 500ms delays)
```
  ⠋ Detecting platform...
  ✓ Detected: macOS arm64                        (green)

  ⠋ Fetching latest version...
  ✓ Latest version: v0.5.1                       (green)

  📥 Downloading contui v0.5.1...
  [████████████████████████████] 100%
   7.1 MB / 7.1 MB

  ⠋ Extracting archive...
  ✓ Extracted                                    (green)

  ⠋ Installing to ~/.local/bin...
  ✓ Installed                                    (green)

  ⠋ Verifying...
  ✓ Verified: contui 0.5.1                       (green)
```

### Success
```
  ┌───────────────────────────────────────────────┐
  │                                               │
  │   ✅ contui installed successfully!           │
  │                                               │
  │   Get started:                                │
  │     $ contui              Launch TUI          │
  │     $ contui --help       Show help           │
  │                                               │
  │   📚 Docs: github.com/aycandv/contui          │
  │                                               │
  └───────────────────────────────────────────────┘
```

## Implementation

### Dependencies (Cargo.toml)
```toml
[dependencies]
indicatif = "0.17"
console = "0.15"
```

### Files to Modify
1. `Cargo.toml` - Add indicatif, console
2. `src/update/mod.rs` - Rewrite check/install with animations
3. `scripts/install.sh` - Add spinner functions, progress bar, delays

### Helper Module Structure (`src/update/mod.rs`)
```rust
// New helper functions
fn spinner(message: &str) -> ProgressBar;
fn progress_bar(total: u64) -> ProgressBar;
fn print_box(lines: &[&str]);
fn print_success(message: &str);
fn print_error(message: &str);
fn delay();  // 500ms sleep
```

### Shell Script Helpers
```bash
spinner() { ... }      # Animated spinner with message
progress_bar() { ... } # Download progress
print_box() { ... }    # Bordered message box
delay() { sleep 0.5 }  # 500ms delay
```

## Testing

- Test with `--skip-update-check` to bypass during development
- Test slow network with `tc` or throttling proxy
- Test offline mode (disconnect network)
- Test install script in Docker containers (various distros)
