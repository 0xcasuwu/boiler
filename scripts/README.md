# Enhanced Deployment and Verification Scripts

This directory contains enhanced deployment and verification scripts that use multi-vout tracing to find actual contract content instead of empty responses.

## Problem Solved

The original issue was that contract tracing with `vout: 0` often returned empty arrays `[]`, but testing `vout: 3` revealed rich contract data including storage entries like `/totalsupply`, `/cap`, `/initialized`, and other contract state information.

These enhanced scripts automatically test **vouts 3, 4, and 5** to find the one with actual contract content.

## Scripts Overview

### 1. `deploy_with_enhanced_tracing.sh`
**Purpose**: Deploy the complete MasterChef staking architecture with enhanced contract verification

**Features**:
- Deploys Free-mint, Position-token, and Vault-factory contracts
- Tests vouts 3-5 for each deployed contract to find actual content
- Shows detailed contract information (storage entries, total supply, etc.)
- Provides comprehensive deployment summary with working vout numbers
- Automatically uses the fixed OYL SDK RPC client

**Usage**:
```bash
# Deploy with namespace
./deploy_with_enhanced_tracing.sh <namespace>

# Example
./deploy_with_enhanced_tracing.sh 100
```

### 2. `verify_with_enhanced_tracing.sh`
**Purpose**: Verify existing deployed contracts using multi-vout tracing

**Features**:
- Tests vouts 3-5 for each contract to find actual content
- Shows detailed contract state information
- Supports multiple usage modes
- Provides verification success rates and summaries

**Usage**:
```bash
# Interactive mode (prompts for TXIDs)
./verify_with_enhanced_tracing.sh

# Command line mode (provide TXIDs as arguments)
./verify_with_enhanced_tracing.sh <txid1> <txid2> <txid3> <txid4> <txid5>

# Test mode (uses example TXIDs)
./verify_with_enhanced_tracing.sh test
```

## Key Improvements

### Multi-Vout Discovery
Instead of assuming `vout: 0`, these scripts:
1. **Test vouts 3, 4, 5** for each contract
2. **Detect actual content** vs empty responses
3. **Show which vout works** for future reference
4. **Extract rich contract data** like storage entries

### Enhanced Contract Analysis
The scripts automatically detect and display:
- **Storage entries count** and key names
- **Total supply** (`/totalsupply`)
- **Token cap** (`/cap`) 
- **Initialization status** (`/initialized`)
- **Minted amount** (`/minted`)
- **Contract data length**

### Comprehensive Reporting
Each script provides:
- **Success/failure rates** for verification
- **Working vout numbers** for each contract
- **Manual verification commands** for future use
- **Troubleshooting tips** when verification fails

## Example Output

When a contract is successfully traced, you'll see:
```
🔍 Enhanced verification for Free-mint Template
   TX ID: 56b0beb401569808c6a74a9b9be3c6a17fe2b3a34fff16f5c5ebf46730ed3b50
   Testing vouts 3, 4, 5 to find actual content...
   🔸 Testing vout 3...
   ✅ Found content in vout 3!
   📊 Content summary:
      • Vout: 3
      • Has storage entries: Yes
      • Storage entries count: 4
      • Data length: 2048 characters
      • Key storage entries:
        - /totalsupply
        - /cap
        - /initialized
        - /minted
      • 💰 Total Supply: 0xa0860100000000000000000000000000
      • 🎯 Cap: 0xa0860100000000000000000000000000
      • ✅ Initialized: 0x01
   🎉 Verification SUCCESS for Free-mint Template!
   📋 Use vout 3 for future traces of this contract
```

## Prerequisites

1. **OYL SDK**: The scripts automatically locate the OYL SDK directory
2. **Compiled contracts**: WASM files must be built before deployment
3. **oylnet network**: Scripts are configured for oylnet blockchain

## File Structure

```
boiler/
├── scripts/
│   ├── deploy_with_enhanced_tracing.sh    # Enhanced deployment
│   ├── verify_with_enhanced_tracing.sh    # Enhanced verification  
│   └── README.md                          # This documentation
├── alkanes/
│   └── free-mint/target/wasm32-unknown-unknown/release/free_mint.wasm
├── position-token/target/wasm32-unknown-unknown/release/position_token.wasm
└── vault-factory/target/wasm32-unknown-unknown/release/vault_factory.wasm
```

## Integration with Fixed OYL SDK

These scripts use the **fixed OYL SDK RPC client** that resolves the original buffer and parameter issues:
- ✅ **trace() method**: Now properly handles txid parameters
- ✅ **simulate() method**: Now properly handles target.block parameters  
- ✅ **Array parameter handling**: CLI properly extracts objects from arrays
- ✅ **Enhanced error messages**: Clear feedback when parameters are missing

## Manual Verification Commands

If you need to manually verify a specific contract:

```bash
# Navigate to OYL SDK directory
cd ../oyl-sdk

# Test different vouts to find content
node bin/oyl.js provider alkanes -method "trace" -params '[{"txid": "YOUR_TXID", "vout": 3}]' -p oylnet
node bin/oyl.js provider alkanes -method "trace" -params '[{"txid": "YOUR_TXID", "vout": 4}]' -p oylnet
node bin/oyl.js provider alkanes -method "trace" -params '[{"txid": "YOUR_TXID", "vout": 5}]' -p oylnet
```

## Troubleshooting

### If verification fails:
1. **Check TXID**: Ensure the transaction ID is correct and complete
2. **Verify deployment**: Confirm the contract was deployed successfully
3. **Test manually**: Try different vout values (0-5) manually
4. **Check blockchain sync**: Ensure the blockchain is synchronized

### If deployment fails:
1. **Check WASM files**: Ensure contracts are compiled successfully
2. **Verify OYL SDK**: Confirm the SDK directory is accessible
3. **Check network**: Ensure oylnet blockchain is accessible
4. **Review logs**: Check the detailed output for specific error messages

## Future Enhancements

These scripts can be extended to:
- **Auto-extract template IDs** from trace data instead of using hardcoded values
- **Support additional networks** beyond oylnet
- **Add contract interaction testing** (deposit, withdraw, rewards)
- **Generate deployment reports** in JSON/CSV format
- **Integrate with CI/CD pipelines** for automated testing

## Related Files

- `../reference/oyl_sdk_bugfix_summary.md` - Details of the OYL SDK fixes
- `../reference/test_rpc_fixes.sh` - Test script for SDK fixes
- Original deployment scripts in `../reference/` directory
