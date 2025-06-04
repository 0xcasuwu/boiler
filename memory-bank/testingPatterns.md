# Testing Patterns & Debugging Lessons

## CRITICAL LESSON: $50 Parameter Validation Bug

### The Bug
Vault factory initialization was failing with seemingly random errors across all tests. Root cause: **preloaded_rewards parameter must EXACTLY match tokens sent via edict**.

```rust
// ❌ WRONG - This cost us $50 in debugging time
let preloaded_rewards = 1000000u128;
let sent_via_edict = 2000000u128; // Different amount!

// ✅ RIGHT - These must match EXACTLY
let available_tokens = mint_sheet.get(&token_id);
let preloaded_rewards = available_tokens; // Same value
let sent_via_edict = preloaded_rewards;   // Same value
```

### Error Messages That Fooled Us
- "Must preload reward pool with tokens" 
- "Reward token amount (2000000) doesn't match preloaded_rewards parameter (1000000)"
- "wasm unreachable instruction executed"

### The Debugging Method That Worked

#### 1. **Isolate to Minimal Test Case**
```rust
#[wasm_bindgen_test]
fn test_step_by_step_initialization() -> Result<()> {
    // Deploy only what's needed
    // Test one step at a time
    // Trace each step individually
}
```

#### 2. **Parameter Validation Matrix**
Test all parameter combinations systematically:
```rust
let test_cases = vec![
    ("CORRECT_PARAMS", 1000000u128, 1000000u128), // preloaded = sent ✅
    ("MISMATCH_HIGH", 1000000u128, 500000u128),    // preloaded > sent ❌
    ("MISMATCH_LOW", 500000u128, 1000000u128),     // preloaded < sent ❌
    ("ZERO_PRELOADED", 0u128, 1000000u128),        // zero preloaded ❌
    ("ZERO_SENT", 1000000u128, 0u128),             // zero sent ❌
];
```

#### 3. **Trace Analysis Pattern**
```rust
let trace_debug_str = format!("{:?}", trace_result.0.lock().unwrap());

if trace_debug_str.contains("doesn't match preloaded_rewards") {
    println!("❌ ERROR: Token amount mismatch - edict not working correctly");
} else if trace_debug_str.contains("ReturnContext") {
    println!("✅ SUCCESS: Initialization completed successfully!");
}
```

## Essential Testing Patterns

### 1. **Balance Sheet Verification**
Always verify token balances at each step:
```rust
let mint_sheet = load_sheet(
    &RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
        .OUTPOINT_TO_RUNES
        .select(&consensus_encode(&outpoint)?)
);
let available_tokens = mint_sheet.get(&token_rune_id);
```

### 2. **Transaction Input/Output Tracing**
Use specific outpoints to avoid "Transaction already used for minting":
```rust
// ❌ WRONG - Reusing same transaction
let input1 = OutPoint { txid: mint_block.txdata[0].compute_txid(), vout: 0 };
let input2 = OutPoint { txid: mint_block.txdata[0].compute_txid(), vout: 0 }; // Same!

// ✅ RIGHT - Different transactions or different outputs
let input1 = OutPoint { txid: mint_block1.txdata[0].compute_txid(), vout: 0 };
let input2 = OutPoint { txid: mint_block2.txdata[0].compute_txid(), vout: 0 };
```

### 3. **Parameter Structure Validation**
Always match the exact parameter order from the contract:
```rust
// From vault factory Initialize message:
Initialize {
    deposit_token_id: AlkaneId,    // 2 u128s (block, tx)
    reward_token_id: AlkaneId,     // 2 u128s (block, tx)  
    reward_per_block: u128,        // 1 u128
    start_block: u128,             // 1 u128
    preloaded_rewards: u128,       // 1 u128 - MUST match edict!
    fee_percentage: u128,          // 1 u128
}

// Correct cellpack construction:
vec![
    4u128, 0x37a, 0u128,                    // target + opcode
    deposit_token_id.block, deposit_token_id.tx,  // AlkaneId
    reward_token_id.block, reward_token_id.tx,    // AlkaneId
    reward_per_block,                       // u128
    start_block,                           // u128
    preloaded_rewards,                     // u128 - KEY!
    fee_percentage                         // u128
]
```

## Debugging Anti-Patterns (What NOT to Do)

### ❌ Don't Assume Complex Issues First
We spent hours debugging token economics, storage patterns, and contract architecture when it was a simple parameter mismatch.

### ❌ Don't Test Everything at Once
Big integration tests hide the root cause. Start with minimal cases.

### ❌ Don't Ignore Exact Error Messages
"Reward token amount (X) doesn't match preloaded_rewards parameter (Y)" - this was the exact clue we needed.

### ❌ Don't Skip Parameter Validation
Always validate that your parameters match the contract's expected structure.

## Success Indicators to Look For

### ✅ Initialization Success Checklist
1. `ReturnContext` (not `RevertContext`) in trace
2. Auth token returned with correct ID
3. Storage map populated with all expected keys
4. No error messages in trace data
5. Test result shows "ok. X passed; 0 failed"

### ✅ Expected Storage Keys After Init
```rust
/start_block, /initialized, /deposit_token_id, /distributed_rewards,
/total_shares, /owner, /total_assets, /position_count, 
/reward_per_block, /remaining_rewards, /reward_token_id,
/collected_fees, /last_update_block, /fee_percentage, /total_reward_pool
```

## Cost-Saving Debugging Strategy

1. **Start Minimal**: Single contract, single function test
2. **Parameter Matrix**: Test all parameter combinations systematically  
3. **Trace Everything**: Every step should have trace analysis
4. **Balance Verification**: Check token balances at each step
5. **Error Message Analysis**: Read error messages literally
6. **Progressive Complexity**: Only add complexity after basics work

**Never again should a simple parameter mismatch cost this much debugging time.**
