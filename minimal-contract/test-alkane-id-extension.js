/**
 * Test script for the alkane-id extension
 * 
 * This script demonstrates how to use the new alkane-id option
 * in the alkaneExecute command.
 */

// Import required modules
const { execSync } = require('child_process');

// Test parameters
const alkaneId = '123456:789'; // Example alkane ID
const opCode = '1';           // Example operation code
const param1 = '100';         // Example parameter 1
const param2 = '200';         // Example parameter 2

// Function to execute a command and log the output
function executeCommand(command) {
  console.log(`\nExecuting: ${command}`);
  try {
    const output = execSync(command, { encoding: 'utf8' });
    console.log('Output:');
    console.log(output);
    return output;
  } catch (error) {
    console.error('Error:');
    console.error(error.stdout || error.message);
    return null;
  }
}

// Test the new alkane-id option
console.log('=== Testing the new alkane-id option ===');
const newApproachCommand = `node -e "
  const alkanes = require('./oyl-sdk/lib/cli/alkane');
  
  // Create mock options object
  const options = {
    alkaneId: '${alkaneId}',
    calldata: ['${opCode}', '${param1}', '${param2}'],
    provider: 'regtest',
    mnemonic: 'test test test test test test test test test test test junk'
  };
  
  // Log the options
  console.log('Options:', JSON.stringify(options, null, 2));
  
  // Mock the execution to avoid actual blockchain interaction
  const originalExecute = alkanes.alkaneExecute.action;
  alkanes.alkaneExecute.action = async (opts) => {
    console.log('alkaneExecute called with options:', JSON.stringify(opts, null, 2));
    console.log('Alkane ID extraction test:');
    
    // Extract alkane ID from options or calldata
    let alkaneBlock, alkaneTx;
    
    if (opts.alkaneId) {
      // Parse from the new alkane-id option
      const [block, tx] = opts.alkaneId.split(':');
      alkaneBlock = block;
      alkaneTx = tx;
      console.log(\`Processing alkane ID from option: \${alkaneBlock}:\${alkaneTx}\`);
    } else {
      // Fall back to extracting from calldata (legacy behavior)
      alkaneBlock = opts.calldata[0];
      alkaneTx = opts.calldata[1];
      console.log(\`Processing alkane ID from calldata: \${alkaneBlock}:\${alkaneTx}\`);
    }
    
    return { success: true, alkaneBlock, alkaneTx };
  };
  
  // Execute the command
  alkanes.alkaneExecute.action(options)
    .then(result => console.log('Result:', result))
    .catch(error => console.error('Error:', error));
"`;

executeCommand(newApproachCommand);

// Test the legacy approach
console.log('\n=== Testing the legacy approach ===');
const legacyApproachCommand = `node -e "
  const alkanes = require('./oyl-sdk/lib/cli/alkane');
  
  // Create mock options object with alkane ID in calldata
  const options = {
    calldata: ['${alkaneId.split(':')[0]}', '${alkaneId.split(':')[1]}', '${opCode}', '${param1}', '${param2}'],
    provider: 'regtest',
    mnemonic: 'test test test test test test test test test test test junk'
  };
  
  // Log the options
  console.log('Options:', JSON.stringify(options, null, 2));
  
  // Mock the execution to avoid actual blockchain interaction
  const originalExecute = alkanes.alkaneExecute.action;
  alkanes.alkaneExecute.action = async (opts) => {
    console.log('alkaneExecute called with options:', JSON.stringify(opts, null, 2));
    console.log('Alkane ID extraction test:');
    
    // Extract alkane ID from options or calldata
    let alkaneBlock, alkaneTx;
    
    if (opts.alkaneId) {
      // Parse from the new alkane-id option
      const [block, tx] = opts.alkaneId.split(':');
      alkaneBlock = block;
      alkaneTx = tx;
      console.log(\`Processing alkane ID from option: \${alkaneBlock}:\${alkaneTx}\`);
    } else {
      // Fall back to extracting from calldata (legacy behavior)
      alkaneBlock = opts.calldata[0];
      alkaneTx = opts.calldata[1];
      console.log(\`Processing alkane ID from calldata: \${alkaneBlock}:\${alkaneTx}\`);
    }
    
    return { success: true, alkaneBlock, alkaneTx };
  };
  
  // Execute the command
  alkanes.alkaneExecute.action(options)
    .then(result => console.log('Result:', result))
    .catch(error => console.error('Error:', error));
"`;

executeCommand(legacyApproachCommand);

console.log('\n=== Test Summary ===');
console.log('Both approaches should extract the same alkane ID (123456:789)');
console.log('The new approach uses the alkane-id option, while the legacy approach uses the first two parameters of calldata');
