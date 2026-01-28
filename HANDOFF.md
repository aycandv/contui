# Handoff to Ralph - US-013 Log Search

## Current State
Branch: `ralph/US-011-log-streaming` (continue on this branch)

## What's Been Implemented (US-013-1 Partial)

### State Management (COMPLETE)
**File: `src/state/app_state.rs`**

Added to `LogViewState`:
```rust
pub search_pattern: Option<String>,
pub search_matches: Vec<usize>,      // indices of matching log entries
pub current_match: Option<usize>,     // index into search_matches
pub show_search_input: bool,
```

Added methods to `AppState`:
- `show_log_search()` - Opens search, disables follow mode
- `hide_log_search()` - Closes search input
- `set_log_search(pattern)` - Sets pattern, finds all matches, scrolls to first
- `clear_log_search()` - Clears pattern and matches
- `next_search_match()` - Jumps to next match
- `prev_search_match()` - Jumps to previous match

### Keybindings (COMPLETE)
**File: `src/ui/app.rs`**

- `/` - Open search input
- `n` - Next match
- `N` - Previous match
- `Esc` - Close search, clear pattern
- Enter - Confirm search

## What Ralph Needs to Do

### 1. Render Search Input (src/ui/components/log_viewer.rs)
When `log_view.show_search_input` is true, render a search bar at the bottom:
```
┌─────────────────────────────────────────┐
│ Log content here...                     │
│                                         │
├─────────────────────────────────────────┤
│ Search: pattern_here               3/15 │
└─────────────────────────────────────────┘
```

### 2. Handle Character Input (src/ui/app.rs)
In `handle_log_search_key()`, handle:
- `KeyCode::Char(c)` - Append to search pattern, call `set_log_search()`
- `KeyCode::Backspace` - Remove last char
- Update search live as user types

### 3. Highlight Matches (src/ui/components/log_viewer.rs)
In `render_log_viewer()`:
- Check if each visible log entry is in `search_matches`
- If yes, highlight the line background in yellow
- Use `Line::styled()` with `Style::default().bg(Color::Yellow)`

### 4. Show Match Counter
In search bar, show: `current_match+1 / search_matches.len()`

## Testing Steps
1. Open log view with 'l'
2. Load logs with 'r'
3. Press '/' to open search
4. Type 'error' - should highlight matching lines
5. Press 'n'/'N' to navigate
6. Press 'Esc' to close

## Key Code Locations
- Search state: `src/state/app_state.rs` lines 50-58 (LogViewState)
- Search methods: `src/state/app_state.rs` lines 385-450
- Key handling: `src/ui/app.rs` lines 221-272
- Log rendering: `src/ui/components/log_viewer.rs` lines 14-104
