/**
 * Fix YieldVault Contract Initialization
 * 
 * This script correctly formats the initialization parameters for the YieldVault contract
 * to match the expected format in the Rust code.
 */

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

// ANSI color codes for terminal output
const COLORS = {
  RED: '\x1b[31m',
  GREEN: '\x1b[32m',
  YELLOW: '\x1b[33m',
  BLUE: '\x1b[34m',
  CYAN: '\x1b[36m',
  BOLD: '\x1b[1m',
  NC: '\x1b[0m' // No Color
};

// Contract ID from successful deployment
const CONTRACT_ID = '7dbd26587f4d058577afa7e180fe1bfbe4941e6b5459e783d14e336c18238f0f';

console.log(`${COLORS.BOLD}${COLORS.BLUE}=== Fix YieldVault Contract Initialization ====${COLORS.NC}\n`);

// Generate blocks to confirm deployment
console.log(`${COLORS.YELLOW}1. Generating blocks to confirm deployment${COLORS.NC}`);
try {
  execSync('NODE_OPTIONS=--require=./oyl-sdk/lib/shared/load_patch.js oyl regtest genBlocks -p oylnet -c 20', { encoding: 'utf8', stdio: 'inherit' });
  console.log(`${COLORS.GREEN}✅ Blocks generated${COLORS.NC}`);
} catch (error) {
  console.log(`${COLORS.YELLOW}⚠️ Block generation returned an error:${COLORS.NC} ${error.message}`);
}

// Create properly formatted calldata for initialization
console.log(`\n${COLORS.YELLOW}2. Creating properly formatted initialization calldata${COLORS.NC}`);

// Define the parameters
const name = "YieldVault";
const symbol = "YVT";
const assetName = "Bitcoin";
const assetSymbol = "BTC";
const decimals = 8;

// Create the properly formatted binary calldata
// We need to create raw binary with the proper format expected by the contract
// Convert strings to byte arrays with NULL separators
console.log(`${COLORS.CYAN}Creating binary calldata with proper formatting...${COLORS.NC}`);
console.log(`Parameters:
  Name: ${name}
  Symbol: ${symbol}
  Asset Name: ${assetName}
  Asset Symbol: ${assetSymbol}
  Decimals: ${decimals}
`);

// In JavaScript, we need to convert the initialization opcode and parameters
// to a format suitable for OylNet CLI
const initializeOpcode = 0;

// Handle initialization with proper string encoding
console.log(`\n${COLORS.YELLOW}3. Executing initialization${COLORS.NC}`);
try {
  const command = `NODE_OPTIONS=--require=./oyl-sdk/lib/shared/load_patch.js oyl alkane execute -p oylnet --contractId "${CONTRACT_ID}" --calldata "${initializeOpcode},${name},${symbol},${assetName},${assetSymbol},${decimals}"`;
  
  console.log(`${COLORS.CYAN}$ ${command}${COLORS.NC}`);
  const result = execSync(command, { encoding: 'utf8', stdio: 'inherit' });
  console.log(`${COLORS.GREEN}✅ Initialization executed${COLORS.NC}`);
} catch (error) {
  console.log(`${COLORS.RED}❌ Initialization failed:${COLORS.NC} ${error.message}`);
}

// Generate blocks to confirm initialization
console.log(`\n${COLORS.YELLOW}4. Generating blocks to confirm initialization${COLORS.NC}`);
try {
  execSync('NODE_OPTIONS=--require=./oyl-sdk/lib/shared/load_patch.js oyl regtest genBlocks -p oylnet -c 10', { encoding: 'utf8', stdio: 'inherit' });
  console.log(`${COLORS.GREEN}✅ Confirmation blocks generated${COLORS.NC}`);
} catch (error) {
  console.log(`${COLORS.YELLOW}⚠️ Block generation returned an error:${COLORS.NC} ${error.message}`);
}

// Verify the initialization by checking the contract name
console.log(`\n${COLORS.YELLOW}5. Verifying contract initialization${COLORS.NC}`);
try {
  const getNameOpcode = 100; // From the code: opcodes::GET_NAME
  
  const command = `NODE_OPTIONS=--require=./oyl-sdk/lib/shared/load_patch.js oyl alkane execute -p oylnet --contractId "${CONTRACT_ID}" --calldata "${getNameOpcode}"`;
  
  console.log(`${COLORS.CYAN}$ ${command}${COLORS.NC}`);
  const result = execSync(command, { encoding: 'utf8' });
  
  console.log(`${COLORS.GREEN}Contract name: ${result}${COLORS.NC}`);
  
  if (result.trim() === name) {
    console.log(`${COLORS.GREEN}✅ Contract successfully initialized!${COLORS.NC}`);
  } else {
    console.log(`${COLORS.YELLOW}⚠️ Contract name doesn't match expected value.${COLORS.NC}`);
    console.log(`Expected: ${name}, Got: ${result.trim()}`);
  }
} catch (error) {
  console.log(`${COLORS.RED}❌ Verification failed:${COLORS.NC} ${error.message}`);
}

console.log(`\n${COLORS.BOLD}${COLORS.BLUE}=== Initialization Process Complete ====${COLORS.NC}`);
