/**
 * Create a compatible OylNet wallet
 * 
 * This script creates a wallet compatible with OylNet and generates an address
 * that can be used for funding operations.
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
 * Create a compatible OylNet wallet using the oyl-sdk directly
 */
function createOylNetWallet() {
  console.log(`${COLORS.BOLD}${COLORS.BLUE}Creating Compatible OylNet Wallet${COLORS.NC}\n`);
  
  try {
    // Step 1: Generate a new wallet using bitcoinjs-lib
    console.log(`${COLORS.YELLOW}1. Creating new Bitcoin wallet${COLORS.NC}`);
    
    // Use Node.js crypto module to generate a random bytes for the private key
    const crypto = require('crypto');
    const privateKey = crypto.randomBytes(32);
    const privateKeyHex = privateKey.toString('hex');
    
    // Load bitcoinjs-lib from oyl-sdk node_modules
    const bitcoin = require('./oyl-sdk/node_modules/bitcoinjs-lib');
    
    // Create REGTEST network parameters
    const regtest = bitcoin.networks.regtest;
    
    // Create key pair from private key
    const keyPair = bitcoin.ECPair.fromPrivateKey(privateKey, { network: regtest });
    
    // Generate P2WPKH address
    const { address } = bitcoin.payments.p2wpkh({ 
      pubkey: keyPair.publicKey, 
      network: regtest 
    });
    
    console.log(`${COLORS.GREEN}✅ Wallet created successfully${COLORS.NC}`);
    console.log(`${COLORS.BLUE}Generated address: ${COLORS.BOLD}${address}${COLORS.NC}`);
    console.log(`${COLORS.BLUE}Private key: ${COLORS.BOLD}${privateKeyHex}${COLORS.NC}`);
    
    // Create wallet info object
    const walletInfo = {
      address,
      privateKey: privateKeyHex,
      networkType: 'regtest',
      timestamp: new Date().toISOString()
    };
    
    // Save wallet info to file
    fs.writeFileSync(path.join(ROOT_DIR, 'wallet_info.json'), JSON.stringify(walletInfo, null, 2));
    console.log(`${COLORS.GREEN}✅ Wallet info saved to wallet_info.json${COLORS.NC}`);
    
    // Step 2: Test if the generated address can be used with OylNet
    console.log(`\n${COLORS.YELLOW}2. Validating address format compatibility with OylNet${COLORS.NC}`);
    
    // Attempt to initialize the regtest with this address
    // This command should validate if the address is in the correct format
    const command = `/bin/bash -c "source ${ROOT_DIR}/.env && oyl regtest init -p oylnet --to ${address} --blocks 1"`;
    console.log(`${COLORS.CYAN}$ ${command}${COLORS.NC}`);
    
    try {
      const result = execSync(command, { encoding: 'utf8' });
      console.log(`${COLORS.GREEN}✅ Address format validation successful${COLORS.NC}`);
      console.log('Output:');
      console.log(result);
    } catch (error) {
      if (error.message.includes('has no matching Script')) {
        console.log(`${COLORS.RED}❌ Address format incompatible with OylNet${COLORS.NC}`);
        throw new Error('Generated address is not compatible with OylNet');
      } else {
        // If we get a different error, the format might be correct but another issue occurred
        console.log(`${COLORS.YELLOW}⚠️ Command failed but address format may be valid${COLORS.NC}`);
        console.log('Error:');
        console.log(error.message);
      }
    }
    
    // Step 3: Update .env file with address
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
    
    // Add or update PRIVATE_KEY in .env
    if (envContent.includes('PRIVATE_KEY=')) {
      envContent = envContent.replace(/PRIVATE_KEY=.*(\r?\n|$)/g, `PRIVATE_KEY="${privateKeyHex}"$1`);
    } else {
      envContent += `\nPRIVATE_KEY="${privateKeyHex}"\n`;
    }
    
    fs.writeFileSync(path.join(ROOT_DIR, '.env'), envContent);
    console.log(`${COLORS.GREEN}✅ .env file updated with FUNDED_ADDRESS and PRIVATE_KEY${COLORS.NC}`);
    
    return { address, privateKey: privateKeyHex };
  } catch (error) {
    console.log(`${COLORS.RED}❌ Failed to create wallet:${COLORS.NC} ${error.message}`);
    return null;
  }
}

// Execute the function
const result = createOylNetWallet();

if (result) {
  console.log(`\n${COLORS.BLUE}Next steps:${COLORS.NC}`);
  console.log(`1. Fund this address (${result.address}) with:\n   ./bin/net/structured_network.sh --fund`);
  console.log(`2. Run the test script:\n   node test_network_operations.js`);
}
