# Test Logger for DockMon

A simple Docker container that generates log output every second for testing the DockMon log viewer.

## Features

- Outputs **INFO** messages to stdout every second (shown in white)
- Outputs **ERROR** messages to stderr every 5th message (shown in red)
- Outputs **WARN** messages every 3rd message
- Timestamps on all messages
- Runs indefinitely until stopped

## Building and Running

### Option 1: Docker Compose (Recommended)

```bash
cd test-logger
docker-compose up --build -d
```

### Option 2: Docker directly

```bash
cd test-logger
docker build -t test-logger .
docker run -d --name test-logger test-logger
```

## Viewing Logs

### Using DockMon

1. Start DockMon
2. Select the `test-logger` container
3. Press **'l'** to view logs
4. Press **'f'** to toggle follow mode
5. Press **'q'** or **Esc** to exit log view

### Using Docker CLI

```bash
# View logs
docker logs test-logger

# Follow logs
docker logs -f test-logger

# View last 20 lines
docker logs --tail 20 test-logger
```

## Stopping and Removing

```bash
# Stop the container
docker stop test-logger

# Remove the container
docker rm test-logger

# Remove the image
docker rmi test-logger
```

## Log Output Example

```
[2024-01-28 10:30:01] INFO: Log message #1 from stdout - everything is working fine
[2024-01-28 10:30:02] INFO: Log message #2 from stdout - everything is working fine
[2024-01-28 10:30:03] WARN: Warning message #3 - this is a warning
[2024-01-28 10:30:04] INFO: Log message #4 from stdout - everything is working fine
[2024-01-28 10:30:05] ERROR: Error message #5 from stderr - something went wrong
```
