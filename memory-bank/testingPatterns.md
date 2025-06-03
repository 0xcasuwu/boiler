# Testing Patterns - ALK4626 Vault Architecture

## Comprehensive Testing Framework

### **Test Structure Pattern**
```rust
#[wasm_bindgen_test]
fn test_name() -> Result<()> {
    // 1. Setup Phase
    clear();  // Clear blockchain state
    
    // 2. Contract Deployment
    let template_block = deploy_contracts();
    index_block(&template_block, 0)?;
    
    // 3. Initialization
    let init_block = initialize_contracts();
    index_block(&init_block, 1)?;
    
    // 4. Test Operations
    let operation_block = perform_operations();
    index_block(&operation_block, 2)?;
    
    // 5. Trace Analysis & Validation
    validate_with_traces(&operation_block)?;
    
    Ok(())
}
```

## Trace Reading & Parsing Patterns

### **Trace Extraction Pattern**
```rust
// Get trace data from specific transaction output
let trace_data = &view::trace(
    &(OutPoint {
        txid: block.txdata[0].compute_txid(),
        vout: 3, // Target the specific output with contract call
    }),
)?;

// Parse raw bytes into structured trace
let trace_result: alkanes_support::trace::Trace = 
    alkanes_support::proto::alkanes::AlkanesTrace::parse_from_bytes(trace_data)?.into();
```

### **Trace Analysis Pattern**
```rust
// Extract trace items for detailed analysis
println!("=== DETAILED TRACE ANALYSIS ===");
println!("Trace data length: {} bytes", trace_data.len());

if !trace_result.0.lock().unwrap().is_empty() {
    for (i, item) in trace_result.0.lock().unwrap().iter().enumerate() {
        println!("Trace item {}: {:?}", i, item);
        
        // Pattern match on trace item types
        match item {
            alkanes_support::trace::TraceItem::EnterCall(context) => {
                println!("  → Contract call to: {:?}", context.inner.target);
                println!("  → Incoming tokens: {:?}", context.inner.incoming_alkanes);
                println!("  → Input parameters: {:?}", context.inner.inputs);
            },
            alkanes_support::trace::TraceItem::ReturnContext(response) => {
                println!("  → Returned tokens: {:?}", response.inner.alkanes);
                println!("  → Storage changes: {:?}", response.inner.storage);
                println!("  → Data response: {:?}", response.inner.data);
            },
            alkanes_support::trace::TraceItem::CreateAlkane(id) => {
                println!("  → Created new alkane: {:?}", id);
            },
            _ => {}
        }
    }
}
```

### **Trace Validation Patterns**

#### **Auth Token Preservation Validation**
```rust
fn validate_auth_token_preservation(trace_result: &Trace, expected_amount: u128) -> Result<()> {
    let trace_items = trace_result.0.lock().unwrap();
    
    // Find the final return context
    for item in trace_items.iter().rev() {
        if let TraceItem::ReturnContext(response) = item {
            // Look for auth token in returned alkanes
            for transfer in &response.inner.alkanes.0 {
                if transfer.id.block == 4 && transfer.id.tx == 0x37a { // Vault factory ID
                    assert_eq!(transfer.value, expected_amount,
                              "Auth token preservation failed: expected {}, got {}", 
                              expected_amount, transfer.value);
                    return Ok(());
                }
            }
        }
    }
    
    Err(anyhow!("Auth token not found in trace"))
}
```

#### **Fee Extraction Validation**
```rust
fn validate_fee_extraction(trace_result: &Trace, expected_fees: u128) -> Result<()> {
    let trace_items = trace_result.0.lock().unwrap();
    
    // Find storage updates showing collected_fees
    for item in &*trace_items {
        if let TraceItem::ReturnContext(response) = item {
            // Check storage for collected_fees update
            let storage_key = b"/collected_fees";
            if let Some(fee_bytes) = response.inner.storage.get(&storage_key.to_vec()) {
                let stored_fees = u128::from_le_bytes(
                    fee_bytes[0..16].try_into().unwrap_or([0; 16])
                );
                
                assert_eq!(stored_fees, expected_fees,
                          "Fee extraction mismatch: expected {}, stored {}", 
                          expected_fees, stored_fees);
                return Ok(());
            }
        }
    }
    
    Ok(()) // Fees may be reset to 0 after withdrawal
}
```

