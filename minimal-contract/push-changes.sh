#!/bin/bash

# Script to commit changes locally
LOG_FILE="commit-changes-log.txt"

# Clear the log file
> $LOG_FILE

# Function to log messages
log() {
  echo "$1" | tee -a $LOG_FILE
}

# Set git user configuration
log "Setting git user configuration..."
git config --global user.email "user@example.com" >> $LOG_FILE 2>&1
git config --global user.name "User" >> $LOG_FILE 2>&1

# Commit changes to oyl-sdk repository
log "Committing changes to oyl-sdk repository..."
cd oyl-sdk
git status >> $LOG_FILE 2>&1
log "Adding files..."
git add lib/cli/alkane.js lib/rpclient/alkanes.js >> $LOG_FILE 2>&1
log "Committing changes..."
git commit -m "Add alkane-id option to alkaneExecute command and toTxId function to alkanes.js" >> $LOG_FILE 2>&1
cd ..

# Commit changes to minimal-contract repository
log "Committing changes to minimal-contract repository..."
git status >> $LOG_FILE 2>&1
log "Adding files..."
git add ALKANE_ID_EXTENSION.md ALKANE_ID_EXTENSION_SPEC.md IMPLEMENTATION_SUMMARY.md README_ALKANE_ID_EXTENSION.md examples/ >> $LOG_FILE 2>&1
log "Committing changes..."
git commit -m "Add documentation and examples for alkane-id extension" >> $LOG_FILE 2>&1

log "All changes committed successfully!"
log "Check $LOG_FILE for details."
log "To push the changes to the remote repository, run:"
log "  cd oyl-sdk && git push origin main && cd .. && git push origin dev"
