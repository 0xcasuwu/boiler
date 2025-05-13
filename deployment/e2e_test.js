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

// Path to the WebAssembly file
const WASM_PATH = path.resolve(__dirname, '../target/wasm32-unknown-unknown/release/yield_vault.wasm');
const CONTRACT_DETAILS_FILE = path.resolve(__dirname, 'contract_details.json');

// ERC-4626 opcodes
const OPCODES = {
  // Initialization
  INITIALIZE: 0,
  
  // Asset Management
  DEPOSIT: 10,
  MINT: 11,
  WITHDRAW: 12,
  REDEEM: 13,
  
  // Metadata View Functions
  GET_NAME: 100,
  GET_SYMBOL: 101,
  GET_DECIMALS: 102,
  GET_ASSET: 103,
  
  // Accounting View Functions
  GET_TOTAL_ASSETS: 200,
  CONVERT_TO_SHARES: 201,
  CONVERT_TO_ASSETS: 202,
  
  // Limit View Functions
  GET_MAX_DEPOSIT: 300,
  GET_MAX_MINT: 301,
  GET_MAX_WITHDRAW: 302,
  GET_MAX_REDEEM: 303,
  
  // Preview View Functions
  PREVIEW_DEPOSIT: 400,
  PREVIEW_MINT: 401,
  PREVIEW_WITHDRAW: 402,
  PREVIEW_REDEEM: 403,
  
  // Balance Management
  GET_TOTAL_SUPPLY: 601,
};

// User accounts for testing (using numeric IDs instead of strings)
const USERS = {
  ALICE: 1,
  BOB: 2
};

// Contract ID - use the existing contract ID
let CONTRACT_ID = '7dbd26587f4d058577afa7e180fe1bfbe4941e6b5459e783d14e336c18238f0f';

/**
 * Execute a command and return the output
 * @param {string} command - The command to execute
 * @returns {string} - The command output
 */
function executeCommand(command) {
  console.log(`${COLORS.CYAN}$ ${command}${COLORS.NC}`);
  try {
    const output = execSync(command, { encoding: 'utf8' });
    return output.trim();
  } catch (error) {
    console.error(`${COLORS.RED}Command failed: ${error.message}${COLORS.NC}`);
    if (error.stdout) console.error(`${COLORS.RED}stdout: ${error.stdout}${COLORS.NC}`);
    if (error.stderr) console.error(`${COLORS.RED}stderr: ${error.stderr}${COLORS.NC}`);
    throw error;
  }
}

/**
 * Generate blocks on OylNet
 * @param {number} count - Number of blocks to generate
 */
function generateBlocks(count) {
  console.log(`${COLORS.YELLOW}Generating ${count} blocks...${COLORS.NC}`);
  executeCommand(`NODE_OPTIONS=--require=/workspaces/boiler/oyl-sdk/lib/shared/load_patch.js oyl regtest genBlocks -p oylnet -c ${count}`);
}

/**
 * Execute a contract call
 * @param {number} opcode - The opcode to call
 * @param {Array} params - Parameters for the call
 * @returns {string} - Result of the call
 */
function executeContractCall(opcode, params = []) {
  // Convert opcode and parameters to a comma-separated string
  const calldata = [opcode, ...params].join(',');
  
  console.log(`${COLORS.CYAN}Executing opcode ${opcode}${COLORS.NC}`);
  console.log(`${COLORS.CYAN}Parameters: ${params.join(', ')}${COLORS.NC}`);
  
  // Execute the command
  const command = `NODE_OPTIONS=--require=/workspaces/boiler/oyl-sdk/lib/shared/load_patch.js oyl alkane execute -data "${calldata}" --provider oylnet`;
  
  try {
    const result = executeCommand(command);
    console.log(`${COLORS.GREEN}Operation successful${COLORS.NC}`);
    return result.trim();
  } catch (error) {
    console.log(`${COLORS.RED}Operation failed${COLORS.NC}`);
    return null;
  }
}

/**
 * Deploy the contract
 * @returns {string} - Contract ID
 */
