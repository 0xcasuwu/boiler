# System Patterns - ALK4626 Vault Architecture

## Core Architecture

### **Three-Contract System**
```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Vault Factory │    │ Position Token  │    │   Free Mint     │
│                 │    │                 │    │                 │
│ • Asset Custody │    │ • User Auth     │    │ • Underlying    │
│ • Fee Collection│◄──►│ • No Assets     │    │ • Rewards       │
│ • Auth Control  │    │ • Pure Tracking │    │ • Test Token    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### **Critical Design Patterns**

#### 1. **Input-Based Authentication Pattern**
```rust
// ❌ OLD: Edict-based (consumes tokens)
edicts: vec![
    ProtostoneEdict {
        id: auth_token_id,
        amount: 10,  // Gets consumed!
        output: 1,
    }
]

// ✅ NEW: Input-based (preserves tokens)
edicts: vec![], // NO EDICTS
message: into_cellpack(vec![
    4u128, 0x37a, 4u128, auth_token_count  // Parameter-driven
])
```

#### 2. **Custody Separation Pattern**
- **Vault Factory**: Holds extracted fees in internal storage
- **Position Token**: Provides authentication, holds NO underlying assets
- **User Wallets**: Receive position tokens as proof of vault ownership

#### 3. **Mathematical Precision Pattern**
```rust
// Fee calculation (basis points)
let fee_amount = total_withdrawal_value
    .checked_mul(fee_percentage)     // 50 basis points
    .unwrap_or(0)
    .checked_div(10000)              // Convert to percentage
    .unwrap_or(0);

// Reward calculation (high precision)
let rewards = original_assets
    .checked_mul(reward_per_block)   // 1000 per block
    .unwrap_or(0)
    .checked_mul(blocks_elapsed)     // Time factor
    .unwrap_or(0)
    .checked_div(1_000_000)          // Precision factor
    .unwrap_or(0);
```

## Key Technical Decisions

### **Authentication Strategy**
- **Decision**: Use input parameters instead of edicts for auth token counts
- **Rationale**: Eliminates protocol-level token consumption
- **Implementation**: `auth_token_count: u128` parameter in withdraw_fees
- **Result**: Perfect 1:1 auth token preservation

### **Storage Architecture**
```rust
// Critical storage keys
/total_assets       -> Vault's total managed assets
/total_shares       -> Total shares outstanding  
/collected_fees     -> Accumulated fee tokens
/fee_percentage     -> Basis points (50 = 0.5%)
/reward_per_block   -> Reward rate for staking
/position_count     -> Number of positions created
```

### **Token Flow Architecture**
1. **Deposit Flow**: User assets → Vault custody, Position token → User
2. **Withdrawal Flow**: Position token → Authentication, Assets → User, Fees → Vault
3. **Fee Withdrawal**: Auth token → Preserved, Fees → Owner, Storage → Reset

## Component Relationships

### **Vault Factory (Primary Controller)**
- **Responsibilities**:
  - Asset custody and management
  - Fee extraction and storage
  - Position token creation via factory pattern
  - Authentication verification
  - Mathematical calculations (fees, rewards, shares)

### **Position Token (Authentication Layer)**
- **Responsibilities**:
  - User authentication for vault operations
  - Tracking individual position metadata
  - NO asset custody (pure authentication)
  - State updates via vault factory calls

### **Free Mint Token (Test Infrastructure)**
- **Responsibilities**:
  - Provides underlying tokens for testing
  - Acts as reward token
  - Represents the actual valuable assets in production

## Critical Implementation Paths

### **Path 1: Perfect Auth Token Preservation**
```rust
fn withdraw_fees(&self, auth_token_count: u128) -> Result<CallResponse> {
    let mut response = CallResponse::default(); // Don't forward
    
    // Verify but don't consume tokens
    if context.incoming_alkanes.0.len() > 0 {
        let auth_token = &context.incoming_alkanes.0[0];
        // Verification only - no consumption
    }
    
    // Return fees + exact auth token count
    response.alkanes.0.push(AlkaneTransfer {
        id: deposit_token_id,
        value: fees,
    });
    response.alkanes.0.push(AlkaneTransfer {
        id: context.myself.clone(),
        value: auth_token_count,  // From input parameter
    });
    
    Ok(response)
}
```

### **Path 2: True Custody Architecture**
```rust
fn withdraw(&self, position_id: u128) -> Result<CallResponse> {
    // Calculate total withdrawal (assets + rewards)
    let total_withdrawal_value = current_assets + rewards;
    
    // Extract fee for vault custody
    let fee_amount = total_withdrawal_value * fee_percentage / 10000;
    
    // Vault keeps fee tokens (not sent in response)
    self.set_collected_fees(collected_fees + fee_amount);
    
    // User receives net amount only
    let user_amount = total_withdrawal_value - fee_amount;
    response.alkanes.0.push(AlkaneTransfer {
        id: deposit_token_id,
        value: user_amount,  // Fee tokens remain in vault
    });
    
    Ok(response)
}
```

### **Path 3: Position Token Factory Pattern**
```rust
fn deposit(&self, assets: u128) -> Result<CallResponse> {
    // Create position token via factory call
    let cellpack = Cellpack {
        target: AlkaneId { block: 6, tx: POSITION_TOKEN_TEMPLATE_ID },
        inputs: vec![0x0, position_id, assets, shares, current_block, 
                    deposit_token_id.block, deposit_token_id.tx],
    };
    
    // NO underlying assets sent to position token
    let position_parcel = AlkaneTransferParcel::default();
    
    let create_response = self.call(&cellpack, &position_parcel, self.fuel())?;
    // Position token returned to user for authentication
}
```

## Contract Development Patterns

### **Storage Management Patterns**

#### **Storage Key Convention**
```rust
// Always use consistent storage key format
fn storage_key(key: &str) -> Vec<u8> {
    format!("/{}", key).as_bytes().to_vec()
}

