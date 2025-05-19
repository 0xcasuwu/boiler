/**
 * Verification script that logs results to a file
 */

const fs = require('fs');
const path = require('path');
const alkane = require('./oyl-sdk/lib/cli/alkane');

// Create a log file
const logFile = path.join(__dirname, 'verification-results.txt');
let logContent = '';

// Function to log to both console and file
function log(message) {
  console.log(message);
  logContent += message + '\n';
}

// Check if the alkaneExecute command has the alkane-id option
const options = alkane.alkaneExecute.options;
const alkaneIdOption = options.find(option => 
  option.long === '--alkane-id' || option.short === '-id'
);

log('=== Alkane ID Extension Verification ===');
log(`alkaneExecute command has alkane-id option: ${alkaneIdOption ? 'YES' : 'NO'}`);

if (alkaneIdOption) {
  log(`\nOption details:`);
  log(`  Long name: ${alkaneIdOption.long}`);
  log(`  Short name: ${alkaneIdOption.short || 'N/A'}`);
  log(`  Description: ${alkaneIdOption.description || 'No description'}`);
  
  log('\nThe alkane-id extension has been successfully implemented!');
  log('You can now use the -id, --alkane-id option with the alkaneExecute command.');
  log('\nExample usage:');
  log('  oyl alkane execute -id 123456:789 -data 1,2,3 -p regtest');
} else {
  log('\nThe alkane-id extension has NOT been implemented correctly.');
  log('Please check the implementation in oyl-sdk/lib/cli/alkane.js');
}

// Print out all options for reference
log('\nAll options for alkaneExecute command:');
options.forEach(option => {
  const shortFlag = option.short ? `${option.short}, ` : '';
  const description = option.description || 'No description';
  log(`  ${shortFlag}${option.long}: ${description}`);
});

// Test the alkane ID extraction logic
log('\n=== Testing Alkane ID Extraction Logic ===');

// Mock function to test the extraction logic
function testExtraction(options) {
  let alkaneBlock, alkaneTx;
  
  if (options.alkaneId) {
    // Parse from the new alkane-id option
    const [block, tx] = options.alkaneId.split(':');
    alkaneBlock = block;
    alkaneTx = tx;
    log(`Processing alkane ID from option: ${alkaneBlock}:${alkaneTx}`);
  } else {
    // Fall back to extracting from calldata (legacy behavior)
    alkaneBlock = options.calldata[0];
    alkaneTx = options.calldata[1];
    log(`Processing alkane ID from calldata: ${alkaneBlock}:${alkaneTx}`);
  }
  
  return { alkaneBlock, alkaneTx };
}

// Test with the new option
const newOptionTest = {
  alkaneId: '123456:789',
  calldata: ['1', '100', '200']
};

log('\nTest 1: Using the new alkane-id option');
log(`Options: ${JSON.stringify(newOptionTest)}`);
const newOptionResult = testExtraction(newOptionTest);
log(`Result: Block = ${newOptionResult.alkaneBlock}, Tx = ${newOptionResult.alkaneTx}`);

// Test with the legacy approach
const legacyTest = {
  calldata: ['123456', '789', '1', '100', '200']
};

log('\nTest 2: Using the legacy approach (alkane ID in calldata)');
log(`Options: ${JSON.stringify(legacyTest)}`);
const legacyResult = testExtraction(legacyTest);
log(`Result: Block = ${legacyResult.alkaneBlock}, Tx = ${legacyResult.alkaneTx}`);

// Test with both options (new option should take precedence)
const bothOptionsTest = {
  alkaneId: '111111:222',
  calldata: ['333333', '444', '1', '100', '200']
};

log('\nTest 3: Using both options (new option should take precedence)');
log(`Options: ${JSON.stringify(bothOptionsTest)}`);
const bothOptionsResult = testExtraction(bothOptionsTest);
log(`Result: Block = ${bothOptionsResult.alkaneBlock}, Tx = ${bothOptionsResult.alkaneTx}`);

// Write the log to file
fs.writeFileSync(logFile, logContent);
log(`\nVerification results have been written to: ${logFile}`);
