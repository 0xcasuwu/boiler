/**
 * YieldVault Network Operations Test Script
 * 
 * This script tests the complete flow:
 * 1. Generate an account
 * 2. Load the account
 * 3. Fund the account
 * 4. Deploy the vault contract
 */

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');
const crypto = require('crypto');

// Import our structured network implementation
const { NetworkUtils, YieldVaultNetwork } = require('./bin/net/structured_network');

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

// Root directory
const ROOT_DIR = path.resolve(__dirname);

/**
 * Step 1: Generate a new Bitcoin account
 * @returns {Object} Account information
 */
async function generateAccount() {
  console.log(`\n${COLORS.BOLD}${COLORS.BLUE}Step 1: Generating Bitcoin Account${COLORS.NC}`);
  
  try {
    // Generate a random mnemonic (in a real scenario, this would be securely generated)
    // For demo purposes, we'll use a test mnemonic
    const mnemonic = "test test test test test test test test test test test junk";
    
    // Create a simple wallet info object
    const walletInfo = {
      mnemonic,
      address: "bcrt1qeyyk6sl5gvr4wzm0dpmfqcjsls9xfkgvurkz7p", // Test address compatible with regtest
      timestamp: new Date().toISOString()
    };
    
    // Save wallet info to file
    fs.writeFileSync(path.join(ROOT_DIR, 'wallet_info.json'), JSON.stringify(walletInfo, null, 2));
    
    // Update .env file with mnemonic
    let envContent = '';
    if (fs.existsSync(path.join(ROOT_DIR, '.env'))) {
      envContent = fs.readFileSync(path.join(ROOT_DIR, '.env'), 'utf8');
    }
    
    // Add or update MNEMONIC in .env
    if (envContent.includes('MNEMONIC=')) {
      envContent = envContent.replace(/MNEMONIC=.*(\r?\n|$)/g, `MNEMONIC="${mnemonic}"$1`);
    } else {
      envContent += `\nMNEMONIC="${mnemonic}"\n`;
    }
    
    fs.writeFileSync(path.join(ROOT_DIR, '.env'), envContent);
    
    console.log(`${COLORS.GREEN}✅ Account generated successfully${COLORS.NC}`);
    console.log(`${COLORS.BLUE}Address: ${COLORS.NC}${walletInfo.address}`);
    
    return walletInfo;
  } catch (error) {
    console.log(`${COLORS.RED}❌ Failed to generate account:${COLORS.NC} ${error.message}`);
    throw error;
  }
}

/**
 * Step 2: Load the Bitcoin account
 * @returns {Object} Account information
 */
async function loadAccount() {
  console.log(`\n${COLORS.BOLD}${COLORS.BLUE}Step 2: Loading Bitcoin Account${COLORS.NC}`);
  
  try {
    // Check if wallet info exists
    const walletInfoPath = path.join(ROOT_DIR, 'wallet_info.json');
    if (!fs.existsSync(walletInfoPath)) {
      throw new Error('No wallet info found. Generate an account first.');
    }
    
    // Load wallet info
    const walletInfo = JSON.parse(fs.readFileSync(walletInfoPath, 'utf8'));
    
    console.log(`${COLORS.GREEN}✅ Account loaded successfully${COLORS.NC}`);
    console.log(`${COLORS.BLUE}Address: ${COLORS.NC}${walletInfo.address}`);
    
    return walletInfo;
  } catch (error) {
    console.log(`${COLORS.RED}❌ Failed to load account:${COLORS.NC} ${error.message}`);
    throw error;
  }
}

/**
 * Step 3: Fund the Bitcoin account
 * @param {Object} walletInfo Account information
 * @returns {boolean} Success status
 */
async function fundAccount(walletInfo) {
  console.log(`\n${COLORS.BOLD}${COLORS.BLUE}Step 3: Funding Bitcoin Account${COLORS.NC}`);
  
  try {
    // Test connection to OylNet first
    const connected = await YieldVaultNetwork.testConnection();
    if (!connected) {
      throw new Error('Cannot connect to OylNet network');
    }
    
    console.log(`${COLORS.BLUE}Requesting funds for address: ${COLORS.NC}${walletInfo.address}`);
    
    // Fund the wallet
    const success = await YieldVaultNetwork.fundWallet();
    
    if (!success) {
      throw new Error('Failed to fund wallet');
    }
    
    console.log(`${COLORS.GREEN}✅ Account funded successfully${COLORS.NC}`);
    
    return true;
  } catch (error) {
    console.log(`${COLORS.RED}❌ Failed to fund account:${COLORS.NC} ${error.message}`);
    throw error;
  }
}

/**
 * Step 4: Deploy the vault contract
 * @returns {string|null} Contract ID if successful
 */
async function deployContract() {
  console.log(`\n${COLORS.BOLD}${COLORS.BLUE}Step 4: Deploying Vault Contract${COLORS.NC}`);
  
  try {
    // Build the contract first if needed
    const wasmPath = path.join(ROOT_DIR, 'target/wasm32-unknown-unknown/release/yield_vault.wasm');
    
    if (!fs.existsSync(wasmPath)) {
      console.log(`${COLORS.YELLOW}⚠️ WASM file not found. Building the contract...${COLORS.NC}`);
      execSync('cargo build --target wasm32-unknown-unknown --release', { stdio: 'inherit' });
    }
    
    // Deploy the contract
    const success = await YieldVaultNetwork.deployContract();
    
    if (!success) {
      throw new Error('Contract deployment failed');
    }
    
    // Read the contract ID
    const contractIdPath = path.join(ROOT_DIR, '.contract_id');
    if (!fs.existsSync(contractIdPath)) {
      throw new Error('Contract ID file not found after deployment');
    }
    
    const contractId = fs.readFileSync(contractIdPath, 'utf8').trim();
    
    console.log(`${COLORS.GREEN}✅ Contract deployed successfully${COLORS.NC}`);
    console.log(`${COLORS.BLUE}Contract ID: ${COLORS.NC}${contractId}`);
    
    return contractId;
  } catch (error) {
    console.log(`${COLORS.RED}❌ Failed to deploy contract:${COLORS.NC} ${error.message}`);
    throw error;
  }
}

/**
 * Execute the complete test flow
 */
async function runTests() {
  console.log(`${COLORS.BOLD}${COLORS.BLUE}=== YieldVault Network Operations Test ====${COLORS.NC}\n`);
  
  try {
    // Step 1: Generate an account
    const walletInfo = await generateAccount();
    
    // Step 2: Load the account
    await loadAccount();
    
    // Step 3: Fund the account
    await fundAccount(walletInfo);
    
    // Step 4: Deploy the contract
    const contractId = await deployContract();
    
    // Test complete!
    console.log(`\n${COLORS.BOLD}${COLORS.GREEN}=== Test Completed Successfully! ====${COLORS.NC}`);
    console.log(`${COLORS.BLUE}Summary:${COLORS.NC}`);
    console.log(`- Account Address: ${walletInfo.address}`);
    console.log(`- Contract ID: ${contractId}`);
    
    return true;
  } catch (error) {
    console.log(`\n${COLORS.BOLD}${COLORS.RED}=== Test Failed! ====${COLORS.NC}`);
    console.log(`${COLORS.RED}Error: ${error.message}${COLORS.NC}`);
    return false;
  }
}

// Run the tests
runTests().then(success => {
  process.exit(success ? 0 : 1);
});