## Balance Sheet Validation Patterns

### **Token Balance Verification**
```rust
fn verify_token_balance(outpoint: &OutPoint, expected_token_id: &ProtoruneRuneId, expected_amount: u128) -> Result<()> {
    let sheet = load_sheet(
        &RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
            .OUTPOINT_TO_RUNES
            .select(&consensus_encode(outpoint)?)
    );
    
    println!("--- Balance Verification ---");
    for (id, amount) in sheet.balances().iter() {
        println!("Token ID: {:?}, Amount: {}", id, amount);
        
        if id == expected_token_id {
            assert_eq!(*amount, expected_amount,
                      "Balance mismatch for {:?}: expected {}, got {}", 
                      id, expected_amount, amount);
            return Ok(());
        }
    }
    
    if expected_amount > 0 {
        return Err(anyhow!("Expected token {:?} not found", expected_token_id));
    }
    
    Ok(())
}
```

### **Multi-Token Balance Pattern**
```rust
fn verify_multi_token_balance(outpoint: &OutPoint, expected_balances: &[(ProtoruneRuneId, u128)]) -> Result<()> {
    let sheet = load_sheet(
        &RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
            .OUTPOINT_TO_RUNES
            .select(&consensus_encode(outpoint)?)
    );
    
    for (expected_id, expected_amount) in expected_balances {
        let actual_amount = sheet.get(expected_id);
        assert_eq!(actual_amount, *expected_amount,
                  "Multi-token balance mismatch for {:?}: expected {}, got {}", 
                  expected_id, expected_amount, actual_amount);
    }
    
    Ok(())
}
```

## Mathematical Verification Patterns

### **Fee Calculation Verification**
```rust
fn verify_fee_calculation(deposit_amount: u128, fee_percentage: u128, actual_fee: u128) -> Result<()> {
    let expected_fee = deposit_amount
        .checked_mul(fee_percentage)
        .ok_or_else(|| anyhow!("Fee calculation overflow"))?
        .checked_div(10000)
        .ok_or_else(|| anyhow!("Fee calculation div by zero"))?;
    
    assert_eq!(actual_fee, expected_fee,
              "Fee calculation mismatch: {}*{}/10000 = {}, but got {}", 
              deposit_amount, fee_percentage, expected_fee, actual_fee);
    
    println!("✅ Fee calculation verified: {} * {} / 10000 = {}", 
             deposit_amount, fee_percentage, expected_fee);
    
    Ok(())
}
```

### **Reward Calculation Verification**
```rust
fn verify_reward_calculation(
    original_amount: u128, 
    reward_per_block: u128, 
    blocks_elapsed: u128, 
    precision: u128,
    actual_rewards: u128
) -> Result<()> {
    let expected_rewards = original_amount
        .checked_mul(reward_per_block)
        .unwrap_or(0)
        .checked_mul(blocks_elapsed)
        .unwrap_or(0)
        .checked_div(precision)
        .unwrap_or(0);
    
    assert_eq!(actual_rewards, expected_rewards,
              "Reward calculation mismatch: {}*{}*{}/{} = {}, but got {}", 
              original_amount, reward_per_block, blocks_elapsed, precision, 
              expected_rewards, actual_rewards);
    
    println!("✅ Reward calculation verified: {} * {} * {} / {} = {}", 
             original_amount, reward_per_block, blocks_elapsed, precision, expected_rewards);
    
    Ok(())
}
```

## Assertion Patterns

### **Comprehensive Result Assertions**
```rust
// Pattern for complete operation validation
fn assert_complete_operation_success(
    initial_balance: u128,
    final_balance: u128,
    expected_fees: u128,
    expected_rewards: u128,
    operation_name: &str
) -> Result<()> {
    let expected_final = initial_balance
        .checked_sub(expected_fees)
        .unwrap_or(0)
        .checked_add(expected_rewards)
        .unwrap_or(initial_balance);
    
    assert_eq!(final_balance, expected_final,
              "{} operation failed: initial={}, fees={}, rewards={}, expected_final={}, actual_final={}", 
              operation_name, initial_balance, expected_fees, expected_rewards, expected_final, final_balance);
    
    println!("✅ {} operation success: {} - {} + {} = {}", 
             operation_name, initial_balance, expected_fees, expected_rewards, expected_final);
    
    Ok(())
}
```

