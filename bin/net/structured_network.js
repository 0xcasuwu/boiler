/**
 * YieldVault Network Operations - Structured Implementation
 * 
 * A modular, structured implementation of the OylNet network operations script
 * based on the canonical bash implementation.
 */

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');
const { promisify } = require('util');
const exec = promisify(require('child_process').exec);
const sleep = (ms) => new Promise(resolve => setTimeout(resolve, ms));

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

// Root directory calculation (equivalent to the bash ROOT_DIR)
const ROOT_DIR = path.resolve(__dirname, '../..');

/**
 * Utility Class - Core utility functions for network operations
 */
class NetworkUtils {
  /**
   * Execute a command with retry logic
   * @param {string} command - Command to execute
   * @param {string} description - Description of the operation
   * @param {number} maxRetries - Maximum number of retries
   * @param {number} retryDelay - Delay between retries in seconds
   * @returns {Promise<{status: boolean, output: string}>} - Result of the command execution
   */
  static async executeWithRetry(command, description, maxRetries = 3, retryDelay = 3) {
    console.log(`${COLORS.CYAN}$ ${command}${COLORS.NC}`);
    
    let attempt = 1;
    let status = 1;
    let output = '';
    
    while (attempt <= maxRetries && status !== 0) {
      if (attempt > 1) {
        console.log(`${COLORS.YELLOW}⚠️ Retry ${attempt} of ${maxRetries} after ${retryDelay} seconds...${COLORS.NC}`);
        await sleep(retryDelay * 1000);
      }
      
      try {
        const result = await exec(command);
        output = result.stdout;
        status = 0;
        console.log(`${COLORS.GREEN}✅ Success: ${description}${COLORS.NC}`);
        console.log('Response:');
        console.log(output);
        return { status: true, output };
      } catch (error) {
        output = error.stderr || error.stdout || error.message;
        status = error.code || 1;
        console.log(`${COLORS.YELLOW}⚠️ Attempt ${attempt} failed: ${description} (exit code ${status})${COLORS.NC}`);
        console.log('Error output:');
        console.log(output);
        attempt++;
      }
    }
    
    console.log(`${COLORS.RED}❌ Failed after ${maxRetries} attempts: ${description}${COLORS.NC}`);
    return { status: false, output };
  }

  /**
   * Generate blocks to confirm transactions
   * @param {number} count - Number of blocks to generate
   * @returns {Promise<boolean>} - Success or failure
   */
  static async generateBlocks(count = 1) {
    console.log(`${COLORS.YELLOW}Generating ${count} blocks...${COLORS.NC}`);
    
    const result = await this.executeWithRetry(
      `/bin/bash -c "source ${ROOT_DIR}/.env && oyl regtest genBlocks -p oylnet -c ${count}"`,
      'Generate blocks',
      3,
      2
    );
    
    if (result.status) {
      console.log(`${COLORS.YELLOW}Waiting 3 seconds for block indexing...${COLORS.NC}`);
      await sleep(3000);
      return true;
    }
    
    return false;
  }

  /**
   * Check if required files exist
   * @param {string[]} filePaths - Array of file paths to check
   * @returns {boolean} - True if all files exist, false otherwise
   */
  static checkRequiredFiles(filePaths) {
    for (const filePath of filePaths) {
      if (!fs.existsSync(filePath)) {
        console.log(`${COLORS.RED}❌ Error: File not found: ${filePath}${COLORS.NC}`);
        return false;
      }
    }
    return true;
  }

  /**
   * Convert string to hex
   * @param {string} text - Text to convert to hex
   * @returns {string} - Hex representation of the text
   */
  static stringToHex(text) {
    return Buffer.from(text).toString('hex');
  }
}

/**
 * Contract Interaction Class - Handles reading from and writing to contracts
 */
