#!/bin/bash

# Script to check if files exist and are modified
LOG_FILE="check-files-log.txt"

# Clear the log file
> $LOG_FILE

# Function to log messages
log() {
  echo "$1" | tee -a $LOG_FILE
}

# Check if files exist
log "Checking if files exist..."

# Check oyl-sdk files
log "Checking oyl-sdk files..."
if [ -f "oyl-sdk/lib/cli/alkane.js" ]; then
  log "oyl-sdk/lib/cli/alkane.js exists"
else
  log "oyl-sdk/lib/cli/alkane.js does not exist"
fi

if [ -f "oyl-sdk/lib/rpclient/alkanes.js" ]; then
  log "oyl-sdk/lib/rpclient/alkanes.js exists"
else
  log "oyl-sdk/lib/rpclient/alkanes.js does not exist"
fi

# Check minimal-contract files
log "Checking minimal-contract files..."
if [ -f "ALKANE_ID_EXTENSION.md" ]; then
  log "ALKANE_ID_EXTENSION.md exists"
else
  log "ALKANE_ID_EXTENSION.md does not exist"
fi

if [ -f "ALKANE_ID_EXTENSION_SPEC.md" ]; then
  log "ALKANE_ID_EXTENSION_SPEC.md exists"
else
  log "ALKANE_ID_EXTENSION_SPEC.md does not exist"
fi

if [ -f "IMPLEMENTATION_SUMMARY.md" ]; then
  log "IMPLEMENTATION_SUMMARY.md exists"
else
  log "IMPLEMENTATION_SUMMARY.md does not exist"
fi

if [ -f "README_ALKANE_ID_EXTENSION.md" ]; then
  log "README_ALKANE_ID_EXTENSION.md exists"
else
  log "README_ALKANE_ID_EXTENSION.md does not exist"
fi

if [ -d "examples" ]; then
  log "examples directory exists"
else
  log "examples directory does not exist"
fi

if [ -f "examples/alkane-execute-examples.js" ]; then
  log "examples/alkane-execute-examples.js exists"
else
  log "examples/alkane-execute-examples.js does not exist"
fi

# Check if files are modified
log "Checking if files are modified..."

# Check oyl-sdk files
log "Checking oyl-sdk files..."
cd oyl-sdk
git diff --name-only lib/cli/alkane.js >> ../$LOG_FILE 2>&1
git diff --name-only lib/rpclient/alkanes.js >> ../$LOG_FILE 2>&1
cd ..

log "All checks completed. Check $LOG_FILE for details."
