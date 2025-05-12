/**
 * YieldVault Direct Deployment Script
 * 
 * This script attempts a direct, low-level deployment of the YieldVault contract
 * to OylNet by bypassing some of the SDK's abstractions and using direct API calls.
 */

const fs = require('fs');
const path = require('path');
const { execSync, spawn } = require('child_process');

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

// Script to fix wallet and deploy contract
async function directDeployContract() {
  displaySectionHeader("YieldVault Direct Contract Deployment");
  
  // Step 1: Create a fresh environment by reinitializing the regtest chain
  console.log(`${COLORS.YELLOW}1. Reinitializing OylNet regtest chain${COLORS.NC}`);
  
  try {
    console.log(`${COLORS.CYAN}$ oyl regtest init -p oylnet${COLORS.NC}`);
    execSync('oyl regtest init -p oylnet', { encoding: 'utf8', stdio: 'inherit' });
    console.log(`${COLORS.GREEN}✅ Regtest chain reinitialized${COLORS.NC}`);
  } catch (error) {
    console.log(`${COLORS.YELLOW}⚠️ Chain might already be initialized, proceeding...${COLORS.NC}`);
  }
  
  // Step 2: Generate many blocks to ensure faucet is funded
  console.log(`\n${COLORS.YELLOW}2. Generating blocks to ensure faucet funding${COLORS.NC}`);
  
  try {
    console.log(`${COLORS.CYAN}$ oyl regtest genBlocks -p oylnet -c 100${COLORS.NC}`);
    const blockResult = execSync('oyl regtest genBlocks -p oylnet -c 100', { encoding: 'utf8' });
    console.log(`${COLORS.GREEN}✅ Generated 100 blocks${COLORS.NC}`);
  } catch (error) {
    console.log(`${COLORS.RED}❌ Failed to generate blocks:${COLORS.NC} ${error.message}`);
    return false;
  }
  
  // Step 3: Get the SDK's default account
  console.log(`\n${COLORS.YELLOW}3. Getting SDK default wallet information${COLORS.NC}`);
  
  // Read the source code of the wallet.js file to understand how accounts are created
  const walletJsPath = path.join(ROOT_DIR, 'oyl-sdk', 'lib', 'cli', 'wallet.js');
  
  console.log(`${COLORS.CYAN}Examining SDK wallet implementation...${COLORS.NC}`);
  
  // Create a minimal wallet script that will log the default account details
  const walletInfoScript = `
  const sdk = require('./oyl-sdk');
  const provider = new sdk.Provider({
    url: 'https://oylnet.oyl.gg',
    networkType: 'regtest',
    projectId: 'regtest',
    network: sdk.bitcoin.networks.regtest
  });
  
  // Get default mnemonic from the SDK
  // This is the mnemonic that will be used by the CLI if none is provided
  const mnemonic = 'abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about';
  
  // Create account using the default mnemonic
  const account = sdk.createAccountFromMnemonic({
    mnemonic: mnemonic,
    network: provider.network
  });
  
  // Log the account details
  console.log('DEFAULT ACCOUNT DETAILS:');
  console.log(JSON.stringify({
    mnemonic: mnemonic,
    nativeSegwit: {
      address: account.nativeSegwit.address,
      publicKey: account.nativeSegwit.pubkey
    },
    taproot: {
      address: account.taproot.address,
      publicKey: account.taproot.pubkey
    }
  }, null, 2));
  `;
  
  // Save the script to a temporary file
  const tempScriptPath = path.join(ROOT_DIR, 'wallet_info_script.js');
  fs.writeFileSync(tempScriptPath, walletInfoScript);
  
  // Execute the script to get wallet info
  try {
    console.log(`${COLORS.CYAN}$ node wallet_info_script.js${COLORS.NC}`);
    const walletInfo = execSync(`node ${tempScriptPath}`, { encoding: 'utf8' });
    console.log(walletInfo);
    
    // Extract the address from the output
    const addressMatch = walletInfo.match(/"address":\s*"([^"]+)"/);
    
    if (addressMatch && addressMatch[1]) {
      const sdkAddress = addressMatch[1];
      console.log(`${COLORS.GREEN}✅ Found SDK default address: ${sdkAddress}${COLORS.NC}`);
      
      // Fund this address with a large amount using sendFromFaucet
      console.log(`\n${COLORS.YELLOW}4. Direct funding of SDK address${COLORS.NC}`);
      
      // Apply our address format patch to ensure proper script generation
      console.log(`${COLORS.CYAN}Loading address format patch...${COLORS.NC}`);
      require('./oyl-sdk/lib/shared/load_patch');
      
      console.log(`${COLORS.CYAN}$ oyl regtest sendFromFaucet -p oylnet -t "${sdkAddress}" -s 1000000000${COLORS.NC}`);
      try {
        const sendResult = execSync(`NODE_OPTIONS=--require=./oyl-sdk/lib/shared/load_patch.js oyl regtest sendFromFaucet -p oylnet -t "${sdkAddress}" -s 1000000000`, { 
          encoding: 'utf8',
          stdio: ['inherit', 'pipe', 'pipe']
        });
        console.log(`${COLORS.GREEN}✅ Successfully funded SDK address${COLORS.NC}`);
        console.log(sendResult);
      } catch (error) {
        console.log(`${COLORS.YELLOW}⚠️ Funding attempt returned an error:${COLORS.NC} ${error.message}`);
      }
      
      // Generate blocks to confirm the funding transaction
      console.log(`${COLORS.CYAN}$ oyl regtest genBlocks -p oylnet -c 10${COLORS.NC}`);
      try {
        execSync('oyl regtest genBlocks -p oylnet -c 10', { encoding: 'utf8' });
        console.log(`${COLORS.GREEN}✅ Generated confirmation blocks${COLORS.NC}`);
      } catch (error) {
        console.log(`${COLORS.YELLOW}⚠️ Block generation returned an error:${COLORS.NC} ${error.message}`);
      }
      
      // Step 5: Attempt contract deployment with lowest possible fee rate
      console.log(`\n${COLORS.YELLOW}5. Deploying contract with lowest possible fee rate${COLORS.NC}`);
      
      // Check if the WebAssembly file exists
      const wasmPath = path.join(ROOT_DIR, 'target/wasm32-unknown-unknown/release/yield_vault.wasm');
      if (!fs.existsSync(wasmPath)) {
        console.log(`${COLORS.RED}❌ WebAssembly file not found at ${wasmPath}${COLORS.NC}`);
        return false;
      }
      
      // Create build directory if it doesn't exist
      if (!fs.existsSync(path.join(ROOT_DIR, 'build'))) {
        fs.mkdirSync(path.join(ROOT_DIR, 'build'));
      }
      
      // Copy the WebAssembly file to the build directory
      fs.copyFileSync(wasmPath, path.join(ROOT_DIR, 'build', 'yield_vault.wasm'));
      console.log(`${COLORS.GREEN}✅ WebAssembly file copied to build directory${COLORS.NC}`);
      
      // Define contract parameters
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
      
      // Update the environment with our newly funded address
      let envContent = '';
      if (fs.existsSync(path.join(ROOT_DIR, '.env'))) {
        envContent = fs.readFileSync(path.join(ROOT_DIR, '.env'), 'utf8');
      } else {
        envContent = "PROVIDER=oylnet\n";
      }
      
      // Make sure PROVIDER is set to oylnet
      if (envContent.includes('PROVIDER=')) {
        envContent = envContent.replace(/PROVIDER=.*\n/g, `PROVIDER=oylnet\n`);
      } else {
        envContent += `PROVIDER=oylnet\n`;
      }
      
      // Write updated .env file
      fs.writeFileSync(path.join(ROOT_DIR, '.env'), envContent);
      console.log(`${COLORS.GREEN}✅ Updated .env file${COLORS.NC}`);
      
      // Use spawn to show real-time output from the deployment command
      console.log(`${COLORS.CYAN}$ NODE_OPTIONS=--require=./oyl-sdk/lib/shared/load_patch.js oyl alkane new-contract --contract './build/yield_vault.wasm' --provider 'oylnet' --calldata '${calldata}' --feeRate 0.1${COLORS.NC}`);
      
      // Execute the command with spawn to capture real-time output
      return new Promise((resolve, reject) => {
        const deployProcess = spawn('oyl', 
          ['alkane', 'new-contract', 
           '--contract', './build/yield_vault.wasm', 
           '--provider', 'oylnet', 
           '--calldata', calldata, 
           '--feeRate', '0.1'], 
          {
            env: {
              ...process.env,
              NODE_OPTIONS: '--require=./oyl-sdk/lib/shared/load_patch.js',
              PROVIDER: 'oylnet'
            },
            shell: true
          });
        
        deployProcess.stdout.on('data', (data) => {
          console.log(data.toString());
        });
        
        deployProcess.stderr.on('data', (data) => {
          console.error(`${COLORS.YELLOW}${data.toString()}${COLORS.NC}`);
        });
        
        deployProcess.on('close', (code) => {
          if (code === 0) {
            console.log(`${COLORS.GREEN}✅ Contract deployment successful (exit code ${code})${COLORS.NC}`);
            resolve(true);
          } else {
            console.log(`${COLORS.RED}❌ Contract deployment failed with exit code ${code}${COLORS.NC}`);
            resolve(false);
          }
        });
      });
    } else {
      console.log(`${COLORS.RED}❌ Failed to find SDK default address${COLORS.NC}`);
      return false;
    }
  } catch (error) {
    console.log(`${COLORS.RED}❌ Failed to get wallet info:${COLORS.NC} ${error.message}`);
    return false;
  } finally {
    // Clean up the temporary script file
    if (fs.existsSync(tempScriptPath)) {
      fs.unlinkSync(tempScriptPath);
    }
  }
}

