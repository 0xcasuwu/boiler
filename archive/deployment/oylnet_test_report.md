# OylNet Alkane ID Verification Test Report

## Summary

- **Total Tests**: 9
- **Passed**: 2
- **Failed**: 7
- **Contract ID**: 7dbd26587f4d058577afa7e180fe1bfbe4941e6b5459e783d14e336c18238f0f
- **Date**: 2025-05-13T10:20:53.884Z

## Test Results

### 1. Deposit with correct alkane ID

- **Status**: ✅ Passed
- **Duration**: 80922ms
- **Start State**: Total Assets: 0, Total Supply: 0
- **End State**: Total Assets: 0, Total Supply: 0
- **Result**: {
  "previewResult": {
    "success": true,
    "result": "Address format validation patch loaded\n{\n  txId: 'a99540a802eb492571a96829dbbee4abc7ed48e0e1b89290d393713a20ef17af',\n  rawTx: '02000000000101548f4a1515d6821902be9906c76f1dd801f162d67f58091791942fe609757f0d0200000000ffffffff032202000000000000225120a60869f0dbcf1dc659c9cecbaf8050135ea9e8cdc487053f1dc6880949dc684c0000000000000000126a5d0fff7f818cec82d08bc0a8908f82fd03ee03813b00000000160014c0cebcd6c3d3ca8c75dc5ec62ebe55330ef910e202483045022100da68ece6bddd961b373ea247c405256d3a2830b62eaf7604b04ade3472610860022050e6044febbac72d446d49d599c4ae74db524c130289435e393cdb9f5550fe1501210330d54fd0dd420a6e5f8d3624f5f3482cae350f79d5f0753bf5beef9c2d91af3c00000000',\n  size: 180,\n  weight: 718,\n  fee: 360,\n  satsPerVByte: '2.01'\n}"
  },
  "depositResult": {
    "success": true,
    "result": "Address format validation patch loaded\n{\n  txId: 'c7b392f97e8875635c53bad270fd179115b574843f0474ac69d27f83d7a29830',\n  rawTx: '02000000000101af17ef203a7193d39092b8e1e048edc7abe4bedb2968a9712549eb02a84095a90200000000ffffffff032202000000000000225120a60869f0dbcf1dc659c9cecbaf8050135ea9e8cdc487053f1dc6880949dc684c00000000000000001d6a5d1aff7f8196ec82d08bc0a88ab1ebf59992a0fd03ff7f808484c07e4e00813b00000000160014c0cebcd6c3d3ca8c75dc5ec62ebe55330ef910e20247304402203d6f391fa0baa715d94a47b553369285d742b1f319b1dc111eadfd19a4b2787002205a9aef3835cef91dfc3203cc68b412b84d92ea43a4f3804ce63c305ea48104d501210330d54fd0dd420a6e5f8d3624f5f3482cae350f79d5f0753bf5beef9c2d91af3c00000000',\n  size: 191,\n  weight: 761,\n  fee: 382,\n  satsPerVByte: '2.01'\n}"
  }
}

### 2. Deposit with incorrect alkane ID

- **Status**: ❌ Failed
- **Error**: Deposit with incorrect alkane ID should fail

### 3. Deposit with multiple alkane IDs

- **Status**: ❌ Failed
- **Error**: Deposit failed: Command failed: NODE_OPTIONS=--require=/workspaces/boiler/oyl-sdk/lib/shared/load_patch.js oyl alkane execute -data "10,61705,1,1000" --edicts "3:1:500:0,2:1:1000:0,4:1:750:0" --provider oylnet
JSON-RPC Error: { code: -5, message: 'Transaction not in mempool' }
Request Error: Error: [object Object]
    at SandshrewBitcoinClient._call (/workspaces/boiler/oyl-sdk/lib/rpclient/sandshrew.js:32:23)
    at process.processTicksAndRejections (node:internal/process/task_queues:95:5)
    at async Provider.pushPsbt (/workspaces/boiler/oyl-sdk/lib/provider/provider.js:75:29)
    at async Object.execute (/workspaces/boiler/oyl-sdk/lib/alkanes/alkanes.js:517:24)
    at async /workspaces/boiler/oyl-sdk/lib/cli/alkane.js:251:17
    at async AlkanesCommand.<anonymous> (/workspaces/boiler/oyl-sdk/lib/cli/alkane.js:37:20)
node:internal/process/promises:391
    triggerUncaughtException(err, true /* fromPromise */);
    ^

Error: [object Object]
    at SandshrewBitcoinClient._call (/workspaces/boiler/oyl-sdk/lib/rpclient/sandshrew.js:32:23)
    at process.processTicksAndRejections (node:internal/process/task_queues:95:5)
    at async Provider.pushPsbt (/workspaces/boiler/oyl-sdk/lib/provider/provider.js:75:29)
    at async Object.execute (/workspaces/boiler/oyl-sdk/lib/alkanes/alkanes.js:517:24)
    at async /workspaces/boiler/oyl-sdk/lib/cli/alkane.js:251:17
    at async AlkanesCommand.<anonymous> (/workspaces/boiler/oyl-sdk/lib/cli/alkane.js:37:20)

