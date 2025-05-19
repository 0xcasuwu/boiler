#!/bin/bash

# Script to commit individual files
LOG_FILE="commit-individual-files-log.txt"

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
log "Adding and committing lib/cli/alkane.js..."
git add lib/cli/alkane.js >> $LOG_FILE 2>&1
git commit -m "Add alkane-id option to alkaneExecute command" >> $LOG_FILE 2>&1

log "Adding and committing lib/rpclient/alkanes.js..."
git add lib/rpclient/alkanes.js >> $LOG_FILE 2>&1
git commit -m "Add toTxId function to alkanes.js" >> $LOG_FILE 2>&1
cd ..

# Commit changes to minimal-contract repository
log "Committing changes to minimal-contract repository..."

log "Adding and committing ALKANE_ID_EXTENSION.md..."
git add ALKANE_ID_EXTENSION.md >> $LOG_FILE 2>&1
git commit -m "Add documentation for alkane-id extension" >> $LOG_FILE 2>&1

log "Adding and committing ALKANE_ID_EXTENSION_SPEC.md..."
git add ALKANE_ID_EXTENSION_SPEC.md >> $LOG_FILE 2>&1
git commit -m "Add technical specification for alkane-id extension" >> $LOG_FILE 2>&1

log "Adding and committing IMPLEMENTATION_SUMMARY.md..."
git add IMPLEMENTATION_SUMMARY.md >> $LOG_FILE 2>&1
git commit -m "Add implementation summary for alkane-id extension" >> $LOG_FILE 2>&1

log "Adding and committing README_ALKANE_ID_EXTENSION.md..."
git add README_ALKANE_ID_EXTENSION.md >> $LOG_FILE 2>&1
git commit -m "Add README for alkane-id extension" >> $LOG_FILE 2>&1

log "Adding and committing examples directory..."
git add examples/ >> $LOG_FILE 2>&1
git commit -m "Add examples for alkane-id extension" >> $LOG_FILE 2>&1

log "All changes committed successfully!"
log "Check $LOG_FILE for details."
log "To push the changes to the remote repository, run:"
log "  cd oyl-sdk && git push origin main && cd .. && git push origin dev"
