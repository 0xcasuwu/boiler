/**
 * OylNet Alkane ID Verification Tests
 * 
 * This script tests the alkane ID verification process in the YieldVault contract
 * on OylNet. It verifies that the contract correctly checks for the presence of
 * tokens in transactions.
 */

// Import the OYL SDK with the patch applied
require('/workspaces/boiler/oyl-sdk/lib/shared/load_patch.js');
const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');
const assert = require('assert');

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

// Path to the contract details file
const CONTRACT_DETAILS_FILE = path.resolve(__dirname, 'contract_details.json');

// Load contract details
const contractDetails = JSON.parse(fs.readFileSync(CONTRACT_DETAILS_FILE, 'utf8'));
const CONTRACT_ID = contractDetails.contractId;

// Asset ID (numeric ID for compatibility with OYL SDK)
// Format should be "block:tx" where both are numeric values
const ASSET_ID = '2:1'; // Using numeric ID for the asset (block 2, tx 1)

// ERC-4626 opcodes
const OPCODES = {
  // Initialization
  INITIALIZE: 0,
  
  // Asset Management
  DEPOSIT: 10,
  MINT: 11,
  WITHDRAW: 12,
  REDEEM: 13,
  
  // Metadata View Functions
  GET_NAME: 100,
  GET_SYMBOL: 101,
  GET_DECIMALS: 102,
  GET_ASSET: 103,
  
  // Accounting View Functions
  GET_TOTAL_ASSETS: 200,
  CONVERT_TO_SHARES: 201,
  CONVERT_TO_ASSETS: 202,
  
  // Limit View Functions
  GET_MAX_DEPOSIT: 300,
  GET_MAX_MINT: 301,
  GET_MAX_WITHDRAW: 302,
  GET_MAX_REDEEM: 303,
  
  // Preview View Functions
  PREVIEW_DEPOSIT: 400,
  PREVIEW_MINT: 401,
  PREVIEW_WITHDRAW: 402,
  PREVIEW_REDEEM: 403,
  
  // Balance Management
  GET_TOTAL_SUPPLY: 601,
};

// User accounts for testing (using numeric IDs instead of strings)
const USERS = {
  ALICE: 1,
  BOB: 2
};

// Test results
const testResults = {
  passed: 0,
  failed: 0,
  results: []
};

/**
 * Execute a command and return the output
 * @param {string} command - The command to execute
 * @returns {string} - The command output
 */
function executeCommand(command) {
  console.log(`${COLORS.CYAN}$ ${command}${COLORS.NC}`);
  try {
    const output = execSync(command, { encoding: 'utf8' });
    return output.trim();
  } catch (error) {
    console.error(`${COLORS.RED}Command failed: ${error.message}${COLORS.NC}`);
    if (error.stdout) console.error(`${COLORS.RED}stdout: ${error.stdout}${COLORS.NC}`);
    if (error.stderr) console.error(`${COLORS.RED}stderr: ${error.stderr}${COLORS.NC}`);
    throw error;
  }
}

/**
 * Generate blocks on OylNet
 * @param {number} count - Number of blocks to generate
 */
function generateBlocks(count) {
  console.log(`${COLORS.YELLOW}Generating ${count} blocks...${COLORS.NC}`);
  executeCommand(`NODE_OPTIONS=--require=/workspaces/boiler/oyl-sdk/lib/shared/load_patch.js oyl regtest genBlocks -p oylnet -c ${count}`);
}

/**
 * Execute a contract call with edicts (alkane transfers)
 * @param {number} opcode - The opcode to call
 * @param {Array} params - Parameters for the call
 * @param {Array} edicts - Edicts (alkane transfers) to include in the transaction
 * @returns {Object} - Result of the call
 */
