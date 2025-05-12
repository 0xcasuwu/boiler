/**
 * Simple Legacy Address Funding Test
 * 
 * This script creates a legacy P2PKH address and attempts to fund it with OylNet.
 * It uses a hardcoded address example from the OylNet SDK fixtures.
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

try {
  console.log(`${COLORS.BOLD}${COLORS.BLUE}Simple Legacy P2PKH Address Funding Test${COLORS.NC}\n`);
  
  // Use a known working legacy address from the SDK fixtures
  // This address was found in the address_investigation.log file
  const legacyAddress = 'mxbBHPuZmf8Ve5pBgdbwDjLP1cR3mqZZQM'; // Legacy (P2PKH)
  
  console.log(`${COLORS.YELLOW}Using hardcoded legacy address from SDK fixtures${COLORS.NC}`);
  console.log(`${COLORS.GREEN}Legacy address: ${COLORS.BOLD}${legacyAddress}${COLORS.NC}`);
  
  // Save wallet info
  const walletInfo = {
    address: legacyAddress,
    networkType: 'regtest',
    format: 'P2PKH (Legacy)',
    timestamp: new Date().toISOString(),
    note: 'This is a test address from SDK fixtures. Private key is not available.'
  };
  
  fs.writeFileSync(path.join(ROOT_DIR, 'legacy_wallet.json'), JSON.stringify(walletInfo, null, 2));
  console.log(`${COLORS.GREEN}✅ Wallet info saved to legacy_wallet.json${COLORS.NC}`);
  
  // Update .env file with the new address
  console.log(`\n${COLORS.YELLOW}Updating .env file with legacy address...${COLORS.NC}`);
  let envContent = '';
  if (fs.existsSync(path.join(ROOT_DIR, '.env'))) {
    envContent = fs.readFileSync(path.join(ROOT_DIR, '.env'), 'utf8');
  }
  
  // Add or update LEGACY_ADDRESS in .env
  if (envContent.includes('LEGACY_ADDRESS=')) {
    envContent = envContent.replace(/LEGACY_ADDRESS=.*(\r?\n|$)/g, `LEGACY_ADDRESS="${legacyAddress}"$1`);
  } else {
    envContent += `\nLEGACY_ADDRESS="${legacyAddress}"\n`;
  }
  
  // Also update FUNDED_ADDRESS to use the legacy address
  if (envContent.includes('FUNDED_ADDRESS=')) {
    envContent = envContent.replace(/FUNDED_ADDRESS=.*(\r?\n|$)/g, `FUNDED_ADDRESS="${legacyAddress}"$1`);
  } else {
    envContent += `\nFUNDED_ADDRESS="${legacyAddress}"\n`;
  }
  
  fs.writeFileSync(path.join(ROOT_DIR, '.env'), envContent);
  console.log(`${COLORS.GREEN}✅ .env file updated with legacy address${COLORS.NC}`);
  
  // Try to initialize the regtest chain and generate some blocks
  console.log(`\n${COLORS.YELLOW}Generating blocks...${COLORS.NC}`);
  const genCommand = `/bin/bash -c "source ${ROOT_DIR}/.env && oyl regtest genBlocks -p oylnet -c 10"`;
  console.log(`${COLORS.CYAN}$ ${genCommand}${COLORS.NC}`);
  
  try {
    const genResult = execSync(genCommand, { encoding: 'utf8' });
    console.log(`${COLORS.GREEN}✅ Generated blocks successfully${COLORS.NC}`);
    console.log(genResult);
  } catch (error) {
    console.log(`${COLORS.RED}❌ Failed to generate blocks${COLORS.NC}`);
    console.log(error.message);
    // Continue anyway as this isn't critical
  }
  
  // Try to fund the legacy address
  console.log(`\n${COLORS.YELLOW}Attempting to fund legacy address from faucet...${COLORS.NC}`);
  const fundCommand = `/bin/bash -c "source ${ROOT_DIR}/.env && oyl regtest sendFromFaucet -p oylnet -t '${legacyAddress}' -s 100000000"`;
  console.log(`${COLORS.CYAN}$ ${fundCommand}${COLORS.NC}`);
  
  try {
    const fundResult = execSync(fundCommand, { encoding: 'utf8' });
    console.log(`${COLORS.GREEN}✅ Successfully funded legacy address!${COLORS.NC}`);
    console.log(fundResult);
    
    // Generate blocks to confirm funding
    console.log(`\n${COLORS.YELLOW}Generating blocks to confirm funding...${COLORS.NC}`);
    const confirmCommand = `/bin/bash -c "source ${ROOT_DIR}/.env && oyl regtest genBlocks -p oylnet -c 10"`;
    console.log(`${COLORS.CYAN}$ ${confirmCommand}${COLORS.NC}`);
    
    try {
      const confirmResult = execSync(confirmCommand, { encoding: 'utf8' });
      console.log(`${COLORS.GREEN}✅ Generated confirmation blocks${COLORS.NC}`);
      console.log(confirmResult);
    } catch (error) {
      console.log(`${COLORS.RED}❌ Failed to generate confirmation blocks${COLORS.NC}`);
      console.log(error.message);
    }
    
    // Success!
    console.log(`\n${COLORS.BOLD}${COLORS.GREEN}==== SUCCESS! =====${COLORS.NC}`);
    console.log(`${COLORS.GREEN}The legacy address ${legacyAddress} has been funded!${COLORS.NC}`);
    console.log(`${COLORS.GREEN}You can now use this address for contract deployment.${COLORS.NC}`);
    
    // Try to deploy a contract with the funded address
    console.log(`\n${COLORS.YELLOW}Would you like to attempt contract deployment with this address?${COLORS.NC}`);
    console.log(`${COLORS.YELLOW}Run this command:${COLORS.NC}`);
    console.log(`node deploy_contract.js --address ${legacyAddress}`);
    
  } catch (error) {
    console.log(`${COLORS.RED}❌ Failed to fund legacy address${COLORS.NC}`);
    console.log(error.message);
    
    // Try one of the other legacy addresses from the fixtures
    const altLegacyAddresses = [
      'mxbBHPuZmf8Ve5pBgdbwDjLP1cR3mqZZQM',   // From fixtures
      '2N3dtsJjqbXWLEK2Np6JePpKHyy5ph6wYPy',  // Nested SegWit from fixtures  
      'n2cEk5AwwS3fBDSRUe1kLfnZsg26hmGUtZ'    // Another legacy format
    ];
    
    // Try each alternative address
    for (let i=0; i < altLegacyAddresses.length; i++) {
      if (altLegacyAddresses[i] === legacyAddress) continue; // Skip the one we already tried
      
      const altAddress = altLegacyAddresses[i];
      console.log(`\n${COLORS.YELLOW}Trying alternative address ${i+1}: ${altAddress}${COLORS.NC}`);
      
      const altFundCommand = `/bin/bash -c "source ${ROOT_DIR}/.env && oyl regtest sendFromFaucet -p oylnet -t '${altAddress}' -s 100000000"`;
      console.log(`${COLORS.CYAN}$ ${altFundCommand}${COLORS.NC}`);
      
      try {
        const altFundResult = execSync(altFundCommand, { encoding: 'utf8' });
        console.log(`${COLORS.GREEN}✅ Successfully funded alternative address!${COLORS.NC}`);
        console.log(altFundResult);
        
        // Update the env file with the working address
        envContent = fs.readFileSync(path.join(ROOT_DIR, '.env'), 'utf8');
        envContent = envContent.replace(/LEGACY_ADDRESS=.*(\r?\n|$)/g, `LEGACY_ADDRESS="${altAddress}"$1`);
        envContent = envContent.replace(/FUNDED_ADDRESS=.*(\r?\n|$)/g, `FUNDED_ADDRESS="${altAddress}"$1`);
        fs.writeFileSync(path.join(ROOT_DIR, '.env'), envContent);
        console.log(`${COLORS.GREEN}✅ .env file updated with working address${COLORS.NC}`);
        
        // Generate blocks to confirm funding
        console.log(`\n${COLORS.YELLOW}Generating blocks to confirm funding...${COLORS.NC}`);
        const altConfirmCommand = `/bin/bash -c "source ${ROOT_DIR}/.env && oyl regtest genBlocks -p oylnet -c 10"`;
        console.log(`${COLORS.CYAN}$ ${altConfirmCommand}${COLORS.NC}`);
        
        try {
          const altConfirmResult = execSync(altConfirmCommand, { encoding: 'utf8' });
          console.log(`${COLORS.GREEN}✅ Generated confirmation blocks${COLORS.NC}`);
          console.log(altConfirmResult);
        } catch (error) {
          console.log(`${COLORS.RED}❌ Failed to generate confirmation blocks${COLORS.NC}`);
          console.log(error.message);
        }
        
        console.log(`\n${COLORS.BOLD}${COLORS.GREEN}==== SUCCESS with alternative address! =====${COLORS.NC}`);
        console.log(`${COLORS.GREEN}The address ${altAddress} has been funded!${COLORS.NC}`);
        console.log(`${COLORS.YELLOW}Run this command to deploy:${COLORS.NC}`);
        console.log(`node deploy_contract.js --address ${altAddress}`);
        
        // We found a working address, no need to try more
        break;
      } catch (altError) {
        console.log(`${COLORS.RED}❌ Failed with alternative address ${altAddress}${COLORS.NC}`);
        console.log(altError.message);
      }
    }
    
    // Try with a test command to get more info
    console.log(`\n${COLORS.YELLOW}Checking for address compatibility details...${COLORS.NC}`);
    try {
      const testCommand = `/bin/bash -c "source ${ROOT_DIR}/.env && oyl regtest sendFromFaucet -h"`;
      console.log(`${COLORS.CYAN}$ ${testCommand}${COLORS.NC}`);
      const testResult = execSync(testCommand, { encoding: 'utf8' });
      console.log('sendFromFaucet help:');
      console.log(testResult);
    } catch (testError) {
      console.log(testError.message);
    }
  }
} catch (error) {
  console.log(`${COLORS.RED}❌ Script execution failed:${COLORS.NC} ${error.message}`);
  process.exit(1);
}