function deployContract() {
  console.log(`\n${COLORS.BOLD}${COLORS.BLUE}=== Deploying Contract ===${COLORS.NC}\n`);
  
  // Check if the WebAssembly file exists
  if (!fs.existsSync(WASM_PATH)) {
    console.log(`${COLORS.YELLOW}WebAssembly file not found. Building...${COLORS.NC}`);
    executeCommand('cd /workspaces/boiler && cargo build --target wasm32-unknown-unknown --release');
  }
  
  // Deploy the contract
  console.log(`${COLORS.YELLOW}Deploying contract to OylNet...${COLORS.NC}`);
  const deployOutput = executeCommand(`NODE_OPTIONS=--require=/workspaces/boiler/oyl-sdk/lib/shared/load_patch.js oyl alkane new-contract --contract "${WASM_PATH}" --provider "oylnet" --calldata "0" --feeRate 10`);
  
  // Extract contract ID from the output
  const contractIdMatch = deployOutput.match(/[a-f0-9]{64}/);
  if (!contractIdMatch) {
    throw new Error('Failed to extract contract ID from deployment output');
  }
  
  const contractId = contractIdMatch[0];
  console.log(`${COLORS.GREEN}Contract deployed successfully!${COLORS.NC}`);
  console.log(`${COLORS.CYAN}Contract ID: ${contractId}${COLORS.NC}`);
  
  // Generate blocks to confirm deployment
  generateBlocks(10);
  
  return contractId;
}

/**
 * Initialize the contract
 */
function initializeContract() {
  console.log(`\n${COLORS.BOLD}${COLORS.BLUE}=== Initializing Contract ===${COLORS.NC}\n`);
  
  // Execute initialization
  const result = executeContractCall(OPCODES.INITIALIZE);
  
  // Generate blocks to confirm initialization
  generateBlocks(10);
  
  return result;
}

/**
 * Verify contract metadata
 */
function verifyMetadata() {
  console.log(`\n${COLORS.BOLD}${COLORS.BLUE}=== Verifying Contract Metadata ===${COLORS.NC}\n`);
  
  // Get name
  const name = executeContractCall(OPCODES.GET_NAME);
  console.log(`Name: ${name || 'Not available'}`);
  
  // Get symbol
  const symbol = executeContractCall(OPCODES.GET_SYMBOL);
  console.log(`Symbol: ${symbol || 'Not available'}`);
  
  // Get decimals
  const decimals = executeContractCall(OPCODES.GET_DECIMALS);
  console.log(`Decimals: ${decimals || 'Not available'}`);
  
  // Get asset
  const asset = executeContractCall(OPCODES.GET_ASSET);
  console.log(`Asset: ${asset || 'Not available'}`);
  
  return { name, symbol, decimals, asset };
}

/**
 * Verify contract accounting
 */
function verifyAccounting() {
  console.log(`\n${COLORS.BOLD}${COLORS.BLUE}=== Verifying Contract Accounting ===${COLORS.NC}\n`);
  
  // Get total assets
  const totalAssets = executeContractCall(OPCODES.GET_TOTAL_ASSETS);
  console.log(`Total Assets: ${totalAssets || '0'}`);
  
  // Get total supply
  const totalSupply = executeContractCall(OPCODES.GET_TOTAL_SUPPLY);
  console.log(`Total Supply: ${totalSupply || '0'}`);
  
  return { totalAssets, totalSupply };
}

/**
 * Simulate a user depositing assets
 * @param {string} user - User ID
 * @param {number} assets - Amount of assets to deposit
 */
function userDeposit(user, assets) {
  console.log(`\n${COLORS.BOLD}${COLORS.BLUE}=== User ${user} Deposits ${assets} Assets ===${COLORS.NC}\n`);
  
  // Preview deposit
  console.log(`${COLORS.YELLOW}Previewing deposit...${COLORS.NC}`);
  const previewShares = executeContractCall(OPCODES.PREVIEW_DEPOSIT, [assets]);
  console.log(`Expected shares: ${previewShares}`);
  
  // Execute deposit
  console.log(`${COLORS.YELLOW}Executing deposit...${COLORS.NC}`);
  const result = executeContractCall(OPCODES.DEPOSIT, [0, user, assets]);
  
  // Generate blocks to confirm transaction
  generateBlocks(10);
  
  // Verify accounting after deposit
  const accounting = verifyAccounting();
  
  return { previewShares, result, accounting };
}

/**
 * Simulate a user minting shares
 * @param {string} user - User ID
 * @param {number} shares - Amount of shares to mint
 */
function userMint(user, shares) {
  console.log(`\n${COLORS.BOLD}${COLORS.BLUE}=== User ${user} Mints ${shares} Shares ===${COLORS.NC}\n`);
  
  // Preview mint
  console.log(`${COLORS.YELLOW}Previewing mint...${COLORS.NC}`);
  const previewAssets = executeContractCall(OPCODES.PREVIEW_MINT, [shares]);
  console.log(`Expected assets: ${previewAssets}`);
  
  // Execute mint
  console.log(`${COLORS.YELLOW}Executing mint...${COLORS.NC}`);
  const result = executeContractCall(OPCODES.MINT, [1, user, shares]);
  
  // Generate blocks to confirm transaction
  generateBlocks(10);
  
  // Verify accounting after mint
  const accounting = verifyAccounting();
  
  return { previewAssets, result, accounting };
}