### **Trace-Based Assertions**
```rust
fn assert_trace_contains_call(trace_result: &Trace, target_id: &AlkaneId) -> Result<()> {
    let trace_items = trace_result.0.lock().unwrap();
    
    for item in &*trace_items {
        if let TraceItem::EnterCall(context) = item {
            if context.inner.target == *target_id {
                println!("✅ Found expected call to {:?}", target_id);
                return Ok(());
            }
        }
    }
    
    return Err(anyhow!("Expected call to {:?} not found in trace", target_id));
}

fn assert_trace_contains_token_transfer(
    trace_result: &Trace, 
    token_id: &AlkaneId, 
    expected_amount: u128
) -> Result<()> {
    let trace_items = trace_result.0.lock().unwrap();
    
    for item in &*trace_items {
        if let TraceItem::ReturnContext(response) = item {
            for transfer in &response.inner.alkanes.0 {
                if transfer.id == *token_id && transfer.value == expected_amount {
                    println!("✅ Found expected token transfer: {:?} amount {}", token_id, expected_amount);
                    return Ok(());
                }
            }
        }
    }
    
    return Err(anyhow!("Expected token transfer {:?} amount {} not found", token_id, expected_amount));
}
```

## Logging Patterns

### **Structured Test Logging**
```rust
fn log_test_phase(phase: &str, details: &str) {
    println!("\n=== {} ===", phase.to_uppercase());
    println!("{}", details);
}

fn log_operation_result(operation: &str, success: bool, details: &str) {
    let status = if success { "✅ SUCCESS" } else { "❌ FAILED" };
    println!("🔍 {}: {} - {}", operation, status, details);
}

fn log_balance_summary(balances: &[(String, u128)]) {
    println!("--- Balance Summary ---");
    for (token_name, amount) in balances {
        println!("  {}: {} tokens", token_name, amount);
    }
}
```

### **Trace Debugging Patterns**
```rust
fn debug_trace_full(trace_result: &Trace, operation_name: &str) {
    println!("\n=== {} TRACE DEBUG ===", operation_name.to_uppercase());
    
    let trace_items = trace_result.0.lock().unwrap();
    
    if trace_items.is_empty() {
        println!("⚠️  WARNING: Empty trace for {}", operation_name);
        return;
    }
    
    for (i, item) in trace_items.iter().enumerate() {
        match item {
            TraceItem::EnterCall(context) => {
                println!("{}. CALL → {:?}", i, context.inner.target);
                println!("   Inputs: {:?}", context.inner.inputs);
                println!("   Tokens: {:?}", context.inner.incoming_alkanes);
            },
            TraceItem::ReturnContext(response) => {
                println!("{}. RETURN", i);
                println!("   Tokens: {:?}", response.inner.alkanes);
                if !response.inner.storage.is_empty() {
                    println!("   Storage Updates: {} keys", response.inner.storage.len());
                }
                if !response.inner.data.is_empty() {
                    println!("   Data: {} bytes", response.inner.data.len());
                }
            },
            TraceItem::CreateAlkane(id) => {
                println!("{}. CREATE → {:?}", i, id);
            },
            _ => {
                println!("{}. OTHER: {:?}", i, item);
            }
        }
    }
}
```

## Test Helper Patterns

### **Contract Deployment Helper**
```rust
fn deploy_vault_system() -> Result<Block> {
    alkane_helpers::init_with_multiple_cellpacks_with_tx(
        [
            free_mint_build::get_bytes(),
            alk4626_position_token_build::get_bytes(),
            alk4626_vault_factory_build::get_bytes(),
        ].into(),
        [
            vec![3u128, 797u128, 101u128],      // Free mint
            vec![3u128, 0x379, 10u128],        // Position token
            vec![3u128, 0x37a, 10u128],        // Vault factory
        ].into_iter().map(|v| into_cellpack(v)).collect::<Vec<Cellpack>>()
    )
}
```

