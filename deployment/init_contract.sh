#!/bin/bash

# Simple initialization script for YieldVault contract
echo "=== Simple YieldVault Initialization ==="

# Generate blocks to ensure chain activity
echo "Generating blocks..."
NODE_OPTIONS=--require=./oyl-sdk/lib/shared/load_patch.js oyl regtest genBlocks -p oylnet -c 10

# Try to execute initialization (opcode 0) with parameters
echo "Trying initialization with basic format..."
NODE_OPTIONS=--require=./oyl-sdk/lib/shared/load_patch.js oyl alkane execute -data "0" --provider oylnet

# Try to execute initialization with string parameters
echo "Trying initialization with full parameters..."
NODE_OPTIONS=--require=./oyl-sdk/lib/shared/load_patch.js oyl alkane execute -data "0,YieldVault,YVT,Bitcoin,BTC,8" --provider oylnet

# Generate confirmation blocks
echo "Generating confirmation blocks..."
NODE_OPTIONS=--require=./oyl-sdk/lib/shared/load_patch.js oyl regtest genBlocks -p oylnet -c 10

# Check contract state - get name (opcode 100)
echo "Checking contract name (opcode 100)..."
NODE_OPTIONS=--require=./oyl-sdk/lib/shared/load_patch.js oyl alkane execute -data "100" --provider oylnet

# Check contract symbol (opcode 101)
echo "Checking contract symbol (opcode 101)..."
NODE_OPTIONS=--require=./oyl-sdk/lib/shared/load_patch.js oyl alkane execute -data "101" --provider oylnet

echo "=== Initialization Process Complete ==="
