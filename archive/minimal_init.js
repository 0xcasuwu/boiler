/**
 * Minimal YieldVault Initialization Script
 * 
 * This script focuses solely on initializing the contract with
 * the correct parameter format as expected by the OylNet CLI.
 */

const { execSync } = require('child_process');

// Contract ID from successful deployment
const CONTRACT_ID = '7dbd26587f4d058577afa7e180fe1bfbe4941e6b5459e783d14e336c18238f0f';

console.log('=== Minimal YieldVault Initialization ===');

// 1. Generate blocks to make sure everything is settled
try {
  console.log('\nGenerating blocks to ensure chain activity...');
  execSync('NODE_OPTIONS=--require=./oyl-sdk/lib/shared/load_patch.js oyl regtest genBlocks -p oylnet -c 20', { stdio: 'inherit' });
} catch (error) {
  console.log('Block generation failed but continuing');
}

// 2. Try checking the contract state before initialization
console.log('\nChecking contract state before initialization:');
try {
  // Try the simplest command format possible to check opcode 100 (getName)
  const command = `NODE_OPTIONS=--require=./oyl-sdk/lib/shared/load_patch.js oyl alkane execute -data "100" --provider oylnet`;
  console.log(`$ ${command}`);
  const result = execSync(command, { encoding: 'utf8', stdio: 'inherit' });
  console.log('Result:', result);
} catch (error) {
  console.log('Failed to check contract state:', error.message);
}

// 3. Try the initialize operation with the most basic format
console.log('\nAttempting to initialize contract with most basic format:');
try {
  // Try numeric-only initialization opcode
  const command = `NODE_OPTIONS=--require=./oyl-sdk/lib/shared/load_patch.js oyl alkane execute -data "0" --provider oylnet`;
  console.log(`$ ${command}`);
  execSync(command, { encoding: 'utf8', stdio: 'inherit' });
  console.log('Basic initialization command executed');
} catch (error) {
  console.log('Basic initialization failed:', error.message);
}

// 4. Try the initialize operation with minimal parameter approach
console.log('\nAttempting to initialize contract with minimal parameters:');
try {
  const command = `NODE_OPTIONS=--require=./oyl-sdk/lib/shared/load_patch.js oyl alkane execute -data "0,YieldVault,YVT,Bitcoin,BTC,8" --provider oylnet`;
  console.log(`$ ${command}`);
  execSync(command, { encoding: 'utf8', stdio: 'inherit' });
  console.log('Initialization with string parameters executed');
} catch (error) {
  console.log('Initialization with string parameters failed:', error.message);
}

// 5. Generate blocks to confirm initialization
try {
  console.log('\nGenerating blocks to confirm any state changes...');
  execSync('NODE_OPTIONS=--require=./oyl-sdk/lib/shared/load_patch.js oyl regtest genBlocks -p oylnet -c 10', { stdio: 'inherit' });
} catch (error) {
  console.log('Block generation failed but continuing');
}

// 6. Check if initialization worked
console.log('\nChecking if initialization was successful:');
try {
  // Check getName (opcode 100)
  const command = `NODE_OPTIONS=--require=./oyl-sdk/lib/shared/load_patch.js oyl alkane execute -data "100" --provider oylnet`;
  console.log(`$ ${command}`);
  const result = execSync(command, { encoding: 'utf8' });
  console.log('Contract name:', result);
  
  if (result.includes('YieldVault')) {
    console.log('\n✅ Contract initialization successful!');
  } else {
    console.log('\n⚠️ Contract initialization may not have been successful');
  }
} catch (error) {
  console.log('Failed to check contract state after initialization:', error.message);
}

console.log('\n=== Minimal Initialization Process Complete ===');