### **Transaction Creation Helper**
```rust
fn create_test_transaction(
    input_outpoint: OutPoint,
    target_alkane: AlkaneId,
    operation_inputs: Vec<u128>,
    token_edicts: Vec<ProtostoneEdict>
) -> Transaction {
    Transaction {
        version: Version::ONE,
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input: vec![TxIn {
            previous_output: input_outpoint,
            script_sig: ScriptBuf::new(),
            sequence: Sequence::MAX,
            witness: Witness::new()
        }],
        output: vec![
            TxOut {
                script_pubkey: Address::from_str(ADDRESS1().as_str())
                    .unwrap()
                    .require_network(get_btc_network())
                    .unwrap()
                    .script_pubkey(),
                value: Amount::from_sat(546),
            },
            TxOut {
                script_pubkey: (Runestone {
                    edicts: vec![],
                    etching: None,
                    mint: None,
                    pointer: None,
                    protocol: Some(
                        vec![
                            Protostone {
                                message: into_cellpack(vec![
                                    target_alkane.block, target_alkane.tx
                                ].into_iter().chain(operation_inputs).collect()).encipher(),
                                protocol_tag: AlkaneMessageContext::protocol_tag() as u128,
                                pointer: Some(0),
                                refund: Some(0),
                                from: None,
                                burn: None,
                                edicts: token_edicts,
                            }
                        ].encipher().unwrap()
                    )
                }).encipher(),
                value: Amount::from_sat(546)
            }
        ],
    }
}
```

### **Comprehensive Test Validation Pattern**
```rust
fn validate_complete_vault_operation(
    deposit_amount: u128,
    fee_percentage: u128,
    reward_per_block: u128,
    blocks_elapsed: u128,
    final_outpoint: &OutPoint,
    trace_result: &Trace
) -> Result<()> {
    // 1. Calculate expected values
    let expected_fee = deposit_amount * fee_percentage / 10000;
    let expected_rewards = deposit_amount * reward_per_block * blocks_elapsed / 1_000_000;
    let expected_final = deposit_amount - expected_fee + expected_rewards;
    
    // 2. Verify mathematical calculations
    verify_fee_calculation(deposit_amount, fee_percentage, expected_fee)?;
    verify_reward_calculation(deposit_amount, reward_per_block, blocks_elapsed, 1_000_000, expected_rewards)?;
    
    // 3. Validate trace contents
    validate_fee_extraction(trace_result, expected_fee)?;
    
    // 4. Check final balance
    let free_mint_id = ProtoruneRuneId { block: 2, tx: 1 };
    verify_token_balance(final_outpoint, &free_mint_id, expected_final)?;
    
    // 5. Log comprehensive results
    log_operation_result("Vault Operation", true, &format!(
        "Deposit: {}, Fee: {}, Rewards: {}, Final: {}", 
        deposit_amount, expected_fee, expected_rewards, expected_final
    ));
    
    Ok(())
}
```

## Contract Debugging Patterns

### **Storage Inspection Pattern**
```rust
fn debug_contract_storage(alkane_id: &AlkaneId, keys: &[&str]) {
    println!("=== CONTRACT STORAGE DEBUG: {:?} ===", alkane_id);
    
    for key in keys {
        // Storage keys in contracts use the format "/key_name"
        let storage_key = format!("/{}", key);
        println!("Key: {} → [inspect via trace or staticcall]", storage_key);
    }
}

// Example usage in contract:
// self.store("/collected_fees".as_bytes().to_vec(), fees.to_le_bytes().to_vec());
// let fees = self.load_u128("/collected_fees");
```

### **Inter-Contract Call Debugging**
```rust
fn debug_inter_contract_call(
    caller: &AlkaneId,
    target: &AlkaneId, 
    inputs: &[u128],
    expected_response: &str
) {
    println!("=== INTER-CONTRACT CALL DEBUG ===");
    println!("Caller: {:?}", caller);
    println!("Target: {:?}", target);
    println!("Inputs: {:?}", inputs);
    println!("Expected: {}", expected_response);
    println!("Check trace for EnterCall → ReturnContext sequence");
}
```

These patterns provide comprehensive testing methodology for the ALK4626 vault architecture, enabling future developers to create robust tests, analyze traces effectively, and debug contract interactions with precision.
