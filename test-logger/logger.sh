#!/bin/sh
# Simple test logger that outputs to stdout and stderr every second

# Disable output buffering
export PYTHONUNBUFFERED=1
counter=0

while true; do
    counter=$((counter + 1))
    timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    
    # Output to stdout (white in log viewer)
    printf "[%s] INFO: Log message #%d from stdout - everything is working fine\n" "$timestamp" "$counter"
    
    # Output to stderr (red in log viewer) every 5th message
    if [ $((counter % 5)) -eq 0 ]; then
        printf "[%s] ERROR: Error message #%d from stderr - something went wrong\n" "$timestamp" "$counter" >&2
    fi
    
    # Output a warning every 3rd message
    if [ $((counter % 3)) -eq 0 ]; then
        printf "[%s] WARN: Warning message #%d - this is a warning\n" "$timestamp" "$counter"
    fi
    
    sleep 1
done
