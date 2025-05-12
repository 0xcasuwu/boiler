/**
 * Generate a compatible OylNet regtest address
 * 
 * This script uses the OylNet SDK to generate an address that's compatible
 * with the OylNet regtest environment.
 */

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

// Root directory
const ROOT_DIR = path.resolve(__dirname);

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

/**
 * Generate a compatible regtest address using oyl CLI directly
 */
function generateRegtestAddress() {
  console.log(`${COLORS.BOLD}${COLORS.BLUE}Generating Compatible OylNet Regtest Address${COLORS.NC}\n`);
  
  try {
    // Execute the oyl CLI command to generate an address
    const command = `/bin/bash -c "source ${ROOT_DIR}/.env && oyl regtest generateAddress -p oylnet"`;
    console.log(`${COLORS.CYAN}$ ${command}${COLORS.NC}`);
    
    const result = execSync(command, { encoding: 'utf8' });
    console.log(`${COLORS.GREEN}✅ Address generated successfully${COLORS.NC}`);
    
    // Parse the output to extract the address
    const addressMatch = result.match(/address:\s*['"]?([a-zA-Z0-9]+)['"]?/i);
    
    if (addressMatch && addressMatch[1]) {
      const address = addressMatch[1];
      console.log(`${COLORS.BLUE}Generated address: ${COLORS.BOLD}${address}${COLORS.NC}`);
      
      // Create a simple wallet info object
      const walletInfo = {
        address,
        networkType: 'regtest',
        timestamp: new Date().toISOString()
      };
      
      // Save wallet info to file
      fs.writeFileSync(path.join(ROOT_DIR, 'wallet_info.json'), JSON.stringify(walletInfo, null, 2));
      console.log(`${COLORS.GREEN}✅ Address saved to wallet_info.json${COLORS.NC}`);
      
      // Update .env file with address
      let envContent = '';
      if (fs.existsSync(path.join(ROOT_DIR, '.env'))) {
        envContent = fs.readFileSync(path.join(ROOT_DIR, '.env'), 'utf8');
      }
      
      // Add or update FUNDED_ADDRESS in .env
      if (envContent.includes('FUNDED_ADDRESS=')) {
        envContent = envContent.replace(/FUNDED_ADDRESS=.*(\r?\n|$)/g, `FUNDED_ADDRESS="${address}"$1`);
      } else {
        envContent += `\nFUNDED_ADDRESS="${address}"\n`;
      }
      
      fs.writeFileSync(path.join(ROOT_DIR, '.env'), envContent);
      console.log(`${COLORS.GREEN}✅ .env file updated with FUNDED_ADDRESS${COLORS.NC}`);
      
      return address;
    } else {
      console.log(`${COLORS.RED}❌ Could not extract address from CLI output${COLORS.NC}`);
      console.log('Output:');
      console.log(result);
      return null;
    }
  } catch (error) {
    console.log(`${COLORS.RED}❌ Failed to generate address:${COLORS.NC} ${error.message}`);
    console.log('Error output:');
    console.log(error.stdout || error.stderr || error.message);
    return null;
  }
}

// Execute the function
const address = generateRegtestAddress();

if (address) {
  console.log(`\n${COLORS.BLUE}Next steps:${COLORS.NC}`);
  console.log(`1. Use this address (${address}) for funding operations`);
  console.log(`2. Run the test script: node test_network_operations.js`);
}
