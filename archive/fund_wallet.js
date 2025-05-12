/**
 * Fund Wallet Script
 * 
 * This script initializes the regtest environment and funds a wallet
 * for contract deployment.
 */

const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

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

// Default address to use if none is provided
const DEFAULT_ADDRESS = "bcrt1qeyyk6sl5gvr4wzm0dpmfqcjsls9xfkgvurkz7p";

/**
 * Initialize the regtest environment and fund a wallet
 */
function fundWallet() {
  console.log(`${COLORS.BOLD}${COLORS.BLUE}Funding Wallet for Contract Deployment${COLORS.NC}\n`);
  
  try {
    // Step 1: Check .env file
    console.log(`${COLORS.YELLOW}1. Checking environment settings${COLORS.NC}`);
    if (!fs.existsSync(path.join(ROOT_DIR, '.env'))) {
      console.log(`${COLORS.RED}❌ .env file not found${COLORS.NC}`);
      console.log(`Creating basic .env file...`);
      fs.writeFileSync(path.join(ROOT_DIR, '.env'), `PROVIDER=oylnet\n`);
    }
    
    // Get currently used address from .env or use default
    let envContent = fs.readFileSync(path.join(ROOT_DIR, '.env'), 'utf8');
    let address = (envContent.match(/FUNDED_ADDRESS\s*=\s*["']?([^"'\n]+)["']?/) || [])[1] || DEFAULT_ADDRESS;
    
    console.log(`${COLORS.BLUE}Using address: ${address}${COLORS.NC}`);
    
    // Step 2: Initialize the regtest environment
    console.log(`\n${COLORS.YELLOW}2. Initializing regtest environment${COLORS.NC}`);
    const initCommand = `/bin/bash -c "source ${ROOT_DIR}/.env && oyl regtest init -p oylnet --to '${address}' --blocks 20"`;
    console.log(`${COLORS.CYAN}$ ${initCommand}${COLORS.NC}`);
    
    try {
      const initResult = execSync(initCommand, { encoding: 'utf8' });
      console.log(`${COLORS.GREEN}✅ Regtest initialized successfully${COLORS.NC}`);
      console.log('Output:');
      console.log(initResult);
    } catch (error) {
      // If we get an error saying the chain is already initialized or similar, continue
      console.log(`${COLORS.YELLOW}⚠️ Regtest initialization returned an error${COLORS.NC}`);
      console.log('Error:');
      console.log(error.message);
      
      // Generate some blocks anyway
      console.log(`\n${COLORS.YELLOW}Generating additional blocks...${COLORS.NC}`);
      const genCommand = `/bin/bash -c "source ${ROOT_DIR}/.env && oyl regtest genBlocks -p oylnet -c 20"`;
      try {
        const genResult = execSync(genCommand, { encoding: 'utf8' });
        console.log(`${COLORS.GREEN}✅ Additional blocks generated${COLORS.NC}`);
        console.log('Output:');
        console.log(genResult);
      } catch (genError) {
        console.log(`${COLORS.RED}❌ Failed to generate blocks${COLORS.NC}`);
        console.log('Error:');
        console.log(genError.message);
      }
    }
    
    // Step 3: Send funds from faucet
    console.log(`\n${COLORS.YELLOW}3. Sending funds from faucet${COLORS.NC}`);
    const fundCommand = `/bin/bash -c "source ${ROOT_DIR}/.env && oyl regtest sendFromFaucet -p oylnet -t '${address}' -s 100000000"`;
    console.log(`${COLORS.CYAN}$ ${fundCommand}${COLORS.NC}`);
    
    try {
      const fundResult = execSync(fundCommand, { encoding: 'utf8' });
      console.log(`${COLORS.GREEN}✅ Funds sent from faucet${COLORS.NC}`);
      console.log('Output:');
      console.log(fundResult);
      
      // Extract transaction IDs if available
      const txIds = extractTxIds(fundResult);
      if (txIds.length > 0) {
        console.log(`${COLORS.BLUE}Transaction IDs: ${txIds.join(', ')}${COLORS.NC}`);
      }
    } catch (error) {
      console.log(`${COLORS.RED}❌ Failed to send funds from faucet${COLORS.NC}`);
      console.log('Error:');
      console.log(error.message);
      
      if (error.message.includes('has no matching Script')) {
        console.log(`\n${COLORS.YELLOW}The address format appears to be incompatible with OylNet.${COLORS.NC}`);
        console.log(`${COLORS.YELLOW}Try using a different address format or generating a new one.${COLORS.NC}`);
      }
      
      return false;
    }
    
    // Step 4: Generate blocks to confirm funding
    console.log(`\n${COLORS.YELLOW}4. Generating blocks to confirm funding${COLORS.NC}`);
    const confirmCommand = `/bin/bash -c "source ${ROOT_DIR}/.env && oyl regtest genBlocks -p oylnet -c 10"`;
    console.log(`${COLORS.CYAN}$ ${confirmCommand}${COLORS.NC}`);
    
    try {
      const confirmResult = execSync(confirmCommand, { encoding: 'utf8' });
      console.log(`${COLORS.GREEN}✅ Confirmation blocks generated${COLORS.NC}`);
      console.log('Output:');
      console.log(confirmResult);
    } catch (error) {
      console.log(`${COLORS.RED}❌ Failed to generate confirmation blocks${COLORS.NC}`);
      console.log('Error:');
      console.log(error.message);
      return false;
    }
    
    // Success!
    console.log(`\n${COLORS.BOLD}${COLORS.GREEN}✅ Wallet Funding Complete${COLORS.NC}`);
    console.log(`${COLORS.GREEN}The address ${address} should now have sufficient funds for contract deployment.${COLORS.NC}`);
    
    // Update .env file with funded address if not already there
    if (!envContent.includes('FUNDED_ADDRESS=')) {
      envContent += `\nFUNDED_ADDRESS="${address}"\n`;
      fs.writeFileSync(path.join(ROOT_DIR, '.env'), envContent);
      console.log(`${COLORS.GREEN}✅ .env file updated with FUNDED_ADDRESS${COLORS.NC}`);
    }
    
    return true;
  } catch (error) {
    console.log(`${COLORS.RED}❌ Unexpected error during wallet funding:${COLORS.NC} ${error.message}`);
    return false;
  }
}

/**
 * Extract transaction IDs from command output
 * @param {string} output - Command output
 * @returns {string[]} - Array of transaction IDs
 */
function extractTxIds(output) {
  const txIdMatches = output.match(/"txid":\s*"([0-9a-f]+)"/g) || [];
  return txIdMatches.map(match => {
    const innerMatch = match.match(/"txid":\s*"([0-9a-f]+)"/);
    return innerMatch ? innerMatch[1] : null;
  }).filter(Boolean);
}

// Execute the function
const success = fundWallet();
process.exit(success ? 0 : 1);
