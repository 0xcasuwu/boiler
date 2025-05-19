/**
 * Script to run the alkane execute command with the legacy approach and capture the output
 */

const { execSync } = require('child_process');
const fs = require('fs');

// Command to execute (legacy approach with alkane ID in calldata)
const command = './oyl-sdk/bin/oyl.js alkane execute -data 2,0,2,699,1 -p oylnet';

// File to write the output to
const outputFile = 'legacy-command-output.txt';

try {
  // Execute the command and capture the output
  console.log(`Executing command: ${command}`);
  const output = execSync(command, { encoding: 'utf8', stdio: 'pipe' });
  
  // Write the output to a file
  fs.writeFileSync(outputFile, output);
  
  console.log(`Command executed successfully. Output written to ${outputFile}`);
} catch (error) {
  // Write the error to a file
  const errorOutput = `
Error executing command: ${command}
Error message: ${error.message}
${error.stdout ? `\nStandard output:\n${error.stdout}` : ''}
${error.stderr ? `\nStandard error:\n${error.stderr}` : ''}
`;
  
  fs.writeFileSync(outputFile, errorOutput);
  
  console.error(`Error executing command. Error details written to ${outputFile}`);
}
