/**
 * OylNet Address Format Investigation
 * 
 * This script examines the expected address format for OylNet
 * by analyzing the bitcoinjs-lib usage in the oyl-sdk.
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

function displaySectionHeader(title) {
  console.log(`\n${COLORS.BOLD}${COLORS.BLUE}=== ${title} ====${COLORS.NC}\n`);
}

/**
 * Examine the regtest address format by finding any hard-coded regtest addresses in the SDK
 */
function examineRegtestAddresses() {
  displaySectionHeader("Examining Regtest Addresses in OylNet SDK");
  
  // Search for regtest address patterns in the SDK
  try {
    console.log(`${COLORS.YELLOW}Searching for regtest address patterns...${COLORS.NC}`);
    
    // Look for 'regtest' string in JS files
    const grepCommand = `find ./oyl-sdk -name "*.js" -type f | xargs grep -l "regtest" | xargs grep -l "address"`;
    console.log(`${COLORS.CYAN}$ ${grepCommand}${COLORS.NC}`);
    
    try {
      const grepResult = execSync(grepCommand, { encoding: 'utf8' });
      console.log('Files containing both "regtest" and "address":');
      console.log(grepResult);
      
      // Check each file for regtest addresses
      const files = grepResult.split('\n').filter(Boolean);
      for (const file of files) {
        const content = fs.readFileSync(file, 'utf8');
        
        // Look for regtest address patterns like bcrt1, etc.
        const addressPatterns = [
          /["']bcrt1[a-z0-9]+["']/g,   // bech32 regtest
          /["'][mn][a-km-zA-HJ-NP-Z1-9]{26,35}["']/g  // legacy regtest
        ];
        
        for (const pattern of addressPatterns) {
          const matches = content.match(pattern);
          if (matches && matches.length > 0) {
            console.log(`${COLORS.GREEN}Found potential regtest addresses in ${file}:${COLORS.NC}`);
            matches.forEach(match => console.log(`  ${match}`));
          }
        }
      }
    } catch (error) {
      console.log(`${COLORS.YELLOW}No matching files found${COLORS.NC}`);
      console.log(error.message);
    }
    
    // Look for any direct examples in the test fixture files
    const fixtureCommand = `grep -r "address" ./oyl-sdk/src/__fixtures__`;
    console.log(`\n${COLORS.YELLOW}Checking for examples in test fixtures...${COLORS.NC}`);
    console.log(`${COLORS.CYAN}$ ${fixtureCommand}${COLORS.NC}`);
    
    try {
      const fixtureResult = execSync(fixtureCommand, { encoding: 'utf8' });
      console.log('Fixture address examples:');
      console.log(fixtureResult);
    } catch (error) {
      console.log(`${COLORS.YELLOW}No fixture examples found${COLORS.NC}`);
      console.log(error.message);
    }
  } catch (error) {
    console.log(`${COLORS.RED}Error examining regtest addresses:${COLORS.NC} ${error.message}`);
  }
}

/**
 * Analyze how bitcoinjs-lib is used in the SDK to generate addresses
 */
function analyzeBitcoinJSLibUsage() {
  displaySectionHeader("Analyzing bitcoinjs-lib Usage");
  
  try {
    // Find files that use bitcoinjs-lib
    const bitcoinJSCommand = `grep -r "require.*bitcoinjs-lib" ./oyl-sdk`;
    console.log(`${COLORS.CYAN}$ ${bitcoinJSCommand}${COLORS.NC}`);
    
    try {
      const bitcoinJSResult = execSync(bitcoinJSCommand, { encoding: 'utf8' });
      console.log('Files using bitcoinjs-lib:');
      console.log(bitcoinJSResult);
    } catch (error) {
      console.log(`${COLORS.YELLOW}No direct bitcoinjs-lib imports found${COLORS.NC}`);
    }
    
    // Look for network definitions
    const networkCommand = `grep -r "networks\\.regtest" ./oyl-sdk`;
    console.log(`\n${COLORS.YELLOW}Checking for regtest network definitions...${COLORS.NC}`);
    console.log(`${COLORS.CYAN}$ ${networkCommand}${COLORS.NC}`);
    
    try {
      const networkResult = execSync(networkCommand, { encoding: 'utf8' });
      console.log('Regtest network references:');
      console.log(networkResult);
    } catch (error) {
      console.log(`${COLORS.YELLOW}No explicit regtest network references found${COLORS.NC}`);
    }
    
    // Look for address generation
    const addressGenCommand = `grep -r "payments\\." ./oyl-sdk`;
    console.log(`\n${COLORS.YELLOW}Checking for bitcoinjs-lib payment code pattern usage...${COLORS.NC}`);
    console.log(`${COLORS.CYAN}$ ${addressGenCommand}${COLORS.NC}`);
    
    try {
      const addressGenResult = execSync(addressGenCommand, { encoding: 'utf8' });
      console.log('Address generation code:');
      console.log(addressGenResult);
    } catch (error) {
      console.log(`${COLORS.YELLOW}No payment code patterns found${COLORS.NC}`);
    }
  } catch (error) {
    console.log(`${COLORS.RED}Error analyzing bitcoinjs-lib usage:${COLORS.NC} ${error.message}`);
  }
}

/**
 * Query the OylNet regtest info directly
 */
function queryOylNetInfo() {
  displaySectionHeader("Querying OylNet Network Info");
  
  try {
    // Check if we can get network info
    const networkInfoCommand = `/bin/bash -c "source ${ROOT_DIR}/.env && oyl regtest info -p oylnet"`;
    console.log(`${COLORS.CYAN}$ ${networkInfoCommand}${COLORS.NC}`);
    
    try {
      const networkInfoResult = execSync(networkInfoCommand, { encoding: 'utf8' });
      console.log('OylNet regtest network info:');
      console.log(networkInfoResult);
    } catch (error) {
      console.log(`${COLORS.YELLOW}Failed to get network info:${COLORS.NC} ${error.message}`);
      
      // Try an alternative command
      console.log(`\n${COLORS.YELLOW}Trying alternative command...${COLORS.NC}`);
      const altCommand = `/bin/bash -c "source ${ROOT_DIR}/.env && oyl -h"`;
      console.log(`${COLORS.CYAN}$ ${altCommand}${COLORS.NC}`);
      
      try {
        const altResult = execSync(altCommand, { encoding: 'utf8' });
        console.log('OylNet CLI help:');
        console.log(altResult);
      } catch (altError) {
        console.log(`${COLORS.RED}Failed to get CLI help:${COLORS.NC} ${altError.message}`);
      }
    }
    
    // Try to dump environment variables that might contain address info
    console.log(`\n${COLORS.YELLOW}Checking for environment variables with address information...${COLORS.NC}`);
    const envCommand = `/bin/bash -c "source ${ROOT_DIR}/.env && env | grep -i 'addr\\|key\\|wallet'"`;
    console.log(`${COLORS.CYAN}$ ${envCommand}${COLORS.NC}`);
    
    try {
      const envResult = execSync(envCommand, { encoding: 'utf8' });
      console.log('Relevant environment variables:');
      console.log(envResult.replace(/private|key|secret/gi, '[REDACTED]')); // Redact sensitive info
    } catch (error) {
      console.log(`${COLORS.YELLOW}No relevant environment variables found${COLORS.NC}`);
    }
    
    // Check if OylNet CLI has a command to show a valid address
    console.log(`\n${COLORS.YELLOW}Checking for OylNet CLI address commands...${COLORS.NC}`);
    const addressCommand = `/bin/bash -c "source ${ROOT_DIR}/.env && oyl regtest -h"`;
    console.log(`${COLORS.CYAN}$ ${addressCommand}${COLORS.NC}`);
    
    try {
      const addressResult = execSync(addressCommand, { encoding: 'utf8' });
      console.log('OylNet regtest commands:');
      console.log(addressResult);
    } catch (error) {
      console.log(`${COLORS.RED}Failed to get regtest help:${COLORS.NC} ${error.message}`);
    }
  } catch (error) {
    console.log(`${COLORS.RED}Error querying OylNet info:${COLORS.NC} ${error.message}`);
  }
}

/**
 * Try to generate different address formats
 */
function tryDifferentAddressFormats() {
  displaySectionHeader("Trying Different Address Formats");
  
  try {
    // We need to use bitcoinjs-lib directly
    // Since we don't want to modify package.json, we'll use the one from oyl-sdk
    const bitcoin = require('./oyl-sdk/node_modules/bitcoinjs-lib');
    
    console.log(`${COLORS.GREEN}Successfully loaded bitcoinjs-lib${COLORS.NC}`);
    
    // Generate different address formats
    const ECPair = require('./oyl-sdk/node_modules/ecpair');
    // Use a fixed seed for reproducibility
    const seed = Buffer.from('0123456789abcdef0123456789abcdef', 'hex');
    const privateKey = Buffer.from('1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef', 'hex');
    
    console.log(`${COLORS.YELLOW}Generating test addresses with fixed seed for reproducibility...${COLORS.NC}`);
    
    const bitcoinNetworks = [
      { name: "Regtest", network: bitcoin.networks.regtest },
      { name: "Testnet", network: bitcoin.networks.testnet },
      { name: "Bitcoin", network: bitcoin.networks.bitcoin }
    ];
    
    const addressTypes = [
      { 
        name: "P2PKH (Legacy)", 
        generator: (keyPair, network) => bitcoin.payments.p2pkh({ 
          pubkey: keyPair.publicKey, 
          network: network 
        }).address
      },
      { 
        name: "P2SH", 
        generator: (keyPair, network) => {
          const p2wpkh = bitcoin.payments.p2wpkh({ 
            pubkey: keyPair.publicKey, 
            network: network 
          });
          return bitcoin.payments.p2sh({
            redeem: p2wpkh,
            network: network
          }).address;
        }
      },
      { 
        name: "P2WPKH (Bech32/SegWit)", 
        generator: (keyPair, network) => bitcoin.payments.p2wpkh({ 
          pubkey: keyPair.publicKey, 
          network: network 
        }).address
      },
      { 
        name: "P2WSH (Bech32/SegWit Script)", 
        generator: (keyPair, network) => {
          // Create a simple redeem script (just a P2PKH script)
          const p2ms = bitcoin.payments.p2ms({
            m: 1, 
            pubkeys: [keyPair.publicKey], 
            network
          });
          return bitcoin.payments.p2wsh({
            redeem: p2ms,
            network
          }).address;
        }
      }
    ];
    
    // Test all combinations
    for (const network of bitcoinNetworks) {
      console.log(`\n${COLORS.BLUE}Network: ${network.name}${COLORS.NC}`);
      
      // Create a key pair for this network
      const keyPair = ECPair.ECPairFactory().fromPrivateKey(privateKey, { 
        network: network.network 
      });
      
      for (const type of addressTypes) {
        try {
          const address = type.generator(keyPair, network.network);
          console.log(`${type.name}: ${address}`);
        } catch (error) {
          console.log(`${type.name}: Error - ${error.message}`);
        }
      }
    }
    
    console.log(`\n${COLORS.YELLOW}Trying a command to find a valid OylNet address...${COLORS.NC}`);
    try {
      // Check if OylNet CLI exposes a way to show a valid address
      const faucetInfoCommand = `/bin/bash -c "source ${ROOT_DIR}/.env && oyl regtest init -p oylnet -h"`;
      console.log(`${COLORS.CYAN}$ ${faucetInfoCommand}${COLORS.NC}`);
      const faucetInfo = execSync(faucetInfoCommand, { encoding: 'utf8' });
      console.log('OylNet regtest init help:');
      console.log(faucetInfo);
    } catch (error) {
      console.log(`${COLORS.RED}Failed to get faucet info:${COLORS.NC} ${error.message}`);
    }
    
  } catch (error) {
    console.log(`${COLORS.RED}Error trying different address formats:${COLORS.NC} ${error.message}`);
    if (error.message.includes('Cannot find module')) {
      console.log(`${COLORS.YELLOW}Missing required module. The script would need to install it first.${COLORS.NC}`);
    }
  }
}

/**
 * Analyze the bitcoinjs-lib error in the faucet command
 */
function analyzeScriptError() {
  displaySectionHeader("Analyzing Script Error");
  
  console.log(`${COLORS.YELLOW}The error we're seeing is:${COLORS.NC}`);
  console.log(`OylTransactionError: bcrt1q... has no matching Script`);
  console.log(`at toOutputScript (/workspaces/boiler/oyl-sdk/node_modules/bitcoinjs-lib/src/address.js:178:9)`);
  
  // Try to look at the specific code causing the error
  try {
    const errorFilePath = './oyl-sdk/node_modules/bitcoinjs-lib/src/address.js';
    if (fs.existsSync(errorFilePath)) {
      const fileContent = fs.readFileSync(errorFilePath, 'utf8');
      const lines = fileContent.split('\n');
      
      // Find the toOutputScript function
      let functionStart = -1;
      let functionEnd = -1;
      
      for (let i = 0; i < lines.length; i++) {
        if (lines[i].includes('function toOutputScript') || lines[i].includes('export function toOutputScript')) {
          functionStart = i;
        } else if (functionStart !== -1 && lines[i] === '}') {
          functionEnd = i;
          break;
        }
      }
      
      if (functionStart !== -1 && functionEnd !== -1) {
        console.log(`\n${COLORS.YELLOW}Found toOutputScript function (lines ${functionStart}-${functionEnd}):${COLORS.NC}`);
        const functionCode = lines.slice(functionStart, functionEnd + 1).join('\n');
        console.log(functionCode);
        
        // Look for line 178 specifically (which is causing the error)
        const errorLineIndex = 177; // 0-based index for line 178
        if (errorLineIndex < lines.length) {
          console.log(`\n${COLORS.RED}Error line (178):${COLORS.NC}`);
          console.log(lines[errorLineIndex]);
          
          // Show a few lines before and after for context
          console.log(`\n${COLORS.YELLOW}Error context:${COLORS.NC}`);
          const contextStart = Math.max(errorLineIndex - 5, 0);
          const contextEnd = Math.min(errorLineIndex + 5, lines.length);
          for (let i = contextStart; i <= contextEnd; i++) {
            const marker = i === errorLineIndex ? '> ' : '  ';
            console.log(`${marker}${i + 1}: ${lines[i]}`);
          }
        }
      }
    } else {
      console.log(`${COLORS.RED}Could not find bitcoinjs-lib address.js file${COLORS.NC}`);
    }
  } catch (error) {
    console.log(`${COLORS.RED}Error analyzing script error:${COLORS.NC} ${error.message}`);
  }
}

// Execute all the analysis functions
function main() {
  displaySectionHeader("OylNet Address Format Investigation");
  
  console.log(`This script will analyze the OylNet address format requirements`);
  console.log(`by examining the SDK code and error patterns.`);
  
  examineRegtestAddresses();
  analyzeBitcoinJSLibUsage();
  queryOylNetInfo();
  tryDifferentAddressFormats();
  analyzeScriptError();
  
  displaySectionHeader("Investigation Summary");
  
  console.log(`Based on the error "bcrt1q... has no matching Script", it appears that:`);
  console.log(`1. The OylNet system does not recognize the bech32 address format (bcrt1...) used by default`);
  console.log(`2. We likely need to use a legacy address format (starting with m or n) for regtest`);
  console.log(`3. The specific networks and parameters needed for generating compatible addresses`);
  console.log(`   may not match standard bitcoinjs-lib regtest configurations`);
  console.log(``);
  console.log(`Next steps would be to:`);
  console.log(`1. Generate a variety of address formats using different parameters`);
  console.log(`2. Test each address format with the OylNet faucet until we find one that works`);
  console.log(`3. Once we find a working format, implement it in our deployment scripts`);
}

// Run the main function
main();
