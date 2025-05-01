#!/usr/bin/env node
/**
 * Sanity Check Tests for SLOP Contracts
 * 
 * This script makes basic calls to the deployed SLOP contracts
 * to verify they are working correctly.
 */

const http = require('http');

// Configuration
const RPC_URL = 'http://localhost:18889';
const CONTRACTS = {
  BOND_CURVE: '00000000000000000000000000000000000000000000000000000000000003e9',
  ORBITAL_BOND: '00000000000000000000000000000000000000000000000000000000000003ea', 
  LAUNCHPAD_FACTORY: '00000000000000000000000000000000000000000000000000000000000003eb'
};

// Color codes for console output
const colors = {
  blue: '\x1b[34m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  red: '\x1b[31m',
  cyan: '\x1b[36m',
  reset: '\x1b[0m'
};

// Make an RPC request to the server
function makeRpcRequest(method, params) {
  return new Promise((resolve, reject) => {
    const requestBody = JSON.stringify({
      jsonrpc: '2.0',
      method: method,
      params: params,
      id: 1
    });

    const options = {
      hostname: 'localhost',
      port: 18889,
      path: '/',
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Content-Length': Buffer.byteLength(requestBody)
      }
    };

    const req = http.request(options, (res) => {
      let data = '';
      
      res.on('data', (chunk) => {
        data += chunk;
      });
      
      res.on('end', () => {
        try {
          const response = JSON.parse(data);
          resolve(response);
        } catch (error) {
          reject(error);
        }
      });
    });
    
    req.on('error', reject);
    req.write(requestBody);
    req.end();
  });
}

// Call a contract function
async function callContract(contractId, opcode, args) {
  console.log(`${colors.blue}Calling ${colors.cyan}${opcode}${colors.blue} on contract ${colors.yellow}${contractId}${colors.reset}`);
  
  try {
    // Make the contract call
    const response = await makeRpcRequest('alkane_callContract', [contractId, opcode, args]);
    
    if (response.error) {
      console.log(`${colors.red}Error calling contract: ${JSON.stringify(response.error)}${colors.reset}`);
      return null;
    }
    
    const txid = response.result;
    console.log(`${colors.green}Call successful with txid: ${colors.yellow}${txid}${colors.reset}`);
    
    // Wait for transaction confirmation
    console.log(`${colors.blue}Waiting for transaction confirmation...${colors.reset}`);
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    // Get transaction receipt
    const receiptResponse = await makeRpcRequest('alkane_getTransactionReceipt', [txid]);
    
    if (receiptResponse.error) {
      console.log(`${colors.red}Error getting receipt: ${JSON.stringify(receiptResponse.error)}${colors.reset}`);
      return null;
    }
    
    console.log(`${colors.green}Result:${colors.reset}`);
    console.log(JSON.stringify(receiptResponse.result, null, 2));
    
    return receiptResponse.result;
  } catch (error) {
    console.log(`${colors.red}Error: ${error}${colors.reset}`);
    return null;
  }
}

// Test Bond Curve contract
async function testBondCurve() {
  console.log(`\n${colors.blue}========== Testing Bond Curve Contract ==========${colors.reset}`);
  
  // Call getPrice function to get current price for 10 tokens
  const amount = 10;
  await callContract(CONTRACTS.BOND_CURVE, 'getPrice', `${amount}`);
  
  // Call getVersion function to get contract version
  await callContract(CONTRACTS.BOND_CURVE, 'getVersion', '');
}

// Test Orbital Bond Collection contract
async function testOrbitalBondCollection() {
  console.log(`\n${colors.blue}========== Testing Orbital Bond Collection Contract ==========${colors.reset}`);
  
  // Call getBondCount function to get number of bonds
  await callContract(CONTRACTS.ORBITAL_BOND, 'getBondCount', '');
  
  // Call getTokenInfo function to get token information
  await callContract(CONTRACTS.ORBITAL_BOND, 'getTokenInfo', '');
}

// Test Launchpad Factory contract
async function testLaunchpadFactory() {
  console.log(`\n${colors.blue}========== Testing Launchpad Factory Contract ==========${colors.reset}`);
  
  // Call getFactoryInfo function to get factory information
  await callContract(CONTRACTS.LAUNCHPAD_FACTORY, 'getFactoryInfo', '');
  
  // Call getVersion function to get contract version
  await callContract(CONTRACTS.LAUNCHPAD_FACTORY, 'getVersion', '');
}

// Run all tests
async function runTests() {
  console.log(`${colors.blue}==========================================${colors.reset}`);
  console.log(`${colors.blue} SLOP Contract Sanity Check Tests${colors.reset}`);
  console.log(`${colors.blue}==========================================${colors.reset}`);
  
  try {
    // Test each contract
    await testBondCurve();
    await testOrbitalBondCollection();
    await testLaunchpadFactory();
    
    console.log(`\n${colors.green}All tests completed!${colors.reset}`);
  } catch (error) {
    console.error(`${colors.red}Error running tests: ${error}${colors.reset}`);
    process.exit(1);
  }
}

// Run the tests
runTests();
