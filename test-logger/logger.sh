#!/bin/sh
# Simple test logger that outputs to stdout and stderr every second

counter=0

while true; do
    counter=$((counter + 1))
    timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    
    # Output to stdout (white in log viewer)
    echo "[$timestamp] INFO: Log message #$counter from stdout - everything is working fine"
    
    # Output to stderr (red in log viewer) every 5th message
    if [ $((counter % 5)) -eq 0 ]; then
        echo "[$timestamp] ERROR: Error message #$counter from stderr - something went wrong" >&2
    fi
    
    # Output a warning every 3rd message
    if [ $((counter % 3)) -eq 0 ]; then
        echo "[$timestamp] WARN: Warning message #$counter - this is a warning"
    fi
    
    sleep 1
done
