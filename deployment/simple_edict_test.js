/**
 * Simple test to verify edicts parameter works
 */

// Import the OYL SDK with the patch applied
require('/workspaces/boiler/oyl-sdk/lib/shared/load_patch.js');
const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

// Path to the contract details file
const CONTRACT_DETAILS_FILE = path.resolve(__dirname, 'contract_details.json');

// Load contract details
const contractDetails = JSON.parse(fs.readFileSync(CONTRACT_DETAILS_FILE, 'utf8'));
const CONTRACT_ID = contractDetails.contractId;

// Asset ID (using block:tx format for numeric ID)
const ASSET_ID = "2:1"; // Using block 2, tx 1 as the asset ID for testing

// Execute a command and return the output
function executeCommand(command) {
  console.log(`$ ${command}`);
  try {
    const output = execSync(command, { encoding: 'utf8' });
    console.log(output);
    return output.trim();
  } catch (error) {
    console.error(`Command failed: ${error.message}`);
    if (error.stdout) console.error(`stdout: ${error.stdout}`);
    if (error.stderr) console.error(`stderr: ${error.stderr}`);
    throw error;
  }
}

// Generate blocks on OylNet
function generateBlocks(count) {
  console.log(`Generating ${count} blocks...`);
  executeCommand(`NODE_OPTIONS=--require=/workspaces/boiler/oyl-sdk/lib/shared/load_patch.js oyl regtest genBlocks -p oylnet -c ${count}`);
}

// Main function
async function main() {
  try {
    console.log(`=== Simple Edict Test ===`);
    console.log(`Using contract: ${CONTRACT_ID}`);
    
    // Generate initial blocks
    generateBlocks(2);
    
    // Random transaction ID to avoid conflicts
    const txId = Math.floor(Math.random() * 1000000);
    
    // Test deposit with correct asset ID using edicts
    console.log(`Testing deposit with correct asset ID using edicts`);
    
    // Execute deposit with correct asset ID
    // Format for edicts is "block:tx:amount:output"
    const [block, tx] = ASSET_ID.split(':');
    const command = `NODE_OPTIONS=--require=/workspaces/boiler/oyl-sdk/lib/shared/load_patch.js oyl alkane execute -data "10,${txId},1,1000" --edicts "${block}:${tx}:1000:0" --provider oylnet`;
    
    try {
      executeCommand(command);
      console.log(`Deposit succeeded`);
    } catch (error) {
      console.error(`Deposit failed: ${error.message}`);
    }
    
    // Generate blocks to confirm transaction
    generateBlocks(2);
    
    console.log(`=== Test Complete ===`);
    
  } catch (error) {
    console.error(`Error running test: ${error.message}`);
    process.exit(1);
  }
}

// Run the main function
main()