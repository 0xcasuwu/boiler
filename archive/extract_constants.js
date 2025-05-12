
    const path = require('path');
    const fs = require('fs');
    
    try {
      // Read the constants file directly to extract the TEST_WALLET address
      const constantsPath = path.join(__dirname, 'oyl-sdk/lib/cli/constants.js');
      const constantsContent = fs.readFileSync(constantsPath, 'utf8');
      
      // Extract the default regtest address using regex
      const addressMatch = constantsContent.match(/nativeSegwit:\s*{\s*address:\s*['"]([^'"]+)['"]/);
      const faucetMatch = constantsContent.match(/REGTEST_FAUCET\s*=\s*{[^}]*address:\s*['"]([^'"]+)['"]/);
      
      // Create a output file with the extracted addresses
      const output = {
        testWalletAddress: addressMatch ? addressMatch[1] : null,
        faucetAddress: faucetMatch ? faucetMatch[1] : null
      };
      
      fs.writeFileSync(path.join(__dirname, 'sdk_constants.json'), JSON.stringify(output, null, 2));
      console.log('SDK constants extracted successfully');
      console.log(output);
      process.exit(0);
    } catch (error) {
      console.error('Error extracting constants:', error.message);
      process.exit(1);
    }
    