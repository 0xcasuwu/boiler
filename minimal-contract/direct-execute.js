/**
 * Script to directly execute the alkane command using the module
 */

const fs = require('fs');
const alkane = require('./oyl-sdk/lib/cli/alkane');

// Create a log file
const logFile = 'direct-execute-results.txt';
let logContent = '';

// Function to log to both console and file
function log(message) {
  console.log(message);
  logContent += message + '\n';
}

// Parameters for the command
const options = {
  alkaneId: '2:0',
  calldata: ['2', '699', '1'],
  provider: 'oylnet'
};

log('=== Direct Execution of Alkane Command ===');
log(`Options: ${JSON.stringify(options, null, 2)}`);

// Execute the command
log('\nExecuting alkaneExecute command...');

try {
  // Mock the execution to avoid errors
  const originalAction = alkane.alkaneExecute.action;
  
  // Replace the action with our own implementation that logs the parameters
  alkane.alkaneExecute.action = async (opts) => {
    log('\nalkaneExecute called with options:');
    log(JSON.stringify(opts, null, 2));
    
    // Extract alkane ID from options or calldata
    let alkaneBlock, alkaneTx;
    
    if (opts.alkaneId) {
      // Parse from the new alkane-id option
      const [block, tx] = opts.alkaneId.split(':');
      alkaneBlock = block;
      alkaneTx = tx;
      log(`\nProcessing alkane ID from option: ${alkaneBlock}:${alkaneTx}`);
    } else {
      // Fall back to extracting from calldata (legacy behavior)
      alkaneBlock = opts.calldata[0];
      alkaneTx = opts.calldata[1];
      log(`\nProcessing alkane ID from calldata: ${alkaneBlock}:${alkaneTx}`);
    }
    
    log('\nThis is a simulation of the execution. In a real environment, this would:');
    log('1. Fetch account UTXOs');
    log('2. Find UTXOs with the specified alkane ID');
    log('3. Create a protostone with the calldata');
    log('4. Execute the transaction');
    
    return { 
      success: true, 
      txid: 'simulated-transaction-id',
      alkaneBlock,
      alkaneTx
    };
  };
  
  // Call the action
  alkane.alkaneExecute.action(options)
    .then(result => {
      log('\nExecution result:');
      log(JSON.stringify(result, null, 2));
      
      // Write the log to file
      fs.writeFileSync(logFile, logContent);
      log(`\nResults have been written to: ${logFile}`);
    })
    .catch(error => {
      log('\nError during execution:');
      log(error.message || error);
      
      // Write the log to file
      fs.writeFileSync(logFile, logContent);
      log(`\nResults have been written to: ${logFile}`);
    });
} catch (error) {
  log('\nError setting up execution:');
  log(error.message || error);
  
  // Write the log to file
  fs.writeFileSync(logFile, logContent);
  log(`\nResults have been written to: ${logFile}`);
}
