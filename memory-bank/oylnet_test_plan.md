# OylNet Test Plan for Alkane ID Verification

This document outlines the plan for creating an OylNet deployed test that verifies the same 9 test behaviors from our alkane ID verification tests on testnet directly.

## Objective

Create a comprehensive test suite that verifies the alkane ID verification process in a real blockchain environment (OylNet). This will provide additional confidence in the token-based architecture's security model and ensure that it works correctly in a production-like setting.

## Test Cases

We will implement the following 9 test cases on OylNet:

1. **Deposit with correct alkane ID**: Verify successful deposit when correct asset ID is provided
2. **Deposit with incorrect alkane ID**: Verify failure when incorrect asset ID is provided
3. **Deposit with multiple alkane IDs**: Verify success when multiple IDs including the correct one are provided
4. **Deposit with insufficient assets**: Verify failure when not enough assets are provided
5. **Redeem with correct alkane ID**: Verify successful redemption when correct share token ID is provided
6. **Redeem with incorrect alkane ID**: Verify failure when incorrect share token ID is provided
7. **Redeem with insufficient shares**: Verify failure when not enough shares are provided
8. **Redeem with multiple alkane IDs**: Verify success when multiple IDs including the correct one are provided
9. **No transaction context**: Verify failure when no transaction context is provided

## Implementation Strategy

### 1. Contract Deployment

1. Build the WebAssembly contract:
   ```bash
   cargo build --target wasm32-unknown-unknown --release
   ```

2. Deploy the contract to OylNet:
   ```bash
   ./deployment/deploy_yield_vault.sh
   ```

3. Initialize the contract:
   ```bash
   NODE_OPTIONS=--require=/workspaces/boiler/oyl-sdk/lib/shared/load_patch.js oyl alkane execute -data "0" --provider oylnet
   ```

4. Generate blocks to confirm initialization:
   ```bash
   NODE_OPTIONS=--require=/workspaces/boiler/oyl-sdk/lib/shared/load_patch.js oyl regtest genBlocks -p oylnet -c 2
   ```

### 2. Test Script Development

Create a JavaScript test script (`oylnet_alkane_verification_tests.js`) that:

1. Connects to the OylNet network
2. Creates test transactions for each test case
3. Verifies the results of each transaction
4. Generates a comprehensive test report

The script will use the OYL SDK with the address format validation patch applied.

### 3. Test Implementation

#### Test Case 1: Deposit with correct alkane ID

```javascript
// Create a transaction with the correct asset ID
const tx = {
  opcode: 10, // Deposit opcode
  params: [0, 1, 1000], // tx_hash, caller, assets
  alkanes: [
    { id: ASSET_ID, value: 1000 } // Correct asset ID
  ]
};

// Execute the transaction
const result = await executeTransaction(tx);

// Verify the result
assert(result.success, "Deposit with correct alkane ID should succeed");
assert(result.totalAssets === 1000, "Total assets should be 1000");
assert(result.totalSupply === 1000, "Total supply should be 1000");
```

#### Test Case 2: Deposit with incorrect alkane ID

```javascript
// Create a transaction with an incorrect asset ID
const tx = {
  opcode: 10, // Deposit opcode
  params: [1, 1, 1000], // tx_hash, caller, assets
  alkanes: [
    { id: "wrong_asset_id", value: 1000 } // Incorrect asset ID
  ]
};

// Execute the transaction
const result = await executeTransaction(tx);

// Verify the result
assert(!result.success, "Deposit with incorrect alkane ID should fail");
assert(result.error.includes("Asset verification error"), "Error should mention asset verification");
```

#### Test Case 3: Deposit with multiple alkane IDs

```javascript
// Create a transaction with multiple asset IDs including the correct one
const tx = {
  opcode: 10, // Deposit opcode
  params: [2, 1, 1000], // tx_hash, caller, assets
  alkanes: [
    { id: "wrong_asset_id_1", value: 500 },
    { id: ASSET_ID, value: 1000 }, // Correct asset ID
    { id: "wrong_asset_id_2", value: 750 }
  ]
};

// Execute the transaction
const result = await executeTransaction(tx);

// Verify the result
assert(result.success, "Deposit with multiple alkane IDs should succeed");
assert(result.totalAssets === 2000, "Total assets should be 2000");
assert(result.totalSupply === 2000, "Total supply should be 2000");
```

#### Test Case 4: Deposit with insufficient assets

```javascript
// Create a transaction with insufficient assets
const tx = {
  opcode: 10, // Deposit opcode
  params: [3, 1, 1000], // tx_hash, caller, assets
  alkanes: [
    { id: ASSET_ID, value: 500 } // Only 500 assets
  ]
};

// Execute the transaction
const result = await executeTransaction(tx);

// Verify the result
assert(!result.success, "Deposit with insufficient assets should fail");
assert(result.error.includes("Insufficient assets received"), "Error should mention insufficient assets");
```

#### Test Case 5: Redeem with correct alkane ID

