# System Patterns & Architecture Lessons

## CRITICAL: Vault Factory Initialization Pattern

### The Working Initialization Sequence

```rust
// 1. Deploy contract templates
let template_block = alkane_helpers::init_with_multiple_cellpacks_with_tx(
    [free_mint_build::get_bytes(), alk4626_vault_factory_build::get_bytes()].into(),
    [vec![3u128, 797u128, 101u128], vec![3u128, 0x37a, 10u128]].into_iter()
        .map(|v| into_cellpack(v)).collect()
);

// 2. Create token contract with sufficient supply
let free_mint_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
    // ... initialize with large supply: 10M+ tokens
    message: into_cellpack(vec![6u128, 797u128, 0u128, 10000000u128, 2000000u128, 20000000u128, ...])
}]);

// 3. Mint tokens to specific outpoint
let mint_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
    // ... mint tokens to outpoint for later use
    message: into_cellpack(vec![2u128, 1u128, 77u128])
}]);

// 4. CRITICAL: Match parameters exactly
let mint_outpoint = OutPoint { txid: mint_block.txdata[0].compute_txid(), vout: 0 };
let available_tokens = mint_sheet.get(&token_rune_id); // Get ACTUAL balance
let preloaded_rewards = available_tokens; // MUST match exactly!

// 5. Initialize vault with exact matching
let init_vault_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
    input: vec![TxIn { previous_output: mint_outpoint, ... }], // Use the tokens
    // ... 
    message: into_cellpack(vec![
        4u128, 0x37a, 0u128,                    // target + Initialize opcode
        deposit_token_id.block, deposit_token_id.tx,  // AlkaneId (2 u128s)
        reward_token_id.block, reward_token_id.tx,    // AlkaneId (2 u128s)
        reward_per_block,                       // u128
        start_block,                           // u128
        preloaded_rewards,                     // u128 - MUST match edict!
        fee_percentage                         // u128
    ]),
    edicts: vec![
        ProtostoneEdict {
            id: ProtoruneRuneId { block: token_id.block, tx: token_id.tx },
            amount: preloaded_rewards, // SAME VALUE as parameter!
            output: 1,
        }
    ],
}]);
```

## Alkane Contract Architecture Patterns

### 1. **Parameter Validation is Strict**
Alkane contracts validate parameters against actual token transfers:
```rust
// In vault factory initialize():
let mut reward_tokens_received = 0u128;
for transfer in &context.incoming_alkanes.0 {
    if transfer.id == reward_token_id {
        reward_tokens_received += transfer.value;
    }
}

// CRITICAL VALIDATION
if reward_tokens_received != preloaded_rewards {
    return Err(anyhow!("Reward token amount ({}) doesn't match preloaded_rewards parameter ({})", 
                       reward_tokens_received, preloaded_rewards));
}
```

### 2. **Token Transfer Mechanics**
```rust
// When you reference an outpoint in transaction input:
input: vec![TxIn { previous_output: outpoint_with_2M_tokens, ... }]

// ALL tokens at that outpoint get transferred (2M tokens)
// Your edict amount must match what's actually being transferred
edicts: vec![ProtostoneEdict { amount: 2_000_000u128, ... }] // Must match!
```

### 3. **Storage Initialization Pattern**
Successful vault initialization sets ALL required storage keys:
```rust
// Expected storage after successful init:
/start_block: [3, 0, 0, 0, ...]
/deposit_token_id: [2, 0, 0, 0, ..., 1, 0, 0, 0, ...]  // AlkaneId as 32 bytes
/reward_token_id: [2, 0, 0, 0, ..., 1, 0, 0, 0, ...]   // AlkaneId as 32 bytes  
/reward_per_block: [232, 3, 0, 0, ...]                 // 1000 as little-endian
/total_reward_pool: [128, 132, 30, 0, ...]             // 2000000 as little-endian
/remaining_rewards: [128, 132, 30, 0, ...]             // 2000000 as little-endian
/distributed_rewards: [0, 0, 0, 0, ...]                // 0 initially
```

## Contract Communication Patterns

### 1. **Auth Token Pattern**
```rust
// Vault factory returns auth token after successful init:
response.alkanes.0.push(AlkaneTransfer {
    id: context.myself.clone(), // Vault factory's own ID
    value: 1u128,               // Exactly 1 auth token
});
```