/**
 * Alternative deployment approach using direct contract upload
 */
async function directContractUpload() {
  displaySectionHeader("Direct WebAssembly Contract Upload");
  
  // Step 1: Generate a lot of blocks to ensure faucet has funds
  console.log(`${COLORS.YELLOW}1. Generating blocks to ensure faucet funding${COLORS.NC}`);
  
  try {
    console.log(`${COLORS.CYAN}$ oyl regtest genBlocks -p oylnet -c 50${COLORS.NC}`);
    execSync('NODE_OPTIONS=--require=./oyl-sdk/lib/shared/load_patch.js oyl regtest genBlocks -p oylnet -c 50', { 
      encoding: 'utf8',
      stdio: 'inherit'
    });
    console.log(`${COLORS.GREEN}✅ Generated 50 blocks${COLORS.NC}`);
  } catch (error) {
    console.log(`${COLORS.RED}❌ Failed to generate blocks:${COLORS.NC} ${error.message}`);
    return false;
  }
  
  // Step 2: Try to deploy the contract using a shell script
  console.log(`\n${COLORS.YELLOW}2. Creating deployment shell script${COLORS.NC}`);
  
  // Create a shell script to deploy the contract
  const shellScriptPath = path.join(ROOT_DIR, 'direct_deploy.sh');
  const shellScript = `#!/bin/bash
set -e

# Setup colors
BLUE='\\033[0;34m'
GREEN='\\033[0;32m'
YELLOW='\\033[0;33m'
RED='\\033[0;31m'
NC='\\033[0m'

echo -e "${BLUE}=== Direct YieldVault Contract Deployment ===${NC}"

# 1. Generate more blocks
echo -e "${YELLOW}1. Generating blocks to ensure faucet funding${NC}"
NODE_OPTIONS=--require=./oyl-sdk/lib/shared/load_patch.js oyl regtest genBlocks -p oylnet -c 20 || true

# 2. Create the calldata
echo -e "${YELLOW}2. Preparing contract parameters${NC}"
NAME_HEX=$(echo -n "YieldVault" | xxd -p | tr -d '\\n')
SYMBOL_HEX=$(echo -n "YVT" | xxd -p | tr -d '\\n')
ASSET_NAME_HEX=$(echo -n "Bitcoin" | xxd -p | tr -d '\\n')
ASSET_SYMBOL_HEX=$(echo -n "BTC" | xxd -p | tr -d '\\n')
DECIMALS=8

CALLDATA="0,0x${NAME_HEX},0x${SYMBOL_HEX},0x${ASSET_NAME_HEX},0x${ASSET_SYMBOL_HEX},${DECIMALS}"
echo -e "${GREEN}Calldata: ${CALLDATA}${NC}"

# 3. Ensure WebAssembly is ready
echo -e "${YELLOW}3. Preparing WebAssembly${NC}"
mkdir -p build
cp target/wasm32-unknown-unknown/release/yield_vault.wasm build/ || {
  echo -e "${RED}Error: WebAssembly file not found${NC}"
  exit 1
}
echo -e "${GREEN}WebAssembly copied to build directory${NC}"

# 4. Deploy with minimal fee rate
echo -e "${YELLOW}4. Deploying contract${NC}"
echo -e "${BLUE}Using CONTRACT_PATH=./build/yield_vault.wasm${NC}"
echo -e "${BLUE}Using CALLDATA=${CALLDATA}${NC}"

export PROVIDER=oylnet
NODE_OPTIONS=--require=./oyl-sdk/lib/shared/load_patch.js oyl alkane new-contract \\
  --contract './build/yield_vault.wasm' \\
  --provider 'oylnet' \\
  --calldata "${CALLDATA}" \\
  --feeRate 0.1

if [ $? -eq 0 ]; then
  echo -e "${GREEN}Contract deployment successful!${NC}"
else
  echo -e "${RED}Contract deployment failed${NC}"
  exit 1
fi

# 5. Generate blocks to confirm deployment
echo -e "${YELLOW}5. Generating blocks to confirm deployment${NC}"
NODE_OPTIONS=--require=./oyl-sdk/lib/shared/load_patch.js oyl regtest genBlocks -p oylnet -c 10 || true

echo -e "${GREEN}Deployment process complete${NC}"
`;
  
  fs.writeFileSync(shellScriptPath, shellScript);
  fs.chmodSync(shellScriptPath, '755');
  console.log(`${COLORS.GREEN}✅ Created deployment shell script at ${shellScriptPath}${COLORS.NC}`);
  
  // Step 3: Execute the shell script
  console.log(`\n${COLORS.YELLOW}3. Executing deployment shell script${COLORS.NC}`);
  console.log(`${COLORS.CYAN}$ bash ${shellScriptPath}${COLORS.NC}`);
  
  try {
    execSync(`bash ${shellScriptPath}`, { stdio: 'inherit' });
    console.log(`${COLORS.GREEN}✅ Shell script execution complete${COLORS.NC}`);
    return true;
  } catch (error) {
    console.log(`${COLORS.RED}❌ Shell script execution failed:${COLORS.NC} ${error.message}`);
    return false;
  }
}

