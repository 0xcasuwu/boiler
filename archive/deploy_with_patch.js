
/**
 * YieldVault Contract Deployment Script with OylNet Compatibility Patch
 * 
 * This script deploys the YieldVault contract to OylNet using a custom patch
 * to work around address format incompatibility issues.
 */

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

// Load our custom patch for OylNet address compatibility
const patchLoader = path.join(__dirname, 'oyl-sdk', 'lib', 'shared', 'load_patch.js');
require(patchLoader);
console.log('OylNet address compatibility patch loaded');

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
 * Display a step header
 */
function displayStepHeader(stepNumber, description) {
  console.log(`\n${COLORS.BOLD}${COLORS.BLUE}Step ${stepNumber}: ${description}${COLORS.NC}`);
}

/**
 * Deploy the contract with our patched address handling
 */
function deployContractWithPatch() {
  console.log(`${COLORS.BOLD}${COLORS.BLUE}=== YieldVault Contract Deployment (Patched) ===${COLORS.NC}`);
  
  // Steps 1-3: Test connection, prepare WebAssembly, and generate blocks
  try {
    // Step 1: Test connection
    displayStepHeader(1, "Testing OylNet Connection (with patch)");
    try {
      const blockResult = execSync(
        `/bin/bash -c "source ${ROOT_DIR}/.env && oyl regtest genBlocks -p oylnet -c 1"`, 
        { encoding: 'utf8' }
      );
      console.log(`${COLORS.GREEN}✅ OylNet block generation successful${COLORS.NC}`);
      console.log(blockResult);
    } catch (error) {
      console.log(`${COLORS.RED}❌ OylNet connection test failed${COLORS.NC}`);
      console.log(error.message);
      throw new Error("Failed to connect to OylNet");
    }
    
    // Step 2: Prepare WebAssembly Binary
    displayStepHeader(2, "Preparing WebAssembly Binary");
    const wasmPath = path.join(ROOT_DIR, 'target/wasm32-unknown-unknown/release/yield_vault.wasm');
    
    if (!fs.existsSync(wasmPath)) {
      console.log(`${COLORS.YELLOW}WebAssembly binary not found at ${wasmPath}${COLORS.NC}`);
      console.log(`${COLORS.YELLOW}Building WebAssembly binary...${COLORS.NC}`);
      
      try {
        execSync('cargo build --target wasm32-unknown-unknown --release', { 
          encoding: 'utf8', 
          stdio: 'inherit'
        });
      } catch (error) {
        throw new Error("Failed to build WebAssembly binary");
      }
      
      if (!fs.existsSync(wasmPath)) {
        throw new Error("WebAssembly binary not found after build attempt");
      }
    }
    
    // Create build directory if it doesn't exist
    if (!fs.existsSync(`${ROOT_DIR}/build`)) {
      fs.mkdirSync(`${ROOT_DIR}/build`);
    }
    
    // Copy the WASM file to the build directory
    fs.copyFileSync(wasmPath, `${ROOT_DIR}/build/yield_vault.wasm`);
    console.log(`${COLORS.GREEN}✅ WebAssembly binary copied to build directory${COLORS.NC}`);
    
    // Step 3: Generate blocks to ensure the faucet is funded
    displayStepHeader(3, "Preparing Network");
    try {
      const prepResult = execSync(
        `/bin/bash -c "source ${ROOT_DIR}/.env && oyl regtest genBlocks -p oylnet -c 20"`,
        { encoding: 'utf8' }
      );
      console.log(`${COLORS.GREEN}✅ Block generation successful${COLORS.NC}`);
      console.log(prepResult);
    } catch (error) {
      console.log(`${COLORS.YELLOW}⚠️ Warning: Block generation returned an error${COLORS.NC}`);
      console.log(error.message);
      // Continue anyway, this isn't fatal
    }
    
    // Step 4: Define contract initialization parameters
    displayStepHeader(4, "Setting Contract Parameters");
    const name = "YieldVault";
    const symbol = "YVT";
    const assetName = "Bitcoin";
    const assetSymbol = "BTC";
    const decimals = 8;
    
    // Convert parameters to hex
    const nameHex = Buffer.from(name).toString('hex');
    const symbolHex = Buffer.from(symbol).toString('hex');
    const assetNameHex = Buffer.from(assetName).toString('hex');
    const assetSymbolHex = Buffer.from(assetSymbol).toString('hex');
    
    // Initialize calldata for deployment
    const calldata = `0,0x${nameHex},0x${symbolHex},0x${assetNameHex},0x${assetSymbolHex},${decimals}`;
    
    console.log(`${COLORS.BLUE}📝 Deployment Parameters:${COLORS.NC}`);
    console.log(`  - Vault Name: ${name}`);
    console.log(`  - Vault Symbol: ${symbol}`);
    console.log(`  - Asset Name: ${assetName}`);
    console.log(`  - Asset Symbol: ${assetSymbol}`);
    console.log(`  - Decimals: ${decimals}`);
    console.log(`  - Calldata: ${calldata}`);
    
    // Step 5: Deploy the contract with our patched address handling
    displayStepHeader(5, "Deploying Contract (with patched address handling)");
    console.log(`${COLORS.YELLOW}Attempting to deploy the contract with patched address validation...${COLORS.NC}`);
    
    // Add NODE_OPTIONS to force require our patch script
    const deployCommand = `/bin/bash -c "source ${ROOT_DIR}/.env && NODE_OPTIONS=--require=./oyl-sdk/lib/shared/load_patch.js oyl alkane new-contract --contract '${ROOT_DIR}/build/yield_vault.wasm' --provider 'oylnet' --calldata '${calldata}' --feeRate 1"`;
    console.log(`${COLORS.CYAN}$ ${deployCommand}${COLORS.NC}`);
    
    let deployResult;
    try {
      deployResult = execSync(deployCommand, { 
        encoding: 'utf8',
        stdio: ['inherit', 'pipe', 'pipe']
      });
      console.log(`${COLORS.GREEN}✅ Contract deployment successful${COLORS.NC}`);
      console.log('Deployment result:');
      console.log(deployResult);
    } catch (error) {
      console.log(`${COLORS.RED}❌ Contract deployment failed${COLORS.NC}`);
      console.log('Error:');
      console.log(error.message);
      
      if (error.message.includes('Insufficient Balance')) {
        // Try with a lower fee rate
        console.log(`${COLORS.YELLOW}Retrying with lower fee rate...${COLORS.NC}`);
        
        // Generate more blocks to ensure funding
        try {
          execSync(`/bin/bash -c "source ${ROOT_DIR}/.env && oyl regtest genBlocks -p oylnet -c 50"`, {
            encoding: 'utf8'
          });
          console.log(`${COLORS.GREEN}✅ Generated additional blocks${COLORS.NC}`);
        } catch (genError) {
          console.log(`${COLORS.YELLOW}⚠️ Warning: Additional block generation failed${COLORS.NC}`);
          console.log(genError.message);
        }
        
        // Try deployment again with lower fee rate
        const altDeployCommand = `/bin/bash -c "source ${ROOT_DIR}/.env && NODE_OPTIONS=--require=./oyl-sdk/lib/shared/load_patch.js oyl alkane new-contract --contract '${ROOT_DIR}/build/yield_vault.wasm' --provider 'oylnet' --calldata '${calldata}' --feeRate 0.1"`;
        console.log(`${COLORS.CYAN}$ ${altDeployCommand}${COLORS.NC}`);
        
        try {
          deployResult = execSync(altDeployCommand, { 
            encoding: 'utf8',
            stdio: ['inherit', 'pipe', 'pipe']
          });
          console.log(`${COLORS.GREEN}✅ Alternative contract deployment successful${COLORS.NC}`);
          console.log('Deployment result:');
          console.log(deployResult);
        } catch (altError) {
          console.log(`${COLORS.RED}❌ Alternative contract deployment also failed${COLORS.NC}`);
          console.log('Error:');
          console.log(altError.message);
          throw new Error("All deployment attempts failed");
        }
      } else {
        throw error;
      }
    }
    
    // Extract contract ID
    const contractId = extractContractId(deployResult);
    
    if (contractId) {
      console.log(`${COLORS.BLUE}Contract ID: ${COLORS.BOLD}${contractId}${COLORS.NC}`);
      fs.writeFileSync(`${ROOT_DIR}/.contract_id`, contractId);
      console.log(`${COLORS.GREEN}✅ Contract ID saved to .contract_id file${COLORS.NC}`);
      
      // Generate blocks to confirm deployment
      try {
        execSync(`/bin/bash -c "source ${ROOT_DIR}/.env && oyl regtest genBlocks -p oylnet -c 5"`, {
          encoding: 'utf8'
        });
        console.log(`${COLORS.GREEN}✅ Generated confirmation blocks${COLORS.NC}`);
      } catch (error) {
        console.log(`${COLORS.YELLOW}⚠️ Warning: Confirmation block generation failed${COLORS.NC}`);
        console.log(error.message);
      }
      
      // Success!
      console.log(`\n${COLORS.BOLD}${COLORS.GREEN}=== Contract Deployed Successfully! ===${COLORS.NC}`);
      console.log(`${COLORS.GREEN}Contract ID: ${contractId}${COLORS.NC}`);
      console.log(`${COLORS.GREEN}To interact with the contract, run:${COLORS.NC}`);
      console.log(`  ./bin/net/structured_network.sh --interact`);
      
      return true;
    } else {
      throw new Error("Could not extract contract ID from deployment output");
    }
  } catch (error) {
    console.log(`\n${COLORS.BOLD}${COLORS.RED}=== Deployment Failed! ===${COLORS.NC}`);
    console.log(`${COLORS.RED}Error: ${error.message}${COLORS.NC}`);
    
    // Create a deployment log for debugging
    const logContent = `
DEPLOYMENT FAILURE LOG
======================
Time: ${new Date().toISOString()}
Error: ${error.message}
Stack: ${error.stack}

This log was created during a deployment attempt with the OylNet address compatibility patch.
`;
    
    fs.writeFileSync(`${ROOT_DIR}/deployment_output.txt`, logContent);
    console.log(`${COLORS.YELLOW}Deployment failure log written to deployment_output.txt${COLORS.NC}`);
    
    return false;
  }
}

/**
 * Extract contract ID from deployment output
 */
function extractContractId(deployResult) {
  // Try the first format: "txId": "abcdef123..."
  let match = deployResult.match(/"txId"\s*:\s*"([0-9a-f]+)"/);
  
  if (!match) {
    // Try alternative format: txId: 'abcdef123...'
    match = deployResult.match(/txId\s*:\s*['"]?([0-9a-f]+)['"]?/);
  }
  
  return match ? match[1] : null;
}

// Execute the deployment with patched address handling
console.log(`${COLORS.BOLD}${COLORS.BLUE}OylNet YieldVault Deployment (Patched Version)${COLORS.NC}`);
console.log('This script uses a custom patch to resolve address compatibility issues');

const success = deployContractWithPatch();
process.exit(success ? 0 : 1);