### 2. **Position Token Creation Pattern**
```rust
// Vault factory creates position tokens via cellpack call:
let cellpack = Cellpack {
    target: AlkaneId { block: 6, tx: POSITION_TOKEN_TEMPLATE_ID },
    inputs: vec![0x0, position_id, assets, shares, current_block, deposit_token_id.block, deposit_token_id.tx],
};
```

### 3. **Trace Analysis Pattern**
```rust
// Success indicators in trace:
ReturnContext(TraceResponse { 
    inner: ExtendedCallResponse { 
        alkanes: AlkaneTransferParcel([AlkaneTransfer { ... }]), // Auth token returned
        storage: StorageMap({...}),                              // Storage populated
        data: []                                                 // No error data
    }
})

// Failure indicators:
RevertContext(TraceResponse { 
    inner: ExtendedCallResponse {
        data: [8, 195, 121, 160, 65, 76, 75, ...] // Error message bytes
    }
})
```

## Token Economics Patterns

### 1. **Edict vs Parameter Matching**
```rust
// Parameter in message
preloaded_rewards: 1000000u128

// MUST equal edict amount
ProtostoneEdict { amount: 1000000u128, ... }

// MUST equal actual tokens being transferred from input outpoint
// If outpoint has 2M tokens, you MUST either:
// A) Use all 2M tokens: preloaded_rewards = 2000000u128
// B) Split tokens first, then use exact amount
```

### 2. **Balance Sheet Verification**
```rust
// Always verify token balances at each step:
let sheet = load_sheet(&RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
    .OUTPOINT_TO_RUNES.select(&consensus_encode(&outpoint)?));
let actual_balance = sheet.get(&ProtoruneRuneId { block: token_id.block, tx: token_id.tx });

// Use actual balance for calculations, don't assume amounts
```

### 3. **Transaction Outpoint Management**
```rust
// ❌ WRONG - Reusing same outpoint causes "Transaction already used for minting"
let outpoint = OutPoint { txid: block.txdata[0].compute_txid(), vout: 0 };
// ... use outpoint in transaction 1
// ... use same outpoint in transaction 2 <- FAILS!

// ✅ RIGHT - Each transaction uses unique outpoints
let outpoint1 = OutPoint { txid: block1.txdata[0].compute_txid(), vout: 0 };
let outpoint2 = OutPoint { txid: block2.txdata[0].compute_txid(), vout: 0 };
```

## Testing Architecture Patterns

### 1. **Progressive Complexity Testing**
```rust
// Level 1: Contract deployment only
#[wasm_bindgen_test] fn test_deploy() { ... }

// Level 2: Single contract initialization
#[wasm_bindgen_test] fn test_token_init() { ... }

// Level 3: Multi-contract initialization
#[wasm_bindgen_test] fn test_vault_init() { ... }

// Level 4: Basic operations
#[wasm_bindgen_test] fn test_deposit() { ... }

// Level 5: Complex scenarios
#[wasm_bindgen_test] fn test_reward_distribution() { ... }
```

### 2. **Debugging Helper Patterns**
```rust
// Always create helper functions for repeated operations:
fn create_basic_token_setup() -> Result<(Block, Block, AlkaneId)> { ... }
fn verify_balance_at_outpoint(outpoint: &OutPoint, expected: u128) -> Result<()> { ... }
fn trace_transaction_and_analyze(outpoint: &OutPoint) -> Result<bool> { ... }
```

### 3. **Error Pattern Recognition**
```rust
// Common error patterns and their meanings:
"Must preload reward pool with tokens" -> No tokens sent via edict
"doesn't match preloaded_rewards parameter" -> Parameter/edict mismatch  
"Transaction already used for minting" -> Outpoint reuse
"wasm unreachable instruction executed" -> Contract panic (check parameters)
"Expected exactly one token type" -> Multiple token types in transfer
```

## Key Architectural Insights

1. **Alkane contracts are parameter-strict** - every parameter must match exactly
2. **Token transfers are all-or-nothing** - referencing an outpoint transfers ALL tokens at that outpoint
3. **Storage initialization is atomic** - either all storage keys are set correctly or the transaction reverts
4. **Auth tokens are proof of successful operations** - always check for auth token return
5. **Trace analysis is essential** - ReturnContext vs RevertContext tells the story

**The $50 lesson: Always validate parameters match exactly before assuming complex architectural issues.**
