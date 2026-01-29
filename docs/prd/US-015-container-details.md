# US-015: Container Details View

## User Story
As a Docker user, I want to view detailed configuration information for a container, so that I can see its ports, mounts, environment variables, and other settings.

## Acceptance Criteria
- [ ] Press 'i' on a container to open details view
- [ ] Display container ID and name
- [ ] Display image information
- [ ] Display status and state
- [ ] Display exposed/published ports
- [ ] Display volume mounts
- [ ] Display environment variables
- [ ] Display labels
- [ ] Display network settings (IP addresses, gateway)
- [ ] Display restart policy
- [ ] Display creation time
- [ ] Display command and entrypoint
- [ ] Press 'q' or 'Esc' to close details view
- [ ] Scroll through long content with ↑/↓ or PgUp/PgDn

## UI Layout
```
┌─────────────────────────────────────────────────────────────┐
│ Container: test-logger                                      │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ID:        a1b2c3d4e5f6...                                 │
│  Image:     test-logger:latest                              │
│  Status:    Up 2 hours (healthy)                            │
│  Created:   2026-01-28 10:30:00                             │
│  Restart:   unless-stopped                                  │
│                                                             │
│  Command:   /app/start.sh                                   │
│  Entrypoint: ["/bin/sh", "-c"]                              │
│                                                             │
│  Ports:                                                     │
│    8080/tcp → 0.0.0.0:8080                                  │
│    5000/tcp → 0.0.0.0:5000                                  │
│                                                             │
│  Mounts:                                                    │
│    /host/data → /app/data (rw)                              │
│    /var/log → /app/logs (ro)                                │
│                                                             │
│  Environment:                                               │
│    ENV=production                                           │
│    DEBUG=false                                              │
│    DATABASE_URL=postgres://...                              │
│                                                             │
│  Labels:                                                    │
│    com.docker.compose.project=myapp                         │
│    com.docker.compose.service=logger                        │
│                                                             │
│  Network:                                                   │
│    Name:    bridge                                          │
│    IP:      172.17.0.2                                      │
│    Gateway: 172.17.0.1                                      │
│    Mac:     02:42:ac:11:00:02                               │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│ [↑/↓]Scroll [q]Close                                        │
└─────────────────────────────────────────────────────────────┘
```

## Technical Notes
- Use Docker API `inspect_container` endpoint
- Format multiline values nicely (arrays, maps)
- Truncate very long values (env vars, commands)
- Handle containers with no ports/mounts/labels gracefully

## Definition of Done
- [ ] Code implemented and tested
- [ ] Help text updated
- [ ] Error handling for stopped/missing containers
- [ ] Scrollable for containers with lots of configuration
