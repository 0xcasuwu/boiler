/**
 * YieldVault Wallet Info Script
 * 
 * This script retrieves information about the default wallet used by the OylNet SDK.
 */

// Import bitcoinjs-lib directly to access networks
const bitcoin = require('./oyl-sdk/node_modules/bitcoinjs-lib');
const fs = require('fs');
const path = require('path');

// Define the regtest network
const REGTEST = bitcoin.networks.regtest;

try {
  console.log("DEFAULT BITCOINJS-LIB NETWORKS:");
  console.log("Regtest pubKeyHash:", REGTEST.pubKeyHash.toString(16));
  console.log("Regtest scriptHash:", REGTEST.scriptHash.toString(16));
  console.log("Regtest bech32:", REGTEST.bech32);
  
  // Load the constants file from the OylNet SDK to see what addresses they're using
  const constantsPath = path.join(__dirname, 'oyl-sdk/lib/cli/constants.js');
  if (fs.existsSync(constantsPath)) {
    console.log("\nLooking at constants in OylNet SDK...");
    const constants = require('./oyl-sdk/lib/cli/constants');
    
    console.log("\nREGTEST_FAUCET from constants:");
    console.log(constants.REGTEST_FAUCET);
    
    console.log("\nTEST_WALLET from constants:");
    console.log(constants.TEST_WALLET);
  } else {
    console.log("Could not find constants file");
  }
  
  // Try to create an address using the same pattern as the SDK
  console.log("\nCreating test addresses for each format:");
  
  // Create a key pair
  const ECPair = require('./oyl-sdk/node_modules/ecpair');
  const crypto = require('crypto');
  
  // Create a deterministic private key for testing
  const privateKey = crypto.randomBytes(32);
  const keyPair = ECPair.ECPairFactory().fromPrivateKey(privateKey, { network: REGTEST });
  
  // Create P2PKH address (Legacy)
  const p2pkhAddress = bitcoin.payments.p2pkh({ 
    pubkey: keyPair.publicKey, 
    network: REGTEST 
  }).address;
  
  console.log("Legacy P2PKH address:", p2pkhAddress);
  
  // Create P2WPKH address (SegWit)
  const p2wpkhAddress = bitcoin.payments.p2wpkh({ 
    pubkey: keyPair.publicKey, 
    network: REGTEST 
  }).address;
  
  console.log("SegWit P2WPKH address:", p2wpkhAddress);
  
  // Create P2SH-P2WPKH address (Nested SegWit)
  const p2sh_p2wpkh = bitcoin.payments.p2sh({
    redeem: bitcoin.payments.p2wpkh({ pubkey: keyPair.publicKey, network: REGTEST }),
    network: REGTEST
  });
  
  console.log("Nested SegWit P2SH-P2WPKH address:", p2sh_p2wpkh.address);
} catch (error) {
  console.error("Error:", error.message);
}