```javascript
// Create a transaction with the correct share token ID
const tx = {
  opcode: 13, // Redeem opcode
  params: [4, 1, 1, 500], // tx_hash, caller, receiver, shares
  alkanes: [
    { id: CONTRACT_ID, value: 500 } // Correct share token ID
  ]
};

// Execute the transaction
const result = await executeTransaction(tx);

// Verify the result
assert(result.success, "Redeem with correct alkane ID should succeed");
assert(result.totalAssets === 1500, "Total assets should be 1500");
assert(result.totalSupply === 1500, "Total supply should be 1500");
```

#### Test Case 6: Redeem with incorrect alkane ID

```javascript
// Create a transaction with an incorrect share token ID
const tx = {
  opcode: 13, // Redeem opcode
  params: [5, 1, 1, 500], // tx_hash, caller, receiver, shares
  alkanes: [
    { id: "wrong_token_id", value: 500 } // Incorrect share token ID
  ]
};

// Execute the transaction
const result = await executeTransaction(tx);

// Verify the result
assert(!result.success, "Redeem with incorrect alkane ID should fail");
assert(result.error.includes("Share verification error"), "Error should mention share verification");
```

#### Test Case 7: Redeem with insufficient shares

```javascript
// Create a transaction with insufficient shares
const tx = {
  opcode: 13, // Redeem opcode
  params: [6, 1, 1, 500], // tx_hash, caller, receiver, shares
  alkanes: [
    { id: CONTRACT_ID, value: 250 } // Only 250 shares
  ]
};

// Execute the transaction
const result = await executeTransaction(tx);

// Verify the result
assert(!result.success, "Redeem with insufficient shares should fail");
assert(result.error.includes("Insufficient shares received"), "Error should mention insufficient shares");
```

#### Test Case 8: Redeem with multiple alkane IDs

```javascript
// Create a transaction with multiple token IDs including the correct one
const tx = {
  opcode: 13, // Redeem opcode
  params: [7, 1, 1, 500], // tx_hash, caller, receiver, shares
  alkanes: [
    { id: "wrong_token_id_1", value: 250 },
    { id: CONTRACT_ID, value: 500 }, // Correct share token ID
    { id: "wrong_token_id_2", value: 300 }
  ]
};

// Execute the transaction
const result = await executeTransaction(tx);

// Verify the result
assert(result.success, "Redeem with multiple alkane IDs should succeed");
assert(result.totalAssets === 1000, "Total assets should be 1000");
assert(result.totalSupply === 1000, "Total supply should be 1000");
```

#### Test Case 9: No transaction context

```javascript
// Create a transaction with no alkanes
const tx = {
  opcode: 10, // Deposit opcode
  params: [8, 1, 1000], // tx_hash, caller, assets
  alkanes: [] // No alkanes
};

// Execute the transaction
const result = await executeTransaction(tx);

// Verify the result
assert(!result.success, "Deposit with no transaction context should fail");
assert(result.error.includes("Asset verification error"), "Error should mention asset verification");
```

### 4. Test Execution

1. Run the test script:
   ```bash
   NODE_OPTIONS=--require=/workspaces/boiler/oyl-sdk/lib/shared/load_patch.js node oylnet_alkane_verification_tests.js
   ```

2. Generate blocks after each transaction to ensure they are confirmed:
   ```javascript
   // Generate blocks to confirm transaction
   await generateBlocks(2);
   ```

3. Verify the contract state after each transaction:
   ```javascript
   // Verify contract state
   const totalAssets = await getTotalAssets();
   const totalSupply = await getTotalSupply();
   ```

### 5. Test Report Generation

The test script will generate a comprehensive test report that includes:

1. Test case name and description
2. Transaction details
3. Expected result
4. Actual result
5. Pass/fail status
6. Error messages (if any)
7. Contract state before and after the transaction

## Challenges and Considerations

### 1. Transaction Context Simulation

OylNet may not provide a direct way to simulate the transaction context with custom alkane IDs. We may need to:

1. Create actual tokens with the desired IDs
2. Transfer them to the test account
3. Use them in the test transactions

### 2. Error Handling

OylNet may return errors in a different format than our mock implementation. We need to:

1. Parse the error messages correctly
2. Map them to the expected error types
3. Handle any unexpected errors gracefully

### 3. Block Generation

We need to be careful not to generate too many blocks at once to avoid API lockout. We should:

1. Generate only 1-2 blocks at a time
2. Add delays between block generation calls
3. Handle any rate limiting errors gracefully

### 4. Transaction Confirmation

We need to ensure that each transaction is confirmed before proceeding to the next test case. We should:

1. Wait for sufficient block confirmations
2. Verify the transaction status
3. Handle any confirmation failures gracefully

## Timeline

1. **Day 1**: Set up the test environment and deploy the contract
2. **Day 2**: Implement the test script and the first 4 test cases
3. **Day 3**: Implement the remaining 5 test cases
4. **Day 4**: Run the tests, debug any issues, and generate the test report
5. **Day 5**: Document the results and update the project documentation

## Success Criteria

The test is considered successful if:

1. All 9 test cases are executed on OylNet
2. The results match the expected behavior
3. The contract state is correctly updated after each transaction
4. The test report is generated and provides comprehensive information about the test results

## Conclusion

This test plan provides a comprehensive approach to verifying the alkane ID verification process in a real blockchain environment. By implementing these tests on OylNet, we can ensure that the token-based architecture works correctly in a production-like setting and provides the expected security guarantees.
