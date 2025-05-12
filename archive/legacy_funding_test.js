/**
 * Legacy Address Funding Test
 * 
 * This script generates a legacy P2PKH address (starting with 'm' for regtest)
 * and attempts to fund it, which may be compatible with OylNet.
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
  console.log(`${COLORS.BOLD}${COLORS.BLUE}Legacy P2PKH Address Funding Test${COLORS.NC}\n`);
  
  // Load bitcoinjs-lib from oyl-sdk
  console.log(`${COLORS.YELLOW}Loading bitcoinjs-lib...${COLORS.NC}`);
  const bitcoin = require('./oyl-sdk/node_modules/bitcoinjs-lib');
  console.log(`${COLORS.GREEN}Successfully loaded bitcoinjs-lib${COLORS.NC}`);
  
  // Create ECPair factory
  const ECPair = require('./oyl-sdk/node_modules/ecpair');
  const ecc = require('./oyl-sdk/node_modules/tiny-secp256k1');
  const ECPairFactory = ECPair.ECPairFactory(ecc);
  
  // Generate key pair for regtest network
  console.log(`\n${COLORS.YELLOW}Generating regtest key pair...${COLORS.NC}`);
  const keyPair = ECPairFactory.makeRandom({ network: bitcoin.networks.regtest });
  console.log(`${COLORS.GREEN}Key pair generated${COLORS.NC}`);
  
  // Generate P2PKH (legacy) address
  console.log(`\n${COLORS.YELLOW}Generating P2PKH (legacy) address...${COLORS.NC}`);
  const { address: legacyAddress } = bitcoin.payments.p2pkh({ 
    pubkey: keyPair.publicKey,
    network: bitcoin.networks.regtest
  });
  
  console.log(`${COLORS.GREEN}Generated legacy address: ${COLORS.BOLD}${legacyAddress}${COLORS.NC}`);
  console.log(`${COLORS.GREEN}WIF private key: ${COLORS.BOLD}${keyPair.toWIF()}${COLORS.NC}`);
  
  // Save wallet info
  const walletInfo = {
    address: legacyAddress,
    privateKey: keyPair.toWIF(),
    networkType: 'regtest',
    format: 'P2PKH (Legacy)',
    timestamp: new Date().toISOString()
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
