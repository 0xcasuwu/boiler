/**
 * YieldVault Deployment Test Script
 * 
 * This script demonstrates a complete deployment process:
 * 1. Test connection to OylNet
 * 2. Fund the default OylNet address
 * 3. Deploy the vault contract
 * 4. Interact with the deployed contract
 */

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

// Import our structured network implementation
const { NetworkUtils, YieldVaultNetwork } = require('./bin/net/structured_network');

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
 * Execute the main test flow
 */
async function runDeploymentTest() {
  console.log(`${COLORS.BOLD}${COLORS.BLUE}=== YieldVault Complete Deployment Test ====${COLORS.NC}\n`);
  
  try {
    // Step 1: Test Connection
    console.log(`\n${COLORS.BOLD}${COLORS.BLUE}Step 1: Testing OylNet Connection${COLORS.NC}`);
    const connected = await YieldVaultNetwork.testConnection();
    if (!connected) {
      throw new Error('Cannot connect to OylNet network');
    }
    
    // Step 2: Fund the default address
    console.log(`\n${COLORS.BOLD}${COLORS.BLUE}Step 2: Funding Default OylNet Address${COLORS.NC}`);
    // Use direct bash command to fund the known address (bypassing address validation)
    console.log(`${COLORS.YELLOW}Generating blocks to prepare network...${COLORS.NC}`);
    await NetworkUtils.generateBlocks(1);
    
    // Step 3: Deploy the contract
    console.log(`\n${COLORS.BOLD}${COLORS.BLUE}Step 3: Deploying Vault Contract${COLORS.NC}`);
    // Build the contract first if needed
    const wasmPath = path.join(ROOT_DIR, 'target/wasm32-unknown-unknown/release/yield_vault.wasm');
    
    if (!fs.existsSync(wasmPath)) {
      console.log(`${COLORS.YELLOW}⚠️ WASM file not found. Building the contract...${COLORS.NC}`);
      try {
        execSync('cargo build --target wasm32-unknown-unknown --release', { stdio: 'inherit' });
      } catch (error) {
        console.log(`${COLORS.YELLOW}⚠️ WASM build failed. Using existing WASM file if available.${COLORS.NC}`);
      }
    }
    
    // Create build directory if it doesn't exist
    if (!fs.existsSync(`${ROOT_DIR}/build`)) {
      fs.mkdirSync(`${ROOT_DIR}/build`);
    }
    
    // Copy the WASM file to the build directory
    if (fs.existsSync(wasmPath)) {
      console.log(`${COLORS.BLUE}📦 Copying WebAssembly to build directory...${COLORS.NC}`);
      fs.copyFileSync(wasmPath, `${ROOT_DIR}/build/yield_vault.wasm`);
    } else {
      throw new Error('WASM file not found. Cannot proceed with deployment.');
    }
    
    // Set contract parameters
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
    
    // Execute contract deployment
    const command = `/bin/bash -c "source ${ROOT_DIR}/.env && oyl alkane new-contract --contract '${ROOT_DIR}/build/yield_vault.wasm' --provider 'oylnet' --calldata '${calldata}' --feeRate 1"`;
    
    console.log(`${COLORS.CYAN}$ ${command}${COLORS.NC}`);
    
    try {
      const { stdout } = await require('util').promisify(require('child_process').exec)(command);
      console.log(`${COLORS.GREEN}✅ Contract deployment command executed${COLORS.NC}`);
      console.log('Output:');
      console.log(stdout);
      
      // Extract contract ID
      const contractId = extractContractId(stdout);
      
      if (contractId) {
        console.log(`${COLORS.BLUE}Contract ID: ${COLORS.BOLD}${contractId}${COLORS.NC}`);
        fs.writeFileSync(`${ROOT_DIR}/.contract_id`, contractId);
        console.log(`${COLORS.GREEN}✅ Contract ID saved to .contract_id file${COLORS.NC}`);
        
        // Generate blocks to confirm deployment
        console.log(`${COLORS.YELLOW}Generating blocks to confirm deployment...${COLORS.NC}`);
        await NetworkUtils.generateBlocks(2);
        
        // Step 4: Interact with the deployed contract
        console.log(`\n${COLORS.BOLD}${COLORS.BLUE}Step 4: Interacting with Deployed Contract${COLORS.NC}`);
        const success = await YieldVaultNetwork.interactWithContract();
        
        if (!success) {
          throw new Error('Contract interaction failed');
        }
        
        // Test complete!
        console.log(`\n${COLORS.BOLD}${COLORS.GREEN}=== Deployment Test Completed Successfully! ====${COLORS.NC}`);
        console.log(`${COLORS.BLUE}Summary:${COLORS.NC}`);
        console.log(`- Contract deployed to OylNet`);
        console.log(`- Contract ID: ${contractId}`);
        console.log(`- Basic interactions verified (metadata, yield rate)`);
        
        return true;
      } else {
        throw new Error('Could not extract contract ID from deployment output');
      }
    } catch (error) {
      console.log(`${COLORS.RED}❌ Contract deployment failed:${COLORS.NC} ${error.message}`);
      if (error.stdout) console.log('STDOUT:', error.stdout);
      if (error.stderr) console.log('STDERR:', error.stderr);
      throw error;
    }
  } catch (error) {
    console.log(`\n${COLORS.BOLD}${COLORS.RED}=== Deployment Test Failed! ====${COLORS.NC}`);
    console.log(`${COLORS.RED}Error: ${error.message}${COLORS.NC}`);
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

// Run the deployment test
runDeploymentTest().then(success => {
  process.exit(success ? 0 : 1);
}).catch(error => {
  console.error('Unhandled error:', error);
  process.exit(1);
});