function executeContractCall(opcode, params = [], edicts = []) {
  // Convert opcode and parameters to a comma-separated string
  const calldata = [opcode, ...params].join(',');
  
  console.log(`${COLORS.CYAN}Executing opcode ${opcode}${COLORS.NC}`);
  console.log(`${COLORS.CYAN}Parameters: ${params.join(', ')}${COLORS.NC}`);
  
  // Build the edicts string if provided
  let edictsString = '';
  if (edicts.length > 0) {
    edictsString = ' --edicts "' + edicts.map(e => {
      // Include output parameter if it exists, otherwise just use id:value
      return e.output !== undefined ? `${e.id}:${e.value}:${e.output}` : `${e.id}:${e.value}`;
    }).join(',') + '"';
    console.log(`${COLORS.CYAN}Edicts: ${edictsString}${COLORS.NC}`);
  }
  
  // Execute the command
  const command = `NODE_OPTIONS=--require=/workspaces/boiler/oyl-sdk/lib/shared/load_patch.js oyl alkane execute -data "${calldata}"${edictsString} --provider oylnet`;
  
  try {
    const result = executeCommand(command);
    console.log(`${COLORS.GREEN}Operation successful${COLORS.NC}`);
    return { success: true, result };
  } catch (error) {
    console.log(`${COLORS.RED}Operation failed${COLORS.NC}`);
    return { success: false, error: error.message };
  }
}

/**
 * Get the total assets in the vault
 * @returns {number} - Total assets
 */
function getTotalAssets() {
  const result = executeContractCall(OPCODES.GET_TOTAL_ASSETS);
  if (result.success) {
    // Extract the total assets from the result
    // In a real implementation, this would parse the result properly
    return 0; // Placeholder
  }
  return 0;
}

/**
 * Get the total supply of shares
 * @returns {number} - Total supply
 */
function getTotalSupply() {
  const result = executeContractCall(OPCODES.GET_TOTAL_SUPPLY);
  if (result.success) {
    // Extract the total supply from the result
    // In a real implementation, this would parse the result properly
    return 0; // Placeholder
  }
  return 0;
}

/**
 * Run a test case and record the result
 * @param {string} name - Test case name
 * @param {Function} testFn - Test function
 */
function runTest(name, testFn) {
  console.log(`\n${COLORS.BOLD}${COLORS.BLUE}=== Running Test: ${name} ===${COLORS.NC}\n`);
  
  const startTime = Date.now();
  const startAssets = getTotalAssets();
  const startSupply = getTotalSupply();
  
  try {
    const result = testFn();
    const endTime = Date.now();
    const endAssets = getTotalAssets();
    const endSupply = getTotalSupply();
    
    console.log(`${COLORS.GREEN}✓ Test passed: ${name}${COLORS.NC}`);
    testResults.passed++;
    testResults.results.push({
      name,
      passed: true,
      duration: endTime - startTime,
      startState: { assets: startAssets, supply: startSupply },
      endState: { assets: endAssets, supply: endSupply },
      result
    });
  } catch (error) {
    console.error(`${COLORS.RED}✗ Test failed: ${name}${COLORS.NC}`);
    console.error(`${COLORS.RED}Error: ${error.message}${COLORS.NC}`);
    testResults.failed++;
    testResults.results.push({
      name,
      passed: false,
      error: error.message
    });
  }
}

/**
 * Generate a test report
 */