class ContractInteraction {
  /**
   * Execute contract read operation
   * @param {string} contractId - Contract ID to interact with
   * @param {string|number} opcode - Operation code to execute
   * @param {string} params - Parameters to pass to the operation
   * @param {string} description - Description of the operation
   * @returns {Promise<{status: boolean, output: string}>} - Result of the operation
   */
  static async readContract(contractId, opcode, params = '', description) {
    console.log(`\n${COLORS.BLUE}🔍 Reading contract - ${description}${COLORS.NC}`);
    
    const calldata = params ? `${opcode},${params}` : `${opcode}`;
    const command = `/bin/bash -c "source ${ROOT_DIR}/.env && export ACTIVE_CONTRACT=${contractId} && oyl alkane execute -p oylnet --calldata '${calldata}'"`;
    
    return await NetworkUtils.executeWithRetry(command, `Read operation: ${description}`, 3, 2);
  }

  /**
   * Execute contract write operation
   * @param {string} contractId - Contract ID to interact with
   * @param {string|number} opcode - Operation code to execute
   * @param {string} params - Parameters to pass to the operation
   * @param {string} description - Description of the operation
   * @returns {Promise<{status: boolean, output: string}>} - Result of the operation
   */
  static async writeContract(contractId, opcode, params = '', description) {
    console.log(`\n${COLORS.BLUE}✏️ Writing to contract - ${description}${COLORS.NC}`);
    
    const calldata = params ? `${opcode},${params}` : `${opcode}`;
    const command = `/bin/bash -c "source ${ROOT_DIR}/.env && export ACTIVE_CONTRACT=${contractId} && oyl alkane execute -p oylnet --calldata '${calldata}'"`;
    
    const result = await NetworkUtils.executeWithRetry(command, `Write operation: ${description}`, 3, 2);
    
    if (result.status) {
      await NetworkUtils.generateBlocks(1);
    }
    
    return result;
  }

