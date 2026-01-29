# US-018: System Disk Usage and Prune

## User Story
As a Docker user, I want to view system-wide disk usage and clean up unused resources so that I can reclaim disk space and keep my Docker environment healthy.

## Acceptance Criteria
- [ ] System tab shows disk usage breakdown:
  - Images (total size, reclaimable size)
  - Containers (total size, reclaimable size)
  - Local volumes (total size, reclaimable size)
  - Build cache (total size, reclaimable size)
- [ ] Show grand total and total reclaimable space
- [ ] Press 'p' to open prune dialog with options:
  - Prune containers (remove stopped)
  - Prune images (remove dangling)
  - Prune volumes (remove unused)
  - Prune networks (remove unused)
  - Prune build cache
  - Prune everything (system prune)
- [ ] Show confirmation dialog before pruning
- [ ] Show notification with reclaimed space after prune
- [ ] Auto-refresh disk usage after prune operation
- [ ] Help text updated with new keybindings

## UI Layout
```
┌─────────────────────────────────────────────────────────────────┐
│ System Information                                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Docker Version:   24.0.7                                       │
│  API Version:      1.43                                         │
│  OS/Arch:          linux/amd64                                  │
│  Kernel Version:   5.15.0                                       │
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│ Disk Usage                                                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Images:           4.2 GB  (Reclaimable: 1.1 GB)               │
│  Containers:       256 MB  (Reclaimable: 128 MB)               │
│  Local Volumes:    1.5 GB  (Reclaimable: 800 MB)               │
│  Build Cache:      2.3 GB  (Reclaimable: 2.3 GB)               │
│  ─────────────────────────────────────────────────────────      │
│  Total:            8.3 GB                                       │
│  Total Reclaimable: 4.3 GB  [51% can be freed]                 │
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│ Actions: [p]Prune unused resources                              │
└─────────────────────────────────────────────────────────────────┘

Prune Dialog:
┌─────────────────────────────────────────┐
│ Prune Unused Resources                  │
├─────────────────────────────────────────┤
│ Select what to prune:                   │
│                                         │
│   [ ] Containers (stopped)              │
│   [✓] Images (dangling)                 │
│   [ ] Volumes (unused)                  │
│   [ ] Networks (unused)                 │
│   [✓] Build Cache                       │
│                                         │
│   [✓] Everything (system prune)         │
│                                         │
├─────────────────────────────────────────┤
│ [Enter]Confirm [Esc]Cancel              │
└─────────────────────────────────────────┘
```

## Technical Notes
- Use Docker API `system_df()` for disk usage
- Use `prune_containers()`, `prune_images()`, `prune_volumes()`, `prune_networks()` for cleanup
- Use `build_prune()` for build cache cleanup
- Format sizes in human-readable format (MB, GB, TB)
- Calculate reclaimable percentage
- Show progress during prune operations

## Definition of Done
- [ ] Code implemented and tested
- [ ] Help text updated with 'p' keybinding
- [ ] Error handling for prune failures
- [ ] Confirmation dialog prevents accidental data loss
- [ ] Build passes
