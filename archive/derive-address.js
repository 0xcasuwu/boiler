// Use require to load oyl-sdk modules which should already be installed
const mnemonic = process.env.MNEMONIC;

// Basic demo script to get address from mnemonic
console.log("Mnemonic:", mnemonic);
console.log("Script is running...");

// Try to use the Account module from oyl-sdk with proper path
try {
  const Account = require('./oyl-sdk/lib/account').default;
  const account = Account.fromMnemonic(mnemonic);
  console.log("Generated address:", account.address);
} catch (error) {
  console.error("Error using oyl-sdk modules:", error.message);
  console.error("Full error:", error);
}