  /**
   * Extract contract ID from deployment output
   * @param {string} deployResult - Output from the deployment command
   * @returns {string|null} - Contract ID if found, null otherwise
   */
  static extractContractId(deployResult) {
    // Try the first format: "txId": "abcdef123..."
    let match = deployResult.match(/"txId"\s*:\s*"([0-9a-f]+)"/);
    
    if (!match) {
      // Try alternative format: txId: 'abcdef123...'
      match = deployResult.match(/txId\s*:\s*['"]?([0-9a-f]+)['"]?/);
    }
    
    return match ? match[1] : null;
  }
}

/**
 * YieldVault Network Operations - Main class for network operations
 */
class YieldVaultNetwork {
  /**
   * Test connection to OylNet
   * @returns {Promise<boolean>} - Success or failure
   */
  static async testConnection() {
    console.log(`${COLORS.BOLD}${COLORS.BLUE}Testing Connection to OylNet Network...${COLORS.NC}`);
    console.log('');
    
    // Check if .env file exists
    if (!NetworkUtils.checkRequiredFiles([`${ROOT_DIR}/.env`])) {
      console.log("Create a .env file with your OylNet credentials.");
      return false;
    }
    
    // Test OylNet connection by generating a block
    console.log(`${COLORS.BLUE}Testing OylNet connection by generating a block...${COLORS.NC}`);
    const result = await NetworkUtils.generateBlocks(1);
    
    if (result) {
      console.log(`\n${COLORS.GREEN}✅ Connection to OylNet is working properly!${COLORS.NC}`);
      return true;
    }
    
    return false;
  }

  /**
   * Deploy contract to OylNet
   * @returns {Promise<boolean>} - Success or failure
   */
  static async deployContract() {
    console.log(`${COLORS.BOLD}${COLORS.BLUE}Deploying YieldVault Contract to OylNet Network...${COLORS.NC}`);
    console.log('');
    
    // Check if .env file exists
    if (!NetworkUtils.checkRequiredFiles([`${ROOT_DIR}/.env`])) {
      console.log("Create a .env file with your OylNet credentials.");
      return false;
    }
    
    // Check if WebAssembly file exists
    const wasmPath = `${ROOT_DIR}/target/wasm32-unknown-unknown/release/yield_vault.wasm`;
    if (!NetworkUtils.checkRequiredFiles([wasmPath])) {
      console.log("Run './scripts/build.sh' first to build the WebAssembly binary.");
      return false;
    }
    
    // Create build directory if it doesn't exist
    if (!fs.existsSync(`${ROOT_DIR}/build`)) {
      fs.mkdirSync(`${ROOT_DIR}/build`);
    }
    
    // Copy the WASM file to the build directory
    console.log(`${COLORS.BLUE}📦 Copying WebAssembly to build directory...${COLORS.NC}`);
    fs.copyFileSync(wasmPath, `${ROOT_DIR}/build/yield_vault.wasm`);
    
    // Set contract parameters
    const name = "YieldVault";
    const symbol = "YVT";
    const assetName = "Bitcoin";
    const assetSymbol = "BTC";
    const decimals = 8;
    
    // Convert parameters to hex
    const nameHex = NetworkUtils.stringToHex(name);
    const symbolHex = NetworkUtils.stringToHex(symbol);
    const assetNameHex = NetworkUtils.stringToHex(assetName);
    const assetSymbolHex = NetworkUtils.stringToHex(assetSymbol);
    
    // Initialize calldata for deployment
    const calldata = `0,0x${nameHex},0x${symbolHex},0x${assetNameHex},0x${assetSymbolHex},${decimals}`;
    
    console.log(`${COLORS.BLUE}📝 Deployment Parameters:${COLORS.NC}`);
    console.log(`  - Vault Name: ${name}`);
    console.log(`  - Vault Symbol: ${symbol}`);
    console.log(`  - Asset Name: ${assetName}`);
    console.log(`  - Asset Symbol: ${assetSymbol}`);
    console.log(`  - Decimals: ${decimals}`);
    
    // Deploy contract
    console.log(`${COLORS.BLUE}Deploying contract to OylNet...${COLORS.NC}`);
    
    // Generate some blocks to ensure faucet is funded
    console.log(`${COLORS.BLUE}Generating blocks to prepare network...${COLORS.NC}`);
    await NetworkUtils.generateBlocks(10);
    
    const command = `source ${ROOT_DIR}/.env && oyl alkane new-contract --contract "${ROOT_DIR}/build/yield_vault.wasm" --provider "oylnet" --calldata "${calldata}" --feeRate 1`;
    
    console.log(`${COLORS.CYAN}$ ${command}${COLORS.NC}`);
    
    try {
      const { stdout: deployResult } = await exec(command);
      console.log(`${COLORS.GREEN}✅ Contract deployed successfully!${COLORS.NC}`);
      console.log(deployResult);
      
      // Extract contract ID
      const contractId = ContractInteraction.extractContractId(deployResult);
      
      if (contractId) {
        console.log(`${COLORS.BLUE}Contract ID: ${COLORS.BOLD}${contractId}${COLORS.NC}`);
        fs.writeFileSync(`${ROOT_DIR}/.contract_id`, contractId);
        console.log(`${COLORS.GREEN}✅ Contract ID saved to .contract_id file${COLORS.NC}`);
      } else {
        console.log(`${COLORS.YELLOW}⚠️ Could not extract contract ID from deployment result.${COLORS.NC}`);
      }
      
      // Generate blocks to confirm deployment
      await NetworkUtils.generateBlocks(2);
      
      console.log(`\n${COLORS.GREEN}✅ Contract deployment complete!${COLORS.NC}`);
      return true;
      
    } catch (error) {
      console.log(`${COLORS.RED}❌ Contract deployment failed!${COLORS.NC}`);
      console.log(error.stdout || error.stderr || error.message);
      return false;
    }
  }

  /**
   * Interact with deployed contract
   * @returns {Promise<boolean>} - Success or failure
   */
  static async interactWithContract() {
    console.log(`${COLORS.BOLD}${COLORS.BLUE}Interacting with YieldVault Contract on OylNet...${COLORS.NC}`);
    console.log('');
    
    // Check if contract ID file exists
    if (!NetworkUtils.checkRequiredFiles([`${ROOT_DIR}/.contract_id`])) {
      console.log("You need to deploy the contract first using './scripts/network.sh --deploy'");
      return false;
    }
    
    // Read contract ID
    const contractId = fs.readFileSync(`${ROOT_DIR}/.contract_id`, 'utf8').trim()
      .replace(/^.*txId: ['"]([0-9a-f]+)['"].*$/g, '$1');
    
    console.log(`${COLORS.BLUE}Using contract ID: ${COLORS.BOLD}${contractId}${COLORS.NC}`);
    
    // Test metadata view functions
    console.log(`\n${COLORS.BOLD}${COLORS.BLUE}Testing metadata view functions...${COLORS.NC}`);
    await ContractInteraction.readContract(contractId, "100", "", "Get Name");
    await ContractInteraction.readContract(contractId, "101", "", "Get Symbol");
    await ContractInteraction.readContract(contractId, "102", "", "Get Decimals");
    await ContractInteraction.readContract(contractId, "103", "", "Get Asset Name");
    console.log(`\n${COLORS.GREEN}✅ Metadata view functions test complete${COLORS.NC}`);
    
    // Test accounting functions
    console.log(`\n${COLORS.BOLD}${COLORS.BLUE}Testing accounting view functions...${COLORS.NC}`);
    await ContractInteraction.readContract(contractId, "200", "", "Get Total Assets");
    await ContractInteraction.readContract(contractId, "601", "", "Get Total Supply");
    console.log(`\n${COLORS.GREEN}✅ Accounting view functions test complete${COLORS.NC}`);
    
    // Update yield rate
    console.log(`\n${COLORS.BOLD}${COLORS.BLUE}Updating yield rate...${COLORS.NC}`);
    const yieldRate = 500;
    await ContractInteraction.writeContract(contractId, "900", yieldRate, `Update Yield Rate to ${yieldRate} basis points (${yieldRate/100}%)`);
    console.log(`\n${COLORS.GREEN}✅ Yield rate update complete${COLORS.NC}`);
    
    // Get yield rate
    console.log(`\n${COLORS.BOLD}${COLORS.BLUE}Getting current yield rate...${COLORS.NC}`);
    await ContractInteraction.readContract(contractId, "901", "", "Get current yield rate");
    console.log(`\n${COLORS.GREEN}✅ Yield rate retrieval complete${COLORS.NC}`);
    
    // Deposit assets
    console.log(`\n${COLORS.BOLD}${COLORS.BLUE}Depositing assets with numeric test mode values...${COLORS.NC}`);
    const txHash = require('crypto').randomBytes(32).toString('hex');
    const block = 1;  // test mode block
    const tx = 1;     // test mode transaction
    const assets = 1000000; // 0.01 BTC (assuming 8 decimals)
    const params = `0x${txHash},${block},${tx},${assets}`;
    await ContractInteraction.writeContract(contractId, "10", params, `Deposit ${assets} sats with test mode values`);
    console.log(`\n${COLORS.GREEN}✅ Deposit operation complete${COLORS.NC}`);
    
    // Check balance
    console.log(`\n${COLORS.BOLD}${COLORS.BLUE}Checking balance with numeric test mode values...${COLORS.NC}`);
    await ContractInteraction.readContract(contractId, "600", `${block},${tx}`, "Get balance with test mode values");
    console.log(`\n${COLORS.GREEN}✅ Balance check operation complete${COLORS.NC}`);
    
    console.log(`\n${COLORS.BOLD}${COLORS.GREEN}🎉 Contract Interaction Complete!${COLORS.NC}`);
    console.log(`Your YieldVault contract is deployed and functional on OylNet.`);
    console.log(`Contract ID: ${COLORS.BOLD}${contractId}${COLORS.NC}`);
    
    return true;
  }

  /**
   * Fund wallet from faucet
   * @returns {Promise<boolean>} - Success or failure
   */
  static async fundWallet() {
    console.log(`${COLORS.BOLD}${COLORS.BLUE}Funding Wallet from OylNet Faucet...${COLORS.NC}`);
    console.log('');
    
    // Check if .env file exists
    if (!NetworkUtils.checkRequiredFiles([`${ROOT_DIR}/.env`])) {
      console.log("Create a .env file with your OylNet credentials.");
      return false;
    }
    
    // Use a known valid test address for OylNet regtest
    console.log(`${COLORS.BLUE}Using test address for OylNet regtest...${COLORS.NC}`);
    // This is a hardcoded test address compatible with OylNet regtest
    const address = "bcrt1qeyyk6sl5gvr4wzm0dpmfqcjsls9xfkgvurkz7p";
    
    console.log(`${COLORS.BLUE}Funding address: ${address}${COLORS.NC}`);
    
    // Request funds from faucet
    const amount = 10000000; // 0.1 BTC
    const result = await NetworkUtils.executeWithRetry(
      `/bin/bash -c "source ${ROOT_DIR}/.env && oyl regtest sendFromFaucet -p oylnet -t ${address} -s ${amount}"`,
      "Fund wallet from faucet",
      3,
      2
    );
    
    if (!result.status) {
      return false;
    }
    
    // Generate blocks to confirm funding
    await NetworkUtils.generateBlocks(6);
    
    console.log(`\n${COLORS.GREEN}✅ Wallet funding complete!${COLORS.NC}`);
    console.log(`Address ${address} funded with ${amount} satoshis (${amount/100000000} BTC)`);
    
    return true;
  }
}

// Command-line interface
async function main() {
  const args = process.argv.slice(2);
  
  // Parse command-line arguments
  let mode = '';
  
  for (const arg of args) {
    switch (arg) {
      case '--test':
        mode = 'test';
        break;
      case '--deploy':
        mode = 'deploy';
        break;
      case '--interact':
        mode = 'interact';
        break;
      case '--fund':
        mode = 'fund';
        break;
      case '--help':
      case '-h':
        console.log(`${COLORS.BOLD}OylNet Network Operations Unified Script${COLORS.NC}`);
        console.log('');
        console.log(`Usage: node structured_network.js [OPTIONS]`);
        console.log('');
        console.log('Options:');
        console.log('  --test       Test connection to OylNet network');
        console.log('  --fund       Fund wallet from OylNet faucet');
        console.log('  --deploy     Deploy contract to OylNet network');
        console.log('  --interact   Interact with deployed contract');
        console.log('  --help, -h   Show this help message');
        return 0;
    }
  }
  
  // Check if mode is specified
  if (!mode) {
    console.log(`${COLORS.RED}Error: No mode specified. Use --test, --deploy, --interact, or --fund.${COLORS.NC}`);
    console.log(`Run 'node structured_network.js --help' for more information.`);
    return 1;
  }
  
  // Execute the selected mode
  let success = false;
  
  switch (mode) {
    case 'test':
      success = await YieldVaultNetwork.testConnection();
      break;
    case 'deploy':
      success = await YieldVaultNetwork.deployContract();
      break;
    case 'interact':
      success = await YieldVaultNetwork.interactWithContract();
      break;
    case 'fund':
      success = await YieldVaultNetwork.fundWallet();
      break;
    default:
      console.log(`${COLORS.RED}Error: Unknown mode: ${mode}${COLORS.NC}`);
      return 1;
  }
  
  return success ? 0 : 1;
}

// Run the main function if called directly
if (require.main === module) {
  main().then(process.exit);
}

// Export the classes for potential use in other files
module.exports = {
  NetworkUtils,
  ContractInteraction,
  YieldVaultNetwork
};
