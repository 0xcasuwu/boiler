
/**
 * OylNet Address Patch Test Script
 *
 * This script validates that our custom patch for address compatibility works.
 */

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

console.log('Loading patched bitcoinjs-lib for testing...');
// Force load our custom patch
require('./oyl-sdk/lib/shared/load_patch');
const bitcoin = require('./oyl-sdk/lib/shared/address_patch');

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

console.log(`${COLORS.BOLD}${COLORS.BLUE}=== OylNet Address Patch Test ===${COLORS.NC}`);

// Test addresses that previously failed
const testAddresses = [
  'bcrt1qeyyk6sl5gvr4wzm0dpmfqcjsls9xfkgvurkz7p', // Bech32 Segwit
  'mxbBHPuZmf8Ve5pBgdbwDjLP1cR3mqZZQM',           // Legacy P2PKH
  '2N3dtsJjqbXWLEK2Np6JePpKHyy5ph6wYPy',          // Nested Segwit
  'n2cEk5AwwS3fBDSRUe1kLfnZsg26hmGUtZ'            // Another legacy format
];

// Try converting each address to an output script with our patched function
console.log(`${COLORS.YELLOW}Testing address to script conversion with patch:${COLORS.NC}`);

for (const address of testAddresses) {
  console.log(`${COLORS.BLUE}Testing address: ${address}${COLORS.NC}`);
  try {
    const script = bitcoin.address.toOutputScript(address, bitcoin.networks.regtest);
    console.log(`${COLORS.GREEN}✅ Success! Script: ${script.toString('hex')}${COLORS.NC}`);
  } catch (error) {
    console.log(`${COLORS.RED}❌ Failed: ${error.message}${COLORS.NC}`);
  }
}

// Test generating a block with OylNet
console.log(`\n${COLORS.YELLOW}Testing OylNet block generation:${COLORS.NC}`);

try {
  const command = `/bin/bash -c "source ${path.resolve(__dirname)}/.env && NODE_OPTIONS=--require=./oyl-sdk/lib/shared/load_patch.js oyl regtest genBlocks -p oylnet -c 1"`;
  console.log(`${COLORS.CYAN}$ ${command}${COLORS.NC}`);
  
  const result = execSync(command, { encoding: 'utf8' });
  console.log(`${COLORS.GREEN}✅ Block generation successful!${COLORS.NC}`);
  console.log(result);
} catch (error) {
  console.log(`${COLORS.RED}❌ Block generation failed: ${error.message}${COLORS.NC}`);
}

console.log(`\n${COLORS.BOLD}${COLORS.GREEN}Test completed.${COLORS.NC}`);
console.log(`If the tests passed, you can now run the patched deployment script:`);
console.log(`node deploy_with_patch.js`);
