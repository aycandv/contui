# US-017: Image Details View

## User Story
As a Docker user, I want to view detailed information about a Docker image including its layers, size breakdown, and history, so that I can understand what's in the image and manage my images effectively.

## Acceptance Criteria
- [ ] Press 'i' on an image to open details view
- [ ] Display image ID and name/tags
- [ ] Display total size
- [ ] Display creation date
- [ ] Display author
- [ ] Display OS and architecture
- [ ] Display exposed ports
- [ ] Display environment variables
- [ ] Display entrypoint and command
- [ ] Display layer history with sizes
- [ ] Display labels
- [ ] Press 'q' or 'Esc' to close details view
- [ ] Scroll through content with ↑/↓ or PgUp/PgDn

## UI Layout
```
┌─────────────────────────────────────────────────────────────┐
│ Image: test-logger:latest                                   │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ID:        sha256:a1b2c3d4...                              │
│  Tags:      test-logger:latest, myregistry/test-logger:v1   │
│  Size:      156.3 MB                                        │
│  Created:   2026-01-28 10:30:00                             │
│  Author:    developer@example.com                           │
│  OS/Arch:   linux/amd64                                     │
│                                                             │
│  Exposed Ports:                                             │
│    8080/tcp, 5000/tcp                                       │
│                                                             │
│  Environment:                                               │
│    PATH=/usr/local/bin:/usr/bin:/bin                        │
│    APP_ENV=production                                       │
│                                                             │
│  Entrypoint: ["/entrypoint.sh"]                             │
│  Command:    ["node", "server.js"]                          │
│                                                             │
│  Layers (12):                                               │
│    45.2 MB  FROM ubuntu:22.04                               │
│    12.1 MB  RUN apt-get update && apt-get install -y nodejs │
│     8.5 MB  COPY package.json /app/                         │
│     2.3 MB  RUN npm install                                 │
│    88.2 MB  COPY . /app/                                    │
│       0 B   CMD ["node", "server.js"]                       │
│                                                             │
│  Labels:                                                    │
│    org.opencontainers.image.version=1.0.0                   │
│    org.opencontainers.image.source=https://...              │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│ [↑/↓]Scroll [q]Close                                        │
└─────────────────────────────────────────────────────────────┘
```

## Technical Notes
- Use Docker API `inspect_image` endpoint for image config
- Use Docker API `image_history` endpoint for layer history
- Format layer sizes in human-readable format (MB, GB)
- Truncate very long layer commands
- Show cumulative size per layer

## Definition of Done
- [ ] Code implemented and tested
- [ ] Help text updated
- [ ] Error handling for missing images
- [ ] Scrollable for images with many layers
