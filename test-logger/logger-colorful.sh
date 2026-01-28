#!/bin/sh
# Colorful test logger with various log types

counter=0

# Colors for terminal output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

while true; do
    counter=$((counter + 1))
    timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    
    # Regular info log
    echo "[$timestamp] INFO: Processing request #$counter - GET /api/users"
    
    # Database log every 2nd message
    if [ $((counter % 2)) -eq 0 ]; then
        echo "[$timestamp] DEBUG: Database query executed in 12ms"
    fi
    
    # Warning every 3rd message
    if [ $((counter % 3)) -eq 0 ]; then
        echo "[$timestamp] WARN: High memory usage detected: 78%"
    fi
    
    # Error every 5th message (to stderr)
    if [ $((counter % 5)) -eq 0 ]; then
        echo "[$timestamp] ERROR: Failed to connect to cache server: Connection timeout" >&2
    fi
    
    # Critical error every 10th message (to stderr)
    if [ $((counter % 10)) -eq 0 ]; then
        echo "[$timestamp] CRITICAL: Database connection lost, retrying..." >&2
        echo "[$timestamp] INFO: Reconnected to database after 2s"
    fi
    
    # Simulate some processing time variation
    sleep 0.8
done