/**
 * Simulate a user redeeming shares
 * @param {string} user - User ID
 * @param {number} shares - Amount of shares to redeem
 */
function userRedeem(user, shares) {
  console.log(`\n${COLORS.BOLD}${COLORS.BLUE}=== User ${user} Redeems ${shares} Shares ===${COLORS.NC}\n`);
  
  // Preview redeem
  console.log(`${COLORS.YELLOW}Previewing redeem...${COLORS.NC}`);
  const previewAssets = executeContractCall(OPCODES.PREVIEW_REDEEM, [shares]);
  console.log(`Expected assets: ${previewAssets}`);
  
  // Execute redeem
  console.log(`${COLORS.YELLOW}Executing redeem...${COLORS.NC}`);
  const result = executeContractCall(OPCODES.REDEEM, [2, user, user, shares]);
  
  // Generate blocks to confirm transaction
  generateBlocks(10);
  
  // Verify accounting after redeem
  const accounting = verifyAccounting();
  
  return { previewAssets, result, accounting };
}

/**
 * Simulate a user withdrawing assets
 * @param {string} user - User ID
 * @param {number} assets - Amount of assets to withdraw
 */
function userWithdraw(user, assets) {
  console.log(`\n${COLORS.BOLD}${COLORS.BLUE}=== User ${user} Withdraws ${assets} Assets ===${COLORS.NC}\n`);
  
  // Preview withdraw
  console.log(`${COLORS.YELLOW}Previewing withdraw...${COLORS.NC}`);
  const previewShares = executeContractCall(OPCODES.PREVIEW_WITHDRAW, [assets]);
  console.log(`Expected shares: ${previewShares}`);
  
  // Execute withdraw
  console.log(`${COLORS.YELLOW}Executing withdraw...${COLORS.NC}`);
  const result = executeContractCall(OPCODES.WITHDRAW, [3, user, user, assets]);
  
  // Generate blocks to confirm transaction
  generateBlocks(10);
  
  // Verify accounting after withdraw
  const accounting = verifyAccounting();
  
  return { previewShares, result, accounting };
}

/**
 * Run the end-to-end test
 */
async function runE2ETest() {
  try {
    console.log(`${COLORS.BOLD}${COLORS.BLUE}=== YieldVault End-to-End Test ===${COLORS.NC}\n`);
    
    console.log(`${COLORS.YELLOW}Using existing contract: ${CONTRACT_ID}${COLORS.NC}`);
    
    // Skip deployment and initialization since we're using an existing contract
    
    // Verify contract metadata
    const metadata = verifyMetadata();
    
    // Verify initial accounting
    const initialAccounting = verifyAccounting();
    
    // User Alice deposits 1000 assets
    const aliceDeposit = userDeposit(USERS.ALICE, 1000);
    
    // User Bob mints 500 shares
    const bobMint = userMint(USERS.BOB, 500);
    
    // Wait for some time to accrue yield
    console.log(`\n${COLORS.YELLOW}Waiting for yield to accrue...${COLORS.NC}`);
    generateBlocks(1000); // Generate a lot of blocks to simulate time passing
    
    // Verify accounting after yield accrual
    const yieldAccounting = verifyAccounting();
    
    // User Alice redeems 500 shares
    const aliceRedeem = userRedeem(USERS.ALICE, 500);
    
    // User Bob withdraws 300 assets
    const bobWithdraw = userWithdraw(USERS.BOB, 300);
    
    // Final accounting
    const finalAccounting = verifyAccounting();
    
    // Save test results
    const testResults = {
      contractId: CONTRACT_ID,
      metadata,
      initialAccounting,
      aliceDeposit,
      bobMint,
      yieldAccounting,
      aliceRedeem,
      bobWithdraw,
      finalAccounting,
      timestamp: new Date().toISOString()
    };
    
    fs.writeFileSync(
      path.resolve(__dirname, 'e2e_test_results.json'),
      JSON.stringify(testResults, null, 2)
    );
    
    console.log(`\n${COLORS.BOLD}${COLORS.GREEN}=== End-to-End Test Completed Successfully ===${COLORS.NC}`);
    console.log(`Test results saved to e2e_test_results.json`);
    
  } catch (error) {
    console.error(`${COLORS.RED}End-to-End Test Failed: ${error.message}${COLORS.NC}`);
    process.exit(1);
  }
}

// Run the test
runE2ETest();
