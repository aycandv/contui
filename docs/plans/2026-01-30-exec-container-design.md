# Exec Into Container (Embedded) Design

## Summary
Add an embedded exec pane to the Containers view that opens an interactive shell inside a container. The exec pane lives **below the container list**, supports full‑screen TTY apps, and keeps the rest of the UI visible. Focus toggles with **Ctrl+E**. The session ends when the shell exits.

## UX & Interaction
- Trigger: `x` on Containers tab.
- Placement: bottom panel under the list (similar to stats), detail panel remains on the right.
- Focus: `Ctrl+E` toggles focus between UI and exec pane. When exec has focus, all keys go to the container except `Ctrl+E`.
- Exit: user closes shell via `exit` / Ctrl+D; pane closes automatically.
- Container stopped: prompt to start container first (Confirm dialog). If started, proceed to exec.
- Session pinning: exec stays bound to the original container even if selection changes.
- Header shows container name + command + focus state.

## Exec Command Selection
Policy (chosen):
- Use `Entrypoint + Cmd` **only if it appears to be a shell** (e.g., ends with `sh`, `bash`, `zsh`, or includes `-lc`).
- Otherwise fall back to `/bin/sh -lc`.
- Default user only (no prompt).

## Architecture
### State
Add `ExecSessionState` to `AppState`:
- `container_id`, `container_name`
- `focus: bool`
- `status: Running | Exited | Error(String)`
- `screen: vt100::Screen` (or equivalent)
- `pty_writer`: handle for stdin
- `last_exit_code: Option<i64>` (if available)

### Data Flow
1. UI emits `UiAction::ExecContainer(id)` from `x`.
2. App validates container state; if stopped, opens confirm dialog.
3. App starts exec via Docker API (Bollard `create_exec` + `start_exec`).
4. Spawn background task to read exec output stream and feed vt100 parser.
5. UI renders `screen` into the exec pane each tick.
6. On stream end, mark session exited and close pane.

### Terminal Emulation
Use a VT parser (e.g., `vt100`) to handle ANSI, cursor movement, clears, and full‑screen apps. Size the virtual screen to the exec pane and update on resize.

### Resizing
On terminal resize or pane size change, call Docker exec resize (Bollard `resize_exec`) with pane width/height so TTY apps adjust.

## Error Handling
- Docker disconnected → notification, no pane.
- Container stopped and start fails → notification, no pane.
- Exec create/start fails → notification, keep UI responsive.
- Stream ends unexpectedly → show “Exec ended” in pane header and close.

## UI/Help Updates
- Help overlay: add `x: Exec`, `Ctrl+E: Toggle exec focus`.
- Footer hint on Containers: include `x: Exec`.

## Testing
- Unit tests:
  - command selection logic (shell detection + fallback)
  - exec state transitions (open/close, focus toggle)
  - resize propagation logic
- UI render tests:
  - exec pane header shows container + focus state
  - error/empty state rendering
- Integration tests (ignored):
  - exec into a running container, run `ls`, run `top`, resize terminal.

## Rollout
- Feature gated only by UI action; no config flag.
- Manual smoke test in a real container before release.

## Alternatives Considered
- Suspend TUI and run `docker exec -it` in the terminal: reliable but exits TUI.
- Raw output only: simpler but breaks full‑screen apps.

## Open Questions
None.