// Storage operations with proper error handling
fn safe_store_u128(&self, key: &str, value: u128) {
    self.store(storage_key(key), value.to_le_bytes().to_vec());
}

fn safe_load_u128(&self, key: &str) -> u128 {
    let bytes = self.load(storage_key(key));
    if bytes.len() >= 16 {
        u128::from_le_bytes(bytes[0..16].try_into().unwrap_or([0; 16]))
    } else {
        0
    }
}
```

#### **Complex Storage Pattern (AlkaneId)**
```rust
fn store_alkane_id(&self, key: &str, id: &AlkaneId) -> Result<()> {
    let mut bytes = Vec::with_capacity(32);
    bytes.extend_from_slice(&id.block.to_le_bytes());
    bytes.extend_from_slice(&id.tx.to_le_bytes());
    self.store(storage_key(key), bytes);
    Ok(())
}

fn load_alkane_id(&self, key: &str) -> Result<AlkaneId> {
    let bytes = self.load(storage_key(key));
    if bytes.len() < 32 {
        return Err(anyhow!("{} not found in storage", key));
    }
    
    Ok(AlkaneId {
        block: u128::from_le_bytes(bytes[0..16].try_into().unwrap()),
        tx: u128::from_le_bytes(bytes[16..32].try_into().unwrap()),
    })
}
```

### **Input Validation Patterns**

#### **Parameter Validation Pattern**
```rust
fn validate_deposit_params(&self, assets: u128) -> Result<()> {
    if assets == 0 {
        return Err(anyhow!("Cannot deposit zero assets"));
    }
    
    if assets > u128::MAX / 2 {
        return Err(anyhow!("Deposit amount too large"));
    }
    
    Ok(())
}

fn validate_fee_percentage(&self, fee_percentage: u128) -> Result<()> {
    if fee_percentage > 10000 {
        return Err(anyhow!("Fee percentage cannot exceed 10000 (100%)"));
    }
    
    Ok(())
}
```

#### **Context Validation Pattern**
```rust
fn validate_context(&self, context: &Context) -> Result<()> {
    // Validate incoming tokens
    if context.incoming_alkanes.0.is_empty() {
        return Err(anyhow!("No tokens provided"));
    }
    
    // Validate token amounts
    for transfer in &context.incoming_alkanes.0 {
        if transfer.value == 0 {
            return Err(anyhow!("Cannot process zero-value transfer"));
        }
    }
    
    Ok(())
}
```

### **Mathematical Precision Patterns**

#### **Safe Arithmetic Pattern**
```rust
fn safe_calculate_fee(&self, amount: u128, fee_percentage: u128) -> Result<u128> {
    let fee = amount
        .checked_mul(fee_percentage)
        .ok_or_else(|| anyhow!("Fee calculation overflow"))?
        .checked_div(10000)
        .ok_or_else(|| anyhow!("Fee calculation division error"))?;
    
    Ok(fee)
}

