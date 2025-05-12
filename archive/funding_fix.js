/**
 * OylNet Funding Fix
 * 
 * This script diagnoses and fixes the funding issues with OylNet deployment.
 * It initializes the regtest chain with more blocks and directly targets
 * funding the deployment account.
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

// Root directory
const ROOT_DIR = path.resolve(__dirname);

/**
 * Generate a properly funded environment
 */
async function fixFunding() {
  displaySectionHeader("OylNet Funding Fix");
  
  // Step 1: Load our address patch
  console.log(`${COLORS.YELLOW}1. Loading OylNet address compatibility patch${COLORS.NC}`);
  
  try {
    // Force require our patch
    require('./oyl-sdk/lib/shared/load_patch');
    console.log(`${COLORS.GREEN}✅ Address patch loaded${COLORS.NC}`);
  } catch (error) {
    console.log(`${COLORS.RED}❌ Failed to load address patch:${COLORS.NC} ${error.message}`);
    return false;
  }
  
  // Step 2: Getting information about the default account used for deployment
  console.log(`\n${COLORS.YELLOW}2. Finding deployment account details${COLORS.NC}`);
  
  // Let's check what account the CLI uses for deployment
  const accountInfoCommand = `/bin/bash -c "source ${ROOT_DIR}/.env && NODE_OPTIONS=--require=./oyl-sdk/lib/shared/load_patch.js oyl account info -p oylnet"`;
  console.log(`${COLORS.CYAN}$ ${accountInfoCommand}${COLORS.NC}`);
  
  let deploymentAddress = '';
  try {
    const accountInfo = execSync(accountInfoCommand, { encoding: 'utf8' });
    console.log(`${COLORS.GREEN}✅ Account info retrieved${COLORS.NC}`);
    console.log(accountInfo);
    
    // Extract the address for use in funding
    const addressMatch = accountInfo.match(/"address":\s*"([^"]+)"/);
    if (addressMatch && addressMatch[1]) {
      deploymentAddress = addressMatch[1];
      console.log(`${COLORS.GREEN}✅ Found deployment address: ${deploymentAddress}${COLORS.NC}`);
    } else {
      console.log(`${COLORS.YELLOW}⚠️ Could not extract address from account info${COLORS.NC}`);
      // Fallback to a known good address format
      deploymentAddress = "bcrt1qeyyk6sl5gvr4wzm0dpmfqcjsls9xfkgvurkz7p";
      console.log(`${COLORS.YELLOW}⚠️ Using fallback address: ${deploymentAddress}${COLORS.NC}`);
    }
  } catch (error) {
    console.log(`${COLORS.RED}❌ Failed to get account info:${COLORS.NC} ${error.message}`);
    // Fallback to a known good address format
    deploymentAddress = "bcrt1qeyyk6sl5gvr4wzm0dpmfqcjsls9xfkgvurkz7p";
    console.log(`${COLORS.YELLOW}⚠️ Using fallback address: ${deploymentAddress}${COLORS.NC}`);
  }
  
  // Step 3: Reinitialize the regtest environment with more blocks directly to our address
  console.log(`\n${COLORS.YELLOW}3. Reinitializing regtest environment with more blocks${COLORS.NC}`);
  
  // First, attempt a clean initialization with our patched address handling
  const initCommand = `/bin/bash -c "source ${ROOT_DIR}/.env && NODE_OPTIONS=--require=./oyl-sdk/lib/shared/load_patch.js oyl regtest init -p oylnet --to '${deploymentAddress}' --blocks 200"`;
  console.log(`${COLORS.CYAN}$ ${initCommand}${COLORS.NC}`);
  
  try {
    const initResult = execSync(initCommand, { encoding: 'utf8' });
    console.log(`${COLORS.GREEN}✅ Regtest reinitialized with 200 blocks${COLORS.NC}`);
    console.log(initResult);
  } catch (error) {
    console.log(`${COLORS.YELLOW}⚠️ Reinitialization returned an error (might be already initialized)${COLORS.NC}`);
    console.log(error.message);
    
    // Generate a large number of blocks to ensure the faucet has coins
    const genCommand = `/bin/bash -c "source ${ROOT_DIR}/.env && NODE_OPTIONS=--require=./oyl-sdk/lib/shared/load_patch.js oyl regtest genBlocks -p oylnet -c 100"`;
    console.log(`${COLORS.CYAN}$ ${genCommand}${COLORS.NC}`);
    
    try {
      const genResult = execSync(genCommand, { encoding: 'utf8' });
      console.log(`${COLORS.GREEN}✅ Generated 100 additional blocks${COLORS.NC}`);
      console.log(genResult);
    } catch (genError) {
      console.log(`${COLORS.RED}❌ Failed to generate blocks:${COLORS.NC} ${genError.message}`);
    }
  }
  
  // Step 4: Direct funding with multiple attempts
  console.log(`\n${COLORS.YELLOW}4. Attempting multiple funding operations to address ${deploymentAddress}${COLORS.NC}`);
  
  // Attempt 1: Standard funding with our patched address handling - higher amount
  const fundCommand1 = `/bin/bash -c "source ${ROOT_DIR}/.env && NODE_OPTIONS=--require=./oyl-sdk/lib/shared/load_patch.js oyl regtest sendFromFaucet -p oylnet -t '${deploymentAddress}' -s 500000000"`;
  console.log(`${COLORS.CYAN}$ ${fundCommand1}${COLORS.NC}`);
  
  try {
    const fundResult1 = execSync(fundCommand1, { encoding: 'utf8' });
    console.log(`${COLORS.GREEN}✅ Funding operation 1 successful${COLORS.NC}`);
    console.log(fundResult1);
  } catch (error) {
    console.log(`${COLORS.RED}❌ Funding operation 1 failed:${COLORS.NC} ${error.message}`);
  }
  
  // Generate some blocks to confirm transactions
  console.log(`\n${COLORS.YELLOW}Generating 10 blocks to confirm transactions${COLORS.NC}`);
  const confirmCommand = `/bin/bash -c "source ${ROOT_DIR}/.env && NODE_OPTIONS=--require=./oyl-sdk/lib/shared/load_patch.js oyl regtest genBlocks -p oylnet -c 10"`;
  
  try {
    const confirmResult = execSync(confirmCommand, { encoding: 'utf8' });
    console.log(`${COLORS.GREEN}✅ Generated confirmation blocks${COLORS.NC}`);
    console.log(confirmResult);
  } catch (error) {
    console.log(`${COLORS.YELLOW}⚠️ Block generation returned an error:${COLORS.NC} ${error.message}`);
  }
  
  // Attempt 2: Try a second funding operation
  console.log(`\n${COLORS.YELLOW}Attempting second funding operation${COLORS.NC}`);
  const fundCommand2 = `/bin/bash -c "source ${ROOT_DIR}/.env && NODE_OPTIONS=--require=./oyl-sdk/lib/shared/load_patch.js oyl regtest sendFromFaucet -p oylnet -t '${deploymentAddress}' -s 500000000"`;
  console.log(`${COLORS.CYAN}$ ${fundCommand2}${COLORS.NC}`);
  
  try {
    const fundResult2 = execSync(fundCommand2, { encoding: 'utf8' });
    console.log(`${COLORS.GREEN}✅ Funding operation 2 successful${COLORS.NC}`);
    console.log(fundResult2);
  } catch (error) {
    console.log(`${COLORS.YELLOW}⚠️ Funding operation 2 failed:${COLORS.NC} ${error.message}`);
  }
  
  // Generate more blocks to confirm transactions
  console.log(`\n${COLORS.YELLOW}Generating 10 more blocks to confirm transactions${COLORS.NC}`);
  
  try {
    const confirmResult2 = execSync(confirmCommand, { encoding: 'utf8' });
    console.log(`${COLORS.GREEN}✅ Generated additional confirmation blocks${COLORS.NC}`);
    console.log(confirmResult2);
  } catch (error) {
    console.log(`${COLORS.YELLOW}⚠️ Block generation returned an error:${COLORS.NC} ${error.message}`);
  }
  
  // Step 5: Update the .env file with the deployment address
  console.log(`\n${COLORS.YELLOW}5. Updating .env file with funded address${COLORS.NC}`);
  
  // Read the current .env file
  let envContent = '';
  if (fs.existsSync(`${ROOT_DIR}/.env`)) {
    envContent = fs.readFileSync(`${ROOT_DIR}/.env`, 'utf8');
  } else {
    envContent = "PROVIDER=oylnet\n";
  }
  
  // Add or update FUNDED_ADDRESS in .env
  if (envContent.includes('FUNDED_ADDRESS=')) {
    envContent = envContent.replace(/FUNDED_ADDRESS=.*\n/g, `FUNDED_ADDRESS="${deploymentAddress}"\n`);
  } else {
    envContent += `\nFUNDED_ADDRESS="${deploymentAddress}"\n`;
  }
  
  // Write updated .env file
  fs.writeFileSync(`${ROOT_DIR}/.env`, envContent);
  console.log(`${COLORS.GREEN}✅ .env file updated with funded address${COLORS.NC}`);
  
  // Step 6: Try to get account balance to verify funding
  console.log(`\n${COLORS.YELLOW}6. Verifying account balance${COLORS.NC}`);
  const balanceCommand = `/bin/bash -c "source ${ROOT_DIR}/.env && NODE_OPTIONS=--require=./oyl-sdk/lib/shared/load_patch.js oyl account balance -p oylnet"`;
  console.log(`${COLORS.CYAN}$ ${balanceCommand}${COLORS.NC}`);
  
  try {
    const balanceResult = execSync(balanceCommand, { encoding: 'utf8' });
    console.log(`${COLORS.GREEN}✅ Account balance retrieved${COLORS.NC}`);
    console.log(balanceResult);
    
    // Check if we have sufficient balance
    const balanceMatch = balanceResult.match(/total.*?:\s*(\d+)/i);
    if (balanceMatch && balanceMatch[1]) {
      const balance = parseInt(balanceMatch[1], 10);
      if (balance > 0) {
        console.log(`${COLORS.GREEN}✅ Account has a positive balance of ${balance} satoshis${COLORS.NC}`);
      } else {
        console.log(`${COLORS.RED}❌ Account balance is zero or not found${COLORS.NC}`);
      }
    }
  } catch (error) {
    console.log(`${COLORS.RED}❌ Failed to get account balance:${COLORS.NC} ${error.message}`);
  }
  
  console.log(`\n${COLORS.BOLD}${COLORS.GREEN}=== Funding Operations Complete ====${COLORS.NC}`);
  console.log(`Deployment address ${deploymentAddress} should now have more funds for contract deployment.`);
  console.log(`To deploy the contract, run: node deploy_with_patch.js`);
}

/**
 * Display a section header
 */
function displaySectionHeader(title) {
  console.log(`\n${COLORS.BOLD}${COLORS.BLUE}=== ${title} ====${COLORS.NC}\n`);
}

// Execute the main function
fixFunding();