Node.js v20.19.0


### 4. Deposit with insufficient assets

- **Status**: ❌ Failed
- **Error**: Deposit with insufficient assets should fail

### 5. Redeem with correct alkane ID

- **Status**: ❌ Failed
- **Error**: Redeem failed: Command failed: NODE_OPTIONS=--require=/workspaces/boiler/oyl-sdk/lib/shared/load_patch.js oyl alkane execute -data "13,972329,1,1,500" --edicts "7dbd26587f4d058577afa7e180fe1bfbe4941e6b5459e783d14e336c18238f0f:500:0" --provider oylnet
/workspaces/boiler/oyl-sdk/node_modules/@magiceden-oss/runestone-lib/dist/src/integer/u128.js:16
    const bigNum = typeof num == 'bigint' ? num : BigInt(num);
                                                  ^

SyntaxError: Cannot convert 7dbd26587f4d058577afa7e180fe1bfbe4941e6b5459e783d14e336c18238f0f to a BigInt
    at BigInt (<anonymous>)
    at u128 (/workspaces/boiler/oyl-sdk/node_modules/@magiceden-oss/runestone-lib/dist/src/integer/u128.js:16:51)
    at /workspaces/boiler/oyl-sdk/lib/cli/alkane.js:235:74
    at Array.map (<anonymous>)
    at /workspaces/boiler/oyl-sdk/lib/cli/alkane.js:230:35
    at process.processTicksAndRejections (node:internal/process/task_queues:95:5)
    at async AlkanesCommand.<anonymous> (/workspaces/boiler/oyl-sdk/lib/cli/alkane.js:37:20)

Node.js v20.19.0


### 6. Redeem with incorrect alkane ID

- **Status**: ❌ Failed
- **Error**: Redeem with incorrect alkane ID should fail

### 7. Redeem with insufficient shares

- **Status**: ✅ Passed
- **Duration**: 55237ms
- **Start State**: Total Assets: 0, Total Supply: 0
- **End State**: Total Assets: 0, Total Supply: 0
- **Result**: {
  "redeemResult": {
    "success": false,
    "error": "Command failed: NODE_OPTIONS=--require=/workspaces/boiler/oyl-sdk/lib/shared/load_patch.js oyl alkane execute -data \"13,94753,1,1,1000\" --edicts \"7dbd26587f4d058577afa7e180fe1bfbe4941e6b5459e783d14e336c18238f0f:500:0\" --provider oylnet\n/workspaces/boiler/oyl-sdk/node_modules/@magiceden-oss/runestone-lib/dist/src/integer/u128.js:16\n    const bigNum = typeof num == 'bigint' ? num : BigInt(num);\n                                                  ^\n\nSyntaxError: Cannot convert 7dbd26587f4d058577afa7e180fe1bfbe4941e6b5459e783d14e336c18238f0f to a BigInt\n    at BigInt (<anonymous>)\n    at u128 (/workspaces/boiler/oyl-sdk/node_modules/@magiceden-oss/runestone-lib/dist/src/integer/u128.js:16:51)\n    at /workspaces/boiler/oyl-sdk/lib/cli/alkane.js:235:74\n    at Array.map (<anonymous>)\n    at /workspaces/boiler/oyl-sdk/lib/cli/alkane.js:230:35\n    at process.processTicksAndRejections (node:internal/process/task_queues:95:5)\n    at async AlkanesCommand.<anonymous> (/workspaces/boiler/oyl-sdk/lib/cli/alkane.js:37:20)\n\nNode.js v20.19.0\n"
  }
}

### 8. Redeem with multiple alkane IDs

- **Status**: ❌ Failed
- **Error**: Redeem failed: Command failed: NODE_OPTIONS=--require=/workspaces/boiler/oyl-sdk/lib/shared/load_patch.js oyl alkane execute -data "13,837808,1,1,500" --edicts "5:1:250:0,7dbd26587f4d058577afa7e180fe1bfbe4941e6b5459e783d14e336c18238f0f:500:0,6:1:300:0" --provider oylnet
/workspaces/boiler/oyl-sdk/node_modules/@magiceden-oss/runestone-lib/dist/src/integer/u128.js:16
    const bigNum = typeof num == 'bigint' ? num : BigInt(num);
                                                  ^

SyntaxError: Cannot convert 7dbd26587f4d058577afa7e180fe1bfbe4941e6b5459e783d14e336c18238f0f to a BigInt
    at BigInt (<anonymous>)
    at u128 (/workspaces/boiler/oyl-sdk/node_modules/@magiceden-oss/runestone-lib/dist/src/integer/u128.js:16:51)
    at /workspaces/boiler/oyl-sdk/lib/cli/alkane.js:235:74
    at Array.map (<anonymous>)
    at /workspaces/boiler/oyl-sdk/lib/cli/alkane.js:230:35
    at process.processTicksAndRejections (node:internal/process/task_queues:95:5)
    at async AlkanesCommand.<anonymous> (/workspaces/boiler/oyl-sdk/lib/cli/alkane.js:37:20)

Node.js v20.19.0


### 9. No transaction context

- **Status**: ❌ Failed
- **Error**: Deposit with no transaction context should fail