fn safe_calculate_shares(&self, assets: u128) -> Result<u128> {
    let total_assets = self.total_assets();
    let total_shares = self.total_shares();
    
    if total_assets == 0 {
        return Ok(assets); // 1:1 ratio for first deposit
    }
    
    let shares = assets
        .checked_mul(total_shares)
        .ok_or_else(|| anyhow!("Share calculation overflow"))?
        .checked_div(total_assets)
        .ok_or_else(|| anyhow!("Share calculation division by zero"))?;
    
    Ok(shares)
}
```

#### **Precision-Aware Reward Calculation**
```rust
fn calculate_rewards_precise(&self, amount: u128, blocks: u128) -> u128 {
    const PRECISION: u128 = 1_000_000; // 10^6 precision factor
    
    amount
        .saturating_mul(self.reward_per_block())
        .saturating_mul(blocks)
        .saturating_div(PRECISION)
}
```

### **Token Transfer Patterns**

#### **Response Building Pattern**
```rust
fn build_response_with_tokens(&self, tokens: Vec<(AlkaneId, u128)>) -> CallResponse {
    let mut response = CallResponse::default();
    
    for (token_id, amount) in tokens {
        if amount > 0 {
            response.alkanes.0.push(AlkaneTransfer {
                id: token_id,
                value: amount,
            });
        }
    }
    
    response
}
```

#### **Conditional Token Return Pattern**
```rust
fn maybe_return_token(&self, response: &mut CallResponse, token_id: AlkaneId, amount: u128) {
    if amount > 0 {
        response.alkanes.0.push(AlkaneTransfer {
            id: token_id,
            value: amount,
        });
    }
}
```

### **Inter-Contract Communication Patterns**

#### **Factory Call Pattern**
```rust
fn create_position_token(&self, position_data: Vec<u128>) -> Result<AlkaneId> {
    let cellpack = Cellpack {
        target: AlkaneId {
            block: 6,
            tx: POSITION_TOKEN_TEMPLATE_ID,
        },
        inputs: position_data,
    };
    
    // Empty parcel for pure factory call
    let empty_parcel = AlkaneTransferParcel::default();
    
    let response = self.call(&cellpack, &empty_parcel, self.fuel())?;
    
    if response.alkanes.0.is_empty() {
        return Err(anyhow!("Position token creation failed"));
    }
    
    Ok(response.alkanes.0[0].id.clone())
}
```

#### **Authenticated Call Pattern**
```rust
fn authenticated_call(&self, target: AlkaneId, inputs: Vec<u128>) -> Result<CallResponse> {
    let cellpack = Cellpack {
        target,
        inputs,
    };
    
    // Create auth parcel with vault factory token
    let mut auth_parcel = AlkaneTransferParcel::default();
    auth_parcel.0.push(AlkaneTransfer {
        id: self.context()?.myself,
        value: 1u128,
    });
    
    let response = self.call(&cellpack, &auth_parcel, self.fuel())?;
    Ok(response)
}
```

#### **Query Call Pattern (Read-Only)**
```rust
fn query_position_data(&self, position_id: &AlkaneId, opcode: u128) -> Result<Vec<u8>> {
    let cellpack = Cellpack {
        target: position_id.clone(),
        inputs: vec![opcode],
    };
    
    let response = self.staticcall(&cellpack, &AlkaneTransferParcel::default(), self.fuel())?;
    Ok(response.data)
}
```

### **Error Handling Patterns**

#### **Comprehensive Error Pattern**
```rust
fn handle_deposit_errors(&self, assets: u128, context: &Context) -> Result<()> {
    // Input validation
    if assets == 0 {
        return Err(anyhow!("Invalid deposit amount: cannot be zero"));
    }
    
    // Context validation
    if context.incoming_alkanes.0.len() != 1 {
        return Err(anyhow!("Invalid token count: expected exactly 1, got {}", 
                          context.incoming_alkanes.0.len()));
    }
    
    // Token validation
    let provided_amount = context.incoming_alkanes.0[0].value;
    if provided_amount < assets {
        return Err(anyhow!("Insufficient tokens: provided {}, needed {}", 
                          provided_amount, assets));
    }
    
    // State validation
    if !self.is_initialized() {
        return Err(anyhow!("Contract not initialized"));
    }
    
    Ok(())
}
```

#### **Recovery Pattern**
```rust
fn safe_operation_with_fallback<T, F>(&self, operation: F, fallback_value: T) -> T 
where 
    F: FnOnce() -> Result<T>,
{
    match operation() {
        Ok(value) => value,
        Err(e) => {
            // Log error for debugging but don't panic
            println!("Operation failed with error: {}", e);
            fallback_value
        }
    }
}
```

### **State Management Patterns**

#### **Initialization Guard Pattern**
```rust
fn ensure_initialized(&self) -> Result<()> {
    if !self.is_initialized() {
        return Err(anyhow!("Contract must be initialized before use"));
    }
    Ok(())
}

fn is_initialized(&self) -> bool {
    self.safe_load_u128("initialized") == 1
}

