/**
 * Simple verification script for the alkane-id extension
 */

// Import the alkane module
const alkane = require('./oyl-sdk/lib/cli/alkane');

// Check if the alkaneExecute command has the alkane-id option
const options = alkane.alkaneExecute.options;
const hasAlkaneIdOption = options.some(option => 
  option.long === '--alkane-id' || option.short === '-id'
);

console.log('=== Alkane ID Extension Verification ===');
console.log(`alkaneExecute command has alkane-id option: ${hasAlkaneIdOption ? 'YES' : 'NO'}`);

if (hasAlkaneIdOption) {
  console.log('\nThe alkane-id extension has been successfully implemented!');
  console.log('You can now use the -id, --alkane-id option with the alkaneExecute command.');
  console.log('\nExample usage:');
  console.log('  oyl alkane execute -id 123456:789 -data 1,2,3 -p regtest');
} else {
  console.log('\nThe alkane-id extension has NOT been implemented correctly.');
  console.log('Please check the implementation in oyl-sdk/lib/cli/alkane.js');
}

// Print out all options for reference
console.log('\nAll options for alkaneExecute command:');
options.forEach(option => {
  const shortFlag = option.short ? `${option.short}, ` : '';
  const description = option.description || 'No description';
  console.log(`  ${shortFlag}${option.long}: ${description}`);
});
