# US-014: Container Stats View

## User Story
As a Docker user, I want to view real-time resource usage statistics for a container, so that I can monitor CPU, memory, and network performance.

## Acceptance Criteria
- [ ] Press 'm' on a container to open stats view
- [ ] Display CPU usage percentage
- [ ] Display memory usage (used / limit, percentage)
- [ ] Display network I/O (RX / TX bytes)
- [ ] Display block I/O (read / write bytes)
- [ ] Display PIDs count
- [ ] Auto-refresh every 1 second
- [ ] Press 'f' to toggle follow/pause updates
- [ ] Press 'q' or 'Esc' to close stats view
- [ ] Show notification on error fetching stats

## UI Layout
```
┌─────────────────────────────────────────────────────────────┐
│ Stats: test-logger [FOLLOW]                                 │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  CPU Usage                                                  │
│  ████████████████████ 45.2%                                 │
│                                                             │
│  Memory                                                     │
│  ██████████ 128MB / 512MB (25.0%)                           │
│                                                             │
│  Network I/O                                                │
│  RX: 1.2 MB    TX: 850 KB                                   │
│                                                             │
│  Block I/O                                                  │
│  Read: 2.5 MB  Write: 1.1 MB                                │
│                                                             │
│  PIDs: 12                                                   │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│ [f]Follow [q]Close                                          │
└─────────────────────────────────────────────────────────────┘
```

## Technical Notes
- Use Docker API `stats` endpoint (streaming)
- Calculate CPU percentage using:
  - `cpu_delta = cpu_stats.cpu_usage.total_usage - precpu_stats.cpu_usage.total_usage`
  - `system_delta = cpu_stats.system_cpu_usage - precpu_stats.system_cpu_usage`
  - `cpu_percent = (cpu_delta / system_delta) * cpu_stats.online_cpus * 100`
- Memory percentage: `memory_stats.usage / memory_stats.limit * 100`
- Handle stopped containers (show error/nothing)
- Similar architecture to log streaming (channel-based)

## Definition of Done
- [ ] Code implemented and tested
- [ ] Help text updated
- [ ] Error handling for stopped containers
- [ ] No UI freezing during stats streaming