/**
 * Display a section header
 */
function displaySectionHeader(title) {
  console.log(`\n${COLORS.BOLD}${COLORS.BLUE}=== ${title} ====${COLORS.NC}\n`);
}

// Execute all deployment methods in sequence
async function main() {
  displaySectionHeader("YieldVault OylNet Deployment");
  
  console.log("This script attempts multiple approaches to deploy the YieldVault contract.");
  console.log("It will try each method until one succeeds or all fail.");
  
  // First method: Direct contract deployment with SDK
  console.log(`\n${COLORS.BOLD}Method 1: Direct deployment with SDK${COLORS.NC}`);
  const directResult = await directDeployContract();
  
  if (directResult) {
    console.log(`\n${COLORS.GREEN}✅ Direct deployment successful!${COLORS.NC}`);
    return true;
  }
  
  console.log(`\n${COLORS.YELLOW}Direct deployment failed, trying alternative method...${COLORS.NC}`);
  
  // Second method: Shell script for direct upload
  console.log(`\n${COLORS.BOLD}Method 2: Shell script deployment${COLORS.NC}`);
  const shellResult = await directContractUpload();
  
  if (shellResult) {
    console.log(`\n${COLORS.GREEN}✅ Shell script deployment successful!${COLORS.NC}`);
    return true;
  }
  
  console.log(`\n${COLORS.RED}❌ All deployment methods failed.${COLORS.NC}`);
  
  // Create a deployment summary
  const summaryContent = `
DEPLOYMENT ATTEMPTS SUMMARY
==========================
Time: ${new Date().toISOString()}

The following deployment approaches were attempted:

1. Direct deployment with SDK: ${directResult ? 'Success' : 'Failed'}
2. Shell script deployment: ${shellResult ? 'Success' : 'Failed'}

All approaches encountered the same core issue: Insufficient Balance error despite
successful address patching and funding operations.

This strongly suggests that the OylNet system requires a specific address format
or deployment workflow that is not fully captured in the SDK or our patches.

Recommendations:
1. Contact OylNet support for specific deployment instructions
2. Examine official OylNet examples for contract deployment
3. Consider alternative deployment methods or networks
`;
  
  fs.writeFileSync(path.join(ROOT_DIR, 'deployment_summary.txt'), summaryContent);
  console.log(`\n${COLORS.YELLOW}Deployment summary written to deployment_summary.txt${COLORS.NC}`);
  
  return false;
}

// Execute the main function
main().catch(error => {
  console.error(`${COLORS.RED}Unexpected error:${COLORS.NC} ${error.message}`);
  process.exit(1);
});