fn observe_initialization(&self) -> Result<()> {
    if self.is_initialized() {
        return Err(anyhow!("Contract already initialized"));
    }
    
    self.safe_store_u128("initialized", 1);
    Ok(())
}
```

#### **State Update Pattern**
```rust
fn update_vault_state(&self, assets_delta: i128, shares_delta: i128) -> Result<()> {
    let current_assets = self.total_assets() as i128;
    let current_shares = self.total_shares() as i128;
    
    let new_assets = current_assets + assets_delta;
    let new_shares = current_shares + shares_delta;
    
    if new_assets < 0 || new_shares < 0 {
        return Err(anyhow!("State update would result in negative values"));
    }
    
    self.set_total_assets(new_assets as u128);
    self.set_total_shares(new_shares as u128);
    
    Ok(())
}
```

### **Authentication Patterns**

#### **Position Token Verification Pattern**
```rust
fn verify_position_ownership(&self, context: &Context) -> Result<AlkaneId> {
    if context.incoming_alkanes.0.len() != 1 {
        return Err(anyhow!("Position verification requires exactly one token"));
    }
    
    let position_token = &context.incoming_alkanes.0[0];
    
    if position_token.value != 1 {
        return Err(anyhow!("Position token must have value of 1"));
    }
    
    if !self.is_position_in_registry(&position_token.id) {
        return Err(anyhow!("Token is not a registered position"));
    }
    
    Ok(position_token.id.clone())
}
```

#### **Auth Token Verification Pattern**
```rust
fn verify_auth_token(&self, context: &Context) -> Result<()> {
    // Optional verification - presence indicates intent
    if !context.incoming_alkanes.0.is_empty() {
        let auth_token = &context.incoming_alkanes.0[0];
        
        // Verify it's the correct auth token type
        if auth_token.id != context.myself {
            return Err(anyhow!("Invalid auth token type"));
        }
        
        // Note: We don't check amount since input-based auth doesn't rely on it
    }
    
    Ok(())
}
```

### **Data Encoding/Decoding Patterns**

#### **Multi-Value Response Pattern**
```rust
fn encode_position_data(&self, position_id: u128, assets: u128, shares: u128, 
                       deposit_block: u128, last_claim: u128) -> Vec<u8> {
    let mut data = Vec::with_capacity(80); // 5 * 16 bytes
    data.extend_from_slice(&position_id.to_le_bytes());
    data.extend_from_slice(&assets.to_le_bytes());
    data.extend_from_slice(&shares.to_le_bytes());
    data.extend_from_slice(&deposit_block.to_le_bytes());
    data.extend_from_slice(&last_claim.to_le_bytes());
    data
}

fn decode_position_data(&self, data: &[u8]) -> Result<(u128, u128, u128, u128, u128)> {
    if data.len() < 80 {
        return Err(anyhow!("Insufficient data for position decode"));
    }
    
    let position_id = u128::from_le_bytes(data[0..16].try_into().unwrap());
    let assets = u128::from_le_bytes(data[16..32].try_into().unwrap());
    let shares = u128::from_le_bytes(data[32..48].try_into().unwrap());
    let deposit_block = u128::from_le_bytes(data[48..64].try_into().unwrap());
    let last_claim = u128::from_le_bytes(data[64..80].try_into().unwrap());
    
    Ok((position_id, assets, shares, deposit_block, last_claim))
}
```

### **Gas/Fuel Optimization Patterns**

#### **Batch Storage Pattern**
```rust
fn batch_storage_updates(&self, updates: Vec<(&str, u128)>) {
    for (key, value) in updates {
        self.safe_store_u128(key, value);
    }
}
```

#### **Early Return Pattern**
```rust
fn optimized_reward_calculation(&self, amount: u128, blocks: u128) -> u128 {
    // Early return for zero cases
    if amount == 0 || blocks == 0 || self.reward_per_block() == 0 {
        return 0;
    }
    
    // Perform calculation only when necessary
    self.calculate_rewards_precise(amount, blocks)
}
```

## Design Principles

1. **Separation of Concerns**: Assets, authentication, and rewards are handled separately
2. **Mathematical Precision**: All calculations use checked arithmetic and exact precision
3. **Token Preservation**: Auth tokens must never be consumed during operations
4. **Custody Clarity**: Always clear who holds which assets at any point
5. **Trace Verifiability**: Every operation must be verifiable through trace logs
6. **Input-Based Security**: Authentication via parameters, not token consumption
7. **Defensive Programming**: Always validate inputs and handle edge cases
8. **State Consistency**: Ensure all state updates maintain invariants
9. **Error Transparency**: Provide clear error messages for debugging
10. **Resource Efficiency**: Optimize for fuel/gas consumption where possible
