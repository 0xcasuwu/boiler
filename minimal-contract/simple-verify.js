/**
 * Simple verification script
 */

const fs = require('fs');
const alkane = require('./oyl-sdk/lib/cli/alkane');

// Check if the alkaneExecute command exists
if (!alkane.alkaneExecute) {
  fs.writeFileSync('verification-result.txt', 'ERROR: alkaneExecute command not found');
  process.exit(1);
}

// Get the command options
const options = alkane.alkaneExecute.options || [];

// Check if the alkane-id option exists
const alkaneIdOption = options.find(option => 
  option.long === '--alkane-id' || option.short === '-id'
);

// Write the result to a file
const result = alkaneIdOption 
  ? 'SUCCESS: alkane-id option found in alkaneExecute command' 
  : 'FAILURE: alkane-id option NOT found in alkaneExecute command';

fs.writeFileSync('verification-result.txt', result);

// Also print to console
console.log(result);
