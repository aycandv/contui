#!/bin/sh
# Ultra simple test logger

i=0
while true; do
    i=$((i + 1))
    echo "$(date -Iseconds) Test log message $i"
    sleep 1
done
