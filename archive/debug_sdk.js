// Simple diagnostic script to check oyl-sdk structure

const fs = require('fs');

console.log("Checking oyl-sdk structure...");

try {
  // Check what's in lib directory
  console.log("\n=== Contents of ./oyl-sdk/lib ===");
  const libFiles = fs.readdirSync('./oyl-sdk/lib');
  console.log(libFiles);
  
  // Check account module
  console.log("\n=== Account Module ===");
  const accountModule = require('./oyl-sdk/lib/account');
  console.log("Exports:", Object.keys(accountModule));
  
  // Check if src directory exists
  console.log("\n=== Contents of ./oyl-sdk/src ===");
  if (fs.existsSync('./oyl-sdk/src')) {
    console.log(fs.readdirSync('./oyl-sdk/src'));
  } else {
    console.log("src directory not found");
  }
  
  console.log("\nDiagnostics complete");
} catch (error) {
  console.error("Error:", error);
}
