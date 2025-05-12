/**
 * YieldVault Contract Deployment Script
 * 
 * This script deploys the YieldVault contract to OylNet using simplified steps
 * and working around address compatibility issues.
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

/**
 * Display a step header
 * @param {string} stepNumber - Step number
 * @param {string} description - Step description
 */
function displayStepHeader(stepNumber, description) {
  console.log(`\n${COLORS.BOLD}${COLORS.BLUE}Step ${stepNumber}: ${description}${COLORS.NC}`);
}

/**
 * Execute a command with proper logging
 * @param {string} command - Command to execute
 * @param {string} description - Description of what the command does
 * @param {boolean} captureOutput - Whether to capture and return output
 * @returns {string|null} - Command output if captureOutput is true
 */
function executeCommand(command, description, captureOutput = true) {
  console.log(`${COLORS.CYAN}$ ${command}${COLORS.NC}`);
  
  try {
    const output = execSync(command, { 
      encoding: 'utf8',
      stdio: captureOutput ? 'pipe' : 'inherit' 
    });
    
    if (captureOutput) {
      console.log(`${COLORS.GREEN}✅ ${description} successful${COLORS.NC}`);
      return output;
    } else {
      return null;
    }
  } catch (error) {
    console.log(`${COLORS.RED}❌ ${description} failed${COLORS.NC}`);
    console.log('Error:');
    console.log(error.message);
    return null;
  }
}

/**
 * Deploy the contract
 */
function deployContract() {
  console.log(`${COLORS.BOLD}${COLORS.BLUE}=== YieldVault Contract Deployment ====${COLORS.NC}`);
  
  try {
    // Step 1: Test connection
    displayStepHeader(1, "Testing OylNet Connection");
    const blockResult = executeCommand(
      `/bin/bash -c "source ${ROOT_DIR}/.env && oyl regtest genBlocks -p oylnet -c 1"`, 
      "OylNet block generation"
    );
    
    if (!blockResult) {
      throw new Error("Failed to connect to OylNet");
    }
    
    // Step 2: Build or copy WebAssembly binary
    displayStepHeader(2, "Preparing WebAssembly Binary");
    const wasmPath = path.join(ROOT_DIR, 'target/wasm32-unknown-unknown/release/yield_vault.wasm');
    
    if (!fs.existsSync(wasmPath)) {
      console.log(`${COLORS.YELLOW}WebAssembly binary not found at ${wasmPath}${COLORS.NC}`);
      console.log(`${COLORS.YELLOW}Building WebAssembly binary...${COLORS.NC}`);
      
      executeCommand('cargo build --target wasm32-unknown-unknown --release', 'WebAssembly build', false);
      
      if (!fs.existsSync(wasmPath)) {
        throw new Error("Failed to build WebAssembly binary");
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
    executeCommand(
      `/bin/bash -c "source ${ROOT_DIR}/.env && oyl regtest genBlocks -p oylnet -c 20"`,
      "Block generation to prepare network",
      false
    );
    
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
    
    // Step 5: Deploy the contract
    displayStepHeader(5, "Deploying Contract");
    console.log(`${COLORS.YELLOW}Attempting to deploy the contract with faucet funds...${COLORS.NC}`);
    
    // Special command - capturing output but showing it in real time
    const deployCommand = `/bin/bash -c "source ${ROOT_DIR}/.env && oyl alkane new-contract --contract '${ROOT_DIR}/build/yield_vault.wasm' --provider 'oylnet' --calldata '${calldata}' --feeRate 1"`;
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
        // This is our key issue - try to work around it by using the --init option
        console.log(`${COLORS.YELLOW}Detected insufficient balance issue. Attempting alternative deployment...${COLORS.NC}`);
        
        // Try initializing the environment with more blocks and explicit address generation
        executeCommand(
          `/bin/bash -c "source ${ROOT_DIR}/.env && oyl regtest init -p oylnet"`,
          "Reinitializing OylNet environment",
          false
        );
        
        // Generate more blocks to ensure funding
        executeCommand(
          `/bin/bash -c "source ${ROOT_DIR}/.env && oyl regtest genBlocks -p oylnet -c 50"`,
          "Generating additional blocks",
          false
        );
        
        // Try deployment again, with lower fee rate
        const altDeployCommand = `/bin/bash -c "source ${ROOT_DIR}/.env && oyl alkane new-contract --contract '${ROOT_DIR}/build/yield_vault.wasm' --provider 'oylnet' --calldata '${calldata}' --feeRate 0.1"`;
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
      executeCommand(
        `/bin/bash -c "source ${ROOT_DIR}/.env && oyl regtest genBlocks -p oylnet -c 5"`,
        "Generating blocks to confirm deployment",
        false
      );
      
      // Success!
      console.log(`\n${COLORS.BOLD}${COLORS.GREEN}=== Contract Deployed Successfully! ====${COLORS.NC}`);
      console.log(`${COLORS.GREEN}Contract ID: ${contractId}${COLORS.NC}`);
      console.log(`${COLORS.GREEN}To interact with the contract, run:${COLORS.NC}`);
      console.log(`  ./bin/net/structured_network.sh --interact`);
      
      return true;
    } else {
      throw new Error("Could not extract contract ID from deployment output");
    }
  } catch (error) {
    console.log(`\n${COLORS.BOLD}${COLORS.RED}=== Deployment Failed! ====${COLORS.NC}`);
    console.log(`${COLORS.RED}Error: ${error.message}${COLORS.NC}`);
    
    // Create a deployment log for debugging
    const logContent = `
DEPLOYMENT FAILURE LOG
======================
Time: ${new Date().toISOString()}
Error: ${error.message}
Stack: ${error.stack}

This typically indicates an issue with OylNet configuration or insufficient funds.
Please check:
1. The OylNet environment is correctly set up
2. The wallet has sufficient funds
3. The WebAssembly binary is correctly built
`;
    
    fs.writeFileSync(`${ROOT_DIR}/deployment_failure.log`, logContent);
    console.log(`${COLORS.YELLOW}Deployment failure log written to deployment_failure.log${COLORS.NC}`);
    
    return false;
  }
}

/**
 * Extract contract ID from deployment output
 * @param {string} deployResult - Output from the deployment command
 * @returns {string|null} - Contract ID if found, null otherwise
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

// Execute the deployment
const success = deployContract();
process.exit(success ? 0 : 1);
