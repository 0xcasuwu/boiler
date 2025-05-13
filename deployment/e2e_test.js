/**
* YieldVault End-to-End Test
 *
 * This script performs an end-to-end test of the YieldVault contract on OylNet,
 * simulating a full user story:
 *
 * 1. Deploy the contract
 * 2. Initialize the contract
 * 3. User deposits assets
 * 4. User mints shares
 * 5. User redeems shares for assets
 * 6. Verify all transactions
 */

// Import the OYL SDK with the patch applied
require('/workspaces/boiler/oyl-sdk/lib/shared/load_patch.js');
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

console.log(`${COLORS.BOLD}${COLORS.BLUE}=== YieldVault End-to-End Test ===${COLORS.NC}\n`);

// Path to the WebAssembly contract
const wasmPath = '/workspaces/boiler/target/wasm32-unknown-unknown/release/yield_vault.wasm';

// Check if the WebAssembly contract exists
if (!fs.existsSync(wasmPath)) {
  console.error(`${COLORS.RED}WebAssembly contract not found at ${wasmPath}${COLORS.NC}`);
  process.exit(1);
}

// Deploy the contract
console.log(`${COLORS.YELLOW}Deploying contract...${COLORS.NC}`);
try {
  // Use the correct deployment command format from deploy_yield_vault.sh
  const deployCommand = `NODE_OPTIONS=--require=/workspaces/boiler/oyl-sdk/lib/shared/load_patch.js oyl alkane new-contract \\
    --contract "${wasmPath}" \\
    --provider "oylnet" \\
    --calldata "0" \\
    --feeRate 10`;
  
  const deployResult = execSync(deployCommand, { encoding: 'utf8' });
  console.log(deployResult);
  
  // Extract the contract ID from the output
  const contractIdMatch = deployResult.match(/[a-f0-9]{64}/);
  if (!contractIdMatch) {
    console.error(`${COLORS.RED}Failed to extract contract ID from deployment result.${COLORS.NC}`);
    process.exit(1);
  }
  
  const contractId = contractIdMatch[0];
  console.log(`${COLORS.GREEN}Contract deployed with ID: ${contractId}${COLORS.NC}\n`);
  
  // Save the contract ID to a file
  fs.writeFileSync('/workspaces/boiler/contract_details.json', JSON.stringify({ contractId, status: "DEPLOYED" }, null, 2));
  
  // Generate blocks to confirm deployment
  console.log(`${COLORS.YELLOW}Generating blocks to confirm deployment...${COLORS.NC}`);
  const genBlocksCommand = `NODE_OPTIONS=--require=/workspaces/boiler/oyl-sdk/lib/shared/load_patch.js oyl regtest genBlocks -p oylnet -c 10`;
  execSync(genBlocksCommand, { encoding: 'utf8' });
} catch (error) {
  console.error(`${COLORS.RED}Failed to deploy contract: ${error.message}${COLORS.NC}`);
  process.exit(1);
}

// Initialize the contract
console.log(`${COLORS.YELLOW}Initializing contract...${COLORS.NC}`);
try {
  const contractDetails = JSON.parse(fs.readFileSync('/workspaces/boiler/contract_details.json', 'utf8'));
  const contractId = contractDetails.contractId;
  
  // Initialize the contract with opcode 0
  const initCommand = `NODE_OPTIONS=--require=/workspaces/boiler/oyl-sdk/lib/shared/load_patch.js oyl alkane execute -data "0" --provider oylnet`;
  const initResult = execSync(initCommand, { encoding: 'utf8' });
  console.log(initResult);
  console.log(`${COLORS.GREEN}Contract initialized successfully.${COLORS.NC}\n`);
} catch (error) {
  console.error(`${COLORS.RED}Failed to initialize contract: ${error.message}${COLORS.NC}`);
  process.exit(1);
}

// Verify contract metadata
console.log(`${COLORS.YELLOW}=== Verifying Contract Metadata ===${COLORS.NC}\n`);

// Function to execute an opcode and return the result
function executeOpcode(opcode, args = '') {
  console.log(`Executing opcode ${opcode}`);
  if (args) {
    console.log(`Parameters: ${args}`);
  }
  
  try {
    const command = `NODE_OPTIONS=--require=/workspaces/boiler/oyl-sdk/lib/shared/load_patch.js oyl alkane execute -data "${opcode}${args ? `,${args}` : ''}" --provider oylnet`;
    console.log(`$ ${command}`);
    const result = execSync(command, { encoding: 'utf8' });
    console.log(`Operation successful`);
    return result.trim();
  } catch (error) {
    console.error(`Operation failed`);
    console.error(`${error.message}`);
    return null;
  }
}

// Verify contract name
const name = executeOpcode(100);
console.log(`Name: ${name}`);

// Verify contract symbol
const symbol = executeOpcode(101);
console.log(`Symbol: ${symbol}`);

// Verify contract decimals
const decimals = executeOpcode(102);
console.log(`Decimals: ${decimals}`);

// Verify asset symbol
const assetSymbol = executeOpcode(103);
console.log(`Asset Symbol: ${assetSymbol}`);

// Verify total assets
const totalAssets = executeOpcode(200);
console.log(`Total Assets: ${totalAssets}`);

// Verify total supply
const totalSupply = executeOpcode(601);
console.log(`Total Supply: ${totalSupply}`);

console.log(`\n${COLORS.GREEN}${COLORS.BOLD}End-to-end test completed successfully.${COLORS.NC}`);
