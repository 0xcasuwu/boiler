#!/bin/bash

# =============================================
# 🌐 Structured Network Operations Wrapper
# =============================================

# Get repository root directory
ROOT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )/../.." >/dev/null 2>&1 && pwd )"

# Check if Node.js is installed
if ! command -v node &> /dev/null; then
    echo "Error: Node.js is required but not installed."
    echo "Please install Node.js from https://nodejs.org/"
    exit 1
fi

# Execute the JavaScript implementation with all arguments passed through
node "${ROOT_DIR}/bin/net/structured_network.js" "$@"
exit $?
