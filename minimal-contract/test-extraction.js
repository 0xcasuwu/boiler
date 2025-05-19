/**
 * Test script for the alkane ID extraction logic
 */

const fs = require('fs');

// Mock the extraction logic from alkane.js
function extractAlkaneId(options) {
  let alkaneBlock, alkaneTx;
  
  if (options.alkaneId) {
    // Parse from the new alkane-id option
    const [block, tx] = options.alkaneId.split(':');
    alkaneBlock = block;
    alkaneTx = tx;
    return { 
      source: 'option',
      alkaneBlock, 
      alkaneTx 
    };
  } else {
    // Fall back to extracting from calldata (legacy behavior)
    alkaneBlock = options.calldata[0];
    alkaneTx = options.calldata[1];
    return { 
      source: 'calldata',
      alkaneBlock, 
      alkaneTx 
    };
  }
}

// Test cases
const testCases = [
  {
    name: 'Test 1: Using the new alkane-id option',
    options: {
      alkaneId: '123456:789',
      calldata: ['1', '100', '200']
    },
    expectedSource: 'option',
    expectedBlock: '123456',
    expectedTx: '789'
  },
  {
    name: 'Test 2: Using the legacy approach (alkane ID in calldata)',
    options: {
      calldata: ['123456', '789', '1', '100', '200']
    },
    expectedSource: 'calldata',
    expectedBlock: '123456',
    expectedTx: '789'
  },
  {
    name: 'Test 3: Using both options (new option should take precedence)',
    options: {
      alkaneId: '111111:222',
      calldata: ['333333', '444', '1', '100', '200']
    },
    expectedSource: 'option',
    expectedBlock: '111111',
    expectedTx: '222'
  }
];

// Run the tests
let results = '';
let allPassed = true;

testCases.forEach(testCase => {
  const { name, options, expectedSource, expectedBlock, expectedTx } = testCase;
  const result = extractAlkaneId(options);
  
  const sourcePass = result.source === expectedSource;
  const blockPass = result.alkaneBlock === expectedBlock;
  const txPass = result.alkaneTx === expectedTx;
  const passed = sourcePass && blockPass && txPass;
  
  if (!passed) allPassed = false;
  
  results += `${name}\n`;
  results += `  Options: ${JSON.stringify(options)}\n`;
  results += `  Expected: source=${expectedSource}, block=${expectedBlock}, tx=${expectedTx}\n`;
  results += `  Actual: source=${result.source}, block=${result.alkaneBlock}, tx=${result.alkaneTx}\n`;
  results += `  Result: ${passed ? 'PASS' : 'FAIL'}\n`;
  results += `    Source: ${sourcePass ? 'PASS' : 'FAIL'}\n`;
  results += `    Block: ${blockPass ? 'PASS' : 'FAIL'}\n`;
  results += `    Tx: ${txPass ? 'PASS' : 'FAIL'}\n\n`;
});

// Add overall result
results += `Overall Result: ${allPassed ? 'ALL TESTS PASSED' : 'SOME TESTS FAILED'}\n`;

// Write results to file
fs.writeFileSync('extraction-test-results.txt', results);

// Also print to console
console.log(results);
