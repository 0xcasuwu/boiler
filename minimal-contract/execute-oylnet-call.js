/**
 * Script to execute a call on oylnet using the new alkane-id option
 */

const { execSync } = require('child_process');

// Command parameters
const alkaneId = '2:0';
const calldata = '2,699,1';
const provider = 'oylnet';

// Build the command
const command = `oyl alkane execute -id ${alkaneId} -data ${calldata} -p ${provider}`;

console.log(`Executing command: ${command}`);

try {
  // Execute the command
  const output = execSync(command, { encoding: 'utf8' });
  console.log('Command executed successfully!');
  console.log('Output:');
  console.log(output);
} catch (error) {
  console.error('Error executing command:');
  console.error(error.message);
  if (error.stdout) console.error('stdout:', error.stdout);
  if (error.stderr) console.error('stderr:', error.stderr);
}