function generateTestReport() {
  const reportPath = path.resolve(__dirname, 'oylnet_test_report.md');
  
  let report = `# OylNet Alkane ID Verification Test Report\n\n`;
  report += `## Summary\n\n`;
  report += `- **Total Tests**: ${testResults.passed + testResults.failed}\n`;
  report += `- **Passed**: ${testResults.passed}\n`;
  report += `- **Failed**: ${testResults.failed}\n`;
  report += `- **Contract ID**: ${CONTRACT_ID}\n`;
  report += `- **Date**: ${new Date().toISOString()}\n\n`;
  
  report += `## Test Results\n\n`;
  
  testResults.results.forEach((result, index) => {
    report += `### ${index + 1}. ${result.name}\n\n`;
    report += `- **Status**: ${result.passed ? '✅ Passed' : '❌ Failed'}\n`;
    
    if (result.passed) {
      report += `- **Duration**: ${result.duration}ms\n`;
      report += `- **Start State**: Total Assets: ${result.startState.assets}, Total Supply: ${result.startState.supply}\n`;
      report += `- **End State**: Total Assets: ${result.endState.assets}, Total Supply: ${result.endState.supply}\n`;
      if (result.result) {
        report += `- **Result**: ${JSON.stringify(result.result, null, 2)}\n`;
      }
    } else {
      report += `- **Error**: ${result.error}\n`;
    }
    
    report += `\n`;
  });
  
  fs.writeFileSync(reportPath, report);
  console.log(`${COLORS.GREEN}Test report generated: ${reportPath}${COLORS.NC}`);
}

/**
 * Test Case 1: Deposit with correct alkane ID
 */
function testDepositWithCorrectAlkaneId() {
  console.log(`${COLORS.YELLOW}Testing deposit with correct alkane ID${COLORS.NC}`);
  
  // Create a transaction with the correct asset ID
  const txId = Math.floor(Math.random() * 1000000); // Random transaction ID to avoid conflicts
  
  // Preview deposit
  const previewResult = executeContractCall(OPCODES.PREVIEW_DEPOSIT, [1000]);
  
  // Execute deposit with correct asset ID
  const depositResult = executeContractCall(
    OPCODES.DEPOSIT, 
    [txId, USERS.ALICE, 1000], 
    [{ id: ASSET_ID, value: 1000, output: 0 }]
  );
  
  // Generate blocks to confirm transaction
  generateBlocks(2);
  
  // Verify the result
  if (!depositResult.success) {
    throw new Error(`Deposit failed: ${depositResult.error}`);
  }
  
  return { previewResult, depositResult };
}

/**
 * Test Case 2: Deposit with incorrect alkane ID
 */
function testDepositWithIncorrectAlkaneId() {
  console.log(`${COLORS.YELLOW}Testing deposit with incorrect alkane ID${COLORS.NC}`);
  
  // Create a transaction with an incorrect asset ID
  const txId = Math.floor(Math.random() * 1000000); // Random transaction ID to avoid conflicts
  
    // Execute deposit with incorrect asset ID
  const depositResult = executeContractCall(
    OPCODES.DEPOSIT, 
    [txId, USERS.ALICE, 1000], 
    [{ id: "3:1", value: 1000, output: 0 }] // Different numeric ID than the expected one
  );
  
  // Generate blocks to confirm transaction
  generateBlocks(2);
  
  // Verify the result
  if (depositResult.success) {
    throw new Error('Deposit with incorrect alkane ID should fail');
  }
  
  return { depositResult };
}

/**
 * Test Case 3: Deposit with multiple alkane IDs
 */
function testDepositWithMultipleAlkaneIds() {
  console.log(`${COLORS.YELLOW}Testing deposit with multiple alkane IDs${COLORS.NC}`);
  
  // Create a transaction with multiple asset IDs including the correct one
  const txId = Math.floor(Math.random() * 1000000); // Random transaction ID to avoid conflicts
  
  // Execute deposit with multiple asset IDs
  const depositResult = executeContractCall(
    OPCODES.DEPOSIT, 
    [txId, USERS.ALICE, 1000], 
    [
      { id: "3:1", value: 500, output: 0 },
      { id: ASSET_ID, value: 1000, output: 0 },
      { id: "4:1", value: 750, output: 0 }
    ]
  );
  
  // Generate blocks to confirm transaction
  generateBlocks(2);
  
  // Verify the result
  if (!depositResult.success) {
    throw new Error(`Deposit failed: ${depositResult.error}`);
  }
  
  return { depositResult };
}

/**
 * Test Case 4: Deposit with insufficient assets
 */
