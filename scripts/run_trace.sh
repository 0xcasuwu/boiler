#!/bin/bash

# Run trace for a given txid
# Usage: ./run_trace.sh <txid> [vout]
# If vout not specified, tries vout 3 first, then vout 4

if [ -z "$1" ]; then
  echo "Usage: ./run_trace.sh <txid> [vout]"
  echo "Example: ./run_trace.sh 15880669141900114a22d5da171f353667246b0bf7b90ed8bcc36b5ebc3f4dfd"
  echo "Example: ./run_trace.sh 15880669141900114a22d5da171f353667246b0bf7b90ed8bcc36b5ebc3f4dfd 3"
  exit 1
fi

TXID="$1"
VOUT="${2:-}"

# Use absolute path to oyl-sdk
OYL_DIR="/home/e/Documents/oyl-sdk"
OYL_CMD="node bin/oyl.js"

# Function to run trace for specific vout
run_trace_vout() {
  local txid="$1"
  local vout="$2"
  
  echo "txid: $txid"
  cd "$OYL_DIR" && $OYL_CMD provider alkanes -method "trace" -params "[{\"txid\": \"$txid\", \"vout\": $vout}]" -p oylnet
}

# Function to check if trace output has valid data
has_valid_trace() {
  local output="$1"
  echo "$output" | grep -q '"event": "create"' && echo "$output" | grep -q '"status": "success"'
}

if [ ! -z "$VOUT" ]; then
  # Specific vout requested
  run_trace_vout "$TXID" "$VOUT"
else
  # Try vout 3 first
  echo "txid: $TXID"
  trace_output=$(cd "$OYL_DIR" && $OYL_CMD provider alkanes -method "trace" -params "[{\"txid\": \"$TXID\", \"vout\": 3}]" -p oylnet 2>/dev/null)
  
  if has_valid_trace "$trace_output"; then
    echo "$trace_output"
  else
    # Try vout 4
    trace_output=$(cd "$OYL_DIR" && $OYL_CMD provider alkanes -method "trace" -params "[{\"txid\": \"$TXID\", \"vout\": 4}]" -p oylnet 2>/dev/null)
    
    if has_valid_trace "$trace_output"; then
      echo "$trace_output"
    else
      echo "[]"
    fi
  fi
fi
