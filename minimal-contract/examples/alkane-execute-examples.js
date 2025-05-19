/**
 * Examples demonstrating how to use the alkaneExecute command with the new alkane-id option
 * 
 * This file provides examples of both the new approach (using the alkane-id option)
 * and the legacy approach (using the alkane ID in calldata).
 */

// Import the OYL SDK
const { execSync } = require('child_process');

/**
 * Execute a command and log the output
 * @param {string} command - The command to execute
 */
function executeCommand(command) {
  console.log(`\nExecuting: ${command}`);
  try {
    const output = execSync(command, { encoding: 'utf8' });
    console.log('Output:');
    console.log(output);
  } catch (error) {
    console.error('Error:');
    console.error(error.stdout || error.message);
  }
}

// Example alkane ID and calldata
const alkaneBlock = '123456';
const alkaneTx = '789';
const alkaneId = `${alkaneBlock}:${alkaneTx}`;
const opCode = '1'; // Example operation code
const param1 = '100';
const param2 = '200';

console.log('=== OYL SDK Alkane Execute Examples ===');

// Example 1: Using the new alkane-id option
console.log('\n=== Example 1: Using the new alkane-id option ===');
const newApproachCommand = `oyl alkane execute -id ${alkaneId} -data ${opCode},${param1},${param2} -p regtest`;
console.log('This approach explicitly specifies the alkane ID using the new -id option:');
console.log(`- Alkane ID: ${alkaneId}`);
console.log(`- Operation: ${opCode}`);
console.log(`- Parameters: ${param1}, ${param2}`);
// Uncomment to execute:
// executeCommand(newApproachCommand);

// Example 2: Using the legacy approach (alkane ID in calldata)
console.log('\n=== Example 2: Using the legacy approach (alkane ID in calldata) ===');
const legacyApproachCommand = `oyl alkane execute -data ${alkaneBlock},${alkaneTx},${opCode},${param1},${param2} -p regtest`;
console.log('This approach includes the alkane ID as the first two parameters in calldata:');
console.log(`- Alkane ID: ${alkaneBlock}:${alkaneTx} (included in calldata)`);
console.log(`- Operation: ${opCode}`);
console.log(`- Parameters: ${param1}, ${param2}`);
// Uncomment to execute:
// executeCommand(legacyApproachCommand);

// Example 3: Using the new approach with edicts
console.log('\n=== Example 3: Using the new approach with edicts ===');
const edictAmount = '1000';
const edictOutput = '0';
const edict = `${alkaneBlock}:${alkaneTx}:${edictAmount}:${edictOutput}`;
const newApproachWithEdictsCommand = `oyl alkane execute -id ${alkaneId} -data ${opCode},${param1},${param2} -e ${edict} -p regtest`;
console.log('This approach uses the new -id option along with edicts:');
console.log(`- Alkane ID: ${alkaneId}`);
console.log(`- Operation: ${opCode}`);
console.log(`- Parameters: ${param1}, ${param2}`);
console.log(`- Edict: ${edict}`);
// Uncomment to execute:
// executeCommand(newApproachWithEdictsCommand);

console.log('\n=== Usage Notes ===');
console.log('1. The new -id option provides a more explicit way to specify the alkane ID');
console.log('2. The legacy approach (using alkane ID in calldata) is still supported for backward compatibility');
console.log('3. When using the new approach, the calldata only needs to include the operation code and parameters');
console.log('4. The alkane ID should be in the format "block:tx" (e.g., "123456:789")');

console.log('\n=== To Execute These Examples ===');
console.log('1. Uncomment the executeCommand() calls in this file');
console.log('2. Ensure you have the OYL SDK installed and configured');
console.log('3. Replace the example values with actual values for your use case');
console.log('4. Run the script with: node alkane-execute-examples.js');