function testDepositWithInsufficientAssets() {
  console.log(`${COLORS.YELLOW}Testing deposit with insufficient assets${COLORS.NC}`);
  
  // Create a transaction with insufficient assets
  const txId = Math.floor(Math.random() * 1000000); // Random transaction ID to avoid conflicts
  
  // Execute deposit with insufficient assets
  const depositResult = executeContractCall(
    OPCODES.DEPOSIT, 
    [txId, USERS.ALICE, 1000], 
    [{ id: ASSET_ID, value: 500, output: 0 }] // Only 500 assets for a 1000 deposit
  );
  
  // Generate blocks to confirm transaction
  generateBlocks(2);
  
  // Verify the result
  if (depositResult.success) {
    throw new Error('Deposit with insufficient assets should fail');
  }
  
  return { depositResult };
}

/**
 * Test Case 5: Redeem with correct alkane ID
 */
function testRedeemWithCorrectAlkaneId() {
  console.log(`${COLORS.YELLOW}Testing redeem with correct alkane ID${COLORS.NC}`);
  
  // Create a transaction with the correct share token ID
  const txId = Math.floor(Math.random() * 1000000); // Random transaction ID to avoid conflicts
  
  // Preview redeem
  const previewResult = executeContractCall(OPCODES.PREVIEW_REDEEM, [500]);
  
  // Execute redeem with correct share token ID
  const redeemResult = executeContractCall(
    OPCODES.REDEEM, 
    [txId, USERS.ALICE, USERS.ALICE, 500], 
    [{ id: CONTRACT_ID, value: 500, output: 0 }]
  );
  
  // Generate blocks to confirm transaction
  generateBlocks(2);
  
  // Verify the result
  if (!redeemResult.success) {
    throw new Error(`Redeem failed: ${redeemResult.error}`);
  }
  
  return { previewResult, redeemResult };
}

/**
 * Test Case 6: Redeem with incorrect alkane ID
 */
function testRedeemWithIncorrectAlkaneId() {
  console.log(`${COLORS.YELLOW}Testing redeem with incorrect alkane ID${COLORS.NC}`);
  
  // Create a transaction with an incorrect share token ID
  const txId = Math.floor(Math.random() * 1000000); // Random transaction ID to avoid conflicts
  
  // Execute redeem with incorrect share token ID
  const redeemResult = executeContractCall(
    OPCODES.REDEEM, 
    [txId, USERS.ALICE, USERS.ALICE, 500], 
    [{ id: "5:1", value: 500, output: 0 }] // Different numeric ID than the contract ID
  );
  
  // Generate blocks to confirm transaction
  generateBlocks(2);
  
  // Verify the result
  if (redeemResult.success) {
    throw new Error('Redeem with incorrect alkane ID should fail');
  }
  
  return { redeemResult };
}

/**
 * Test Case 7: Redeem with insufficient shares
 */
function testRedeemWithInsufficientShares() {
  console.log(`${COLORS.YELLOW}Testing redeem with insufficient shares${COLORS.NC}`);
  
  // Create a transaction with insufficient shares
  const txId = Math.floor(Math.random() * 1000000); // Random transaction ID to avoid conflicts
  
  // Execute redeem with insufficient shares
  const redeemResult = executeContractCall(
    OPCODES.REDEEM, 
    [txId, USERS.ALICE, USERS.ALICE, 1000], 
    [{ id: CONTRACT_ID, value: 500, output: 0 }] // Only 500 shares for a 1000 redeem
  );
  
  // Generate blocks to confirm transaction
  generateBlocks(2);
  
  // Verify the result
  if (redeemResult.success) {
    throw new Error('Redeem with insufficient shares should fail');
  }
  
  return { redeemResult };
}

/**
 * Test Case 8: Redeem with multiple alkane IDs
 */
function testRedeemWithMultipleAlkaneIds() {
  console.log(`${COLORS.YELLOW}Testing redeem with multiple alkane IDs${COLORS.NC}`);
  
  // Create a transaction with multiple token IDs including the correct one
  const txId = Math.floor(Math.random() * 1000000); // Random transaction ID to avoid conflicts
  
  // Execute redeem with multiple token IDs
  const redeemResult = executeContractCall(
    OPCODES.REDEEM, 
    [txId, USERS.ALICE, USERS.ALICE, 500], 
    [
      { id: "5:1", value: 250, output: 0 },
      { id: CONTRACT_ID, value: 500, output: 0 },
      { id: "6:1", value: 300, output: 0 }
    ]
  );
  
  // Generate blocks to confirm transaction
  generateBlocks(2);
  
  // Verify the result
  if (!redeemResult.success) {
    throw new Error(`Redeem failed: ${redeemResult.error}`);
  }
  
  return { redeemResult };
}

/**
 * Test Case 9: No transaction context
 */
function testNoTransactionContext() {
  console.log(`${COLORS.YELLOW}Testing no transaction context${COLORS.NC}`);
  
  // Create a transaction with no alkanes
  const txId = Math.floor(Math.random() * 1000000); // Random transaction ID to avoid conflicts
  
  // Execute deposit with no alkanes
  const depositResult = executeContractCall(
    OPCODES.DEPOSIT, 
    [txId, USERS.ALICE, 1000], 
    [] // No alkanes
  );
  
  // Generate blocks to confirm transaction
  generateBlocks(2);
  
  // Verify the result
  if (depositResult.success) {
    throw new Error('Deposit with no transaction context should fail');
  }
  
  return { depositResult };
}

/**
 * Run all tests
 */
async function runAllTests() {
  try {
    console.log(`${COLORS.BOLD}${COLORS.BLUE}=== OylNet Alkane ID Verification Tests ===${COLORS.NC}\n`);
    
    console.log(`${COLORS.YELLOW}Using contract: ${CONTRACT_ID}${COLORS.NC}`);
    
    // Generate initial blocks
    generateBlocks(2);
    
    // Run tests
    runTest('Deposit with correct alkane ID', testDepositWithCorrectAlkaneId);
    runTest('Deposit with incorrect alkane ID', testDepositWithIncorrectAlkaneId);
    runTest('Deposit with multiple alkane IDs', testDepositWithMultipleAlkaneIds);
    runTest('Deposit with insufficient assets', testDepositWithInsufficientAssets);
    runTest('Redeem with correct alkane ID', testRedeemWithCorrectAlkaneId);
    runTest('Redeem with incorrect alkane ID', testRedeemWithIncorrectAlkaneId);
    runTest('Redeem with insufficient shares', testRedeemWithInsufficientShares);
    runTest('Redeem with multiple alkane IDs', testRedeemWithMultipleAlkaneIds);
    runTest('No transaction context', testNoTransactionContext);
    
    // Generate test report
    generateTestReport();
    
    // Print summary
    console.log(`\n${COLORS.BOLD}${COLORS.BLUE}=== Test Summary ===${COLORS.NC}`);
    console.log(`${COLORS.BOLD}Total Tests: ${testResults.passed + testResults.failed}${COLORS.NC}`);
    console.log(`${COLORS.GREEN}Passed: ${testResults.passed}${COLORS.NC}`);
    console.log(`${COLORS.RED}Failed: ${testResults.failed}${COLORS.NC}`);
    
    if (testResults.failed === 0) {
      console.log(`\n${COLORS.BOLD}${COLORS.GREEN}=== All Tests Passed ===${COLORS.NC}`);
    } else {
      console.log(`\n${COLORS.BOLD}${COLORS.RED}=== Some Tests Failed ===${COLORS.NC}`);
    }
    
  } catch (error) {
    console.error(`${COLORS.RED}Error running tests: ${error.message}${COLORS.NC}`);
    process.exit(1);
  }
}

// Run all tests
runAllTests();
