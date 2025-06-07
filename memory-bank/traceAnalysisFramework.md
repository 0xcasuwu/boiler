# Trace Analysis Framework - Contract Call to JSON Mapping

## Overview

This document establishes the paradigm for translating raw blockchain traces into human-readable JSON structures. Based on comprehensive analysis of the vault system's trace patterns, this framework enables granular understanding of contract execution flow.

## Core Translation Paradigm

### Universal Contract Call Structure
```json
{
  "trace_analysis": {
    "operation_context": {
      "myself": "contract_being_called",
      "caller": "calling_contract", 
      "vout": "output_index",
      "fuel": "gas_allocated"
    },
    "input_parameters": {
      "raw_inputs": "[opcode, param1, param2, ...]",
      "decoded_parameters": {
        "opcode": "operation_type",
        "parameters": "decoded_based_on_opcode"
      }
    },
    "token_transfers": {
      "incoming": [{"id": "token_id", "amount": "amount"}],
      "outgoing": [{"id": "token_id", "amount": "amount"}]
    },
    "execution_result": {
      "status": "ReturnContext|RevertContext",
      "storage_changes": "key_value_pairs",
      "tokens_created": "new_token_list",
      "error_data": "error_message_if_failed"
    }
  }
}
```

## Operation Type Mappings

### 1. Contract Deployment (Block 0)

**Trace Evidence:**
```
CreateAlkane(AlkaneId { block: 4, tx: 797 }), EnterCall(TraceContext { ... }), ReturnContext(...)
```

**JSON Translation:**
```json
{
  "operation_type": "contract_deployment",
  "block": 0,
  "contracts_deployed": [
    {
      "contract_id": {"block": 4, "tx": 797},
      "contract_type": "free_mint_token",
      "deployment_status": "success",
      "fuel_allocated": 33333333,
      "fuel_used": 0,
      "storage_initialized": false
    },
    {
      "contract_id": {"block": 4, "tx": 889},
      "contract_type": "position_token_factory", 
      "deployment_status": "success",
      "fuel_allocated": 49967356,
      "fuel_used": 0
    },
    {
      "contract_id": {"block": 4, "tx": 890},
      "contract_type": "vault_factory",
      "deployment_status": "success", 
      "fuel_allocated": 99872128,
      "fuel_used": 0
    }
  ]
}
```

### 2. Token Contract Initialization (Block 1)

**Trace Evidence:**
```
CreateAlkane(AlkaneId { block: 2, tx: 1 }), EnterCall(...inputs: [0, 100, 5000, 10000, 4276545, 0, 4276545...]...), ReturnContext(...AlkaneTransfer { id: AlkaneId { block: 2, tx: 1 }, value: 100 }...)
```

**Parameter Decoding:**
- `inputs[0] = 0` → Initialize opcode
- `inputs[1] = 100` → Initial supply  
- `inputs[2] = 5000` → Value per mint
- `inputs[3] = 10000` → Supply cap
- `inputs[4] = 4276545` → Token name ("AAA")
- `inputs[5] = 0` → Token data
- `inputs[6] = 4276545` → Token symbol ("AAA")

**JSON Translation:**
```json
{
  "operation_type": "token_initialization",
  "block": 1,
  "token_contract": {
    "id": {"block": 2, "tx": 1},
    "initialization_parameters": {
      "opcode": 0,
      "initial_supply": 100,
      "value_per_mint": 5000,
      "supply_cap": 10000,
      "name": "AAA",
      "symbol": "AAA"
    },
    "tokens_created": 100,
    "storage_state": {
      "/name": "AAA",
      "/symbol": "AAA", 
      "/totalsupply": 100,
      "/value-per-mint": 5000,
      "/initialized": true,
      "/cap": 10000
    }
  }
}
```

### 3. Vault Initialization (Block 3) - Critical Parameter Validation

**Trace Evidence:**
```
EnterCall(TraceContext { inner: Context { incoming_alkanes: AlkaneTransferParcel([AlkaneTransfer { id: AlkaneId { block: 2, tx: 1 }, value: 5000 }]), inputs: [0, 2, 1, 2, 1, 500000, 3, 5000, 0] }...)
```

**Critical Parameter Mapping (The $50 Lesson):**
- `inputs[0] = 0` → Initialize opcode
- `inputs[1-2] = [2, 1]` → Deposit token ID (AlkaneId)
- `inputs[3-4] = [2, 1]` → Reward token ID (AlkaneId)  
- `inputs[5] = 500000` → Reward per block
- `inputs[6] = 3` → Start block
- `inputs[7] = 5000` → **CRITICAL: Preloaded rewards (must match incoming tokens exactly!)**
- `inputs[8] = 0` → Fee percentage

**JSON Translation:**
```json
{
  "operation_type": "vault_initialization",
  "block": 3,
  "parameter_validation": {
    "preloaded_rewards_parameter": 5000,
    "incoming_tokens_actual": 5000,
    "validation_status": "EXACT_MATCH_SUCCESS",
    "validation_rule": "preloaded_rewards_parameter MUST equal incoming_tokens_actual"
  },
  "vault_contract": {
    "id": {"block": 4, "tx": 890},
    "initialization_parameters": {
      "deposit_token_id": {"block": 2, "tx": 1},
      "reward_token_id": {"block": 2, "tx": 1}, 
      "reward_per_block": 500000,
      "start_block": 3,
      "preloaded_rewards": 5000,
      "fee_percentage": 0
    },
    "incoming_tokens": [
      {"token_id": {"block": 2, "tx": 1}, "amount": 5000}
    ],
    "auth_tokens_issued": [
      {"token_id": {"block": 4, "tx": 890}, "amount": 1}
    ],
    "storage_state": {
      "/remaining_rewards": 5000,
      "/total_reward_pool": 5000, 
      "/reward_per_block": 500000,
      "/start_block": 3,
      "/position_count": 0,
      "/total_shares": 0,
      "/total_assets": 0,
      "/initialized": true
    }
  }
}
```

### 4. Token Minting Operations (Block 2, 4, 11, 43)

**Trace Evidence:**
```
EnterCall(TraceContext { inner: Context { myself: AlkaneId { block: 2, tx: 1 }, inputs: [77, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] }...), ReturnContext(...AlkaneTransfer { id: AlkaneId { block: 2, tx: 1 }, value: 5000 }...)
```

**Parameter Decoding:**
- `inputs[0] = 77` → MintTokens opcode

**JSON Translation:**
```json
{
  "operation_type": "token_mint",
  "block": 2,
  "mint_operation": {
    "opcode": 77,
    "token_contract": {"block": 2, "tx": 1},
    "tokens_minted": 5000,
    "mint_counter_increment": 1,
    "total_supply_after": 5100
  },
  "storage_changes": {
    "/totalsupply": {"before": 100, "after": 5100},
    "/minted": {"before": 0, "after": 1},
    "/tx-hashes/[hash]": true
  }
}
```

### 5. Deposit Operations (Block 10, 20, 30)

**Trace Evidence:**
```
EnterCall(...inputs: [1, 1000, 0, 0, 0, 0, 0, 0, 0, 0, 0]...), CreateAlkane(AlkaneId { block: 2, tx: 2 }), EnterCall(...inputs: [0, 0, 1000, 1000, 10, 2, 1]...)
```

**Parameter Decoding:**
- **Vault Call**: `inputs[0] = 1` (deposit opcode), `inputs[1] = 1000` (amount)
- **Position Token Creation**: `inputs[0] = 0` (init), `inputs[1] = 0` (position_id), `inputs[2] = 1000` (initial_assets), `inputs[3] = 1000` (current_assets), `inputs[4] = 10` (deposit_block), `inputs[5-6] = [2, 1]` (deposit_token_id)

**JSON Translation:**
```json
{
  "operation_type": "deposit",
  "block": 10,
  "deposit_request": {
    "opcode": 1,
    "amount_requested": 1000,
    "incoming_tokens": 10000000,
    "user": "Deposit 1"
  },
  "position_token_creation": {
    "position_contract_id": {"block": 2, "tx": 2},
    "position_parameters": {
      "position_id": 0,
      "initial_assets": 1000,
      "current_assets": 1000, 
      "deposit_block": 10,
      "deposit_token_id": {"block": 2, "tx": 1}
    },
    "position_storage": {
      "/position_id": 0,
      "/shares": 1000,
      "/vault_id": {"block": 4, "tx": 890},
      "/current_assets": 1000,
      "/deposit_token_id": {"block": 2, "tx": 1},
      "/last_claim_block": 10,
      "/deposit_block": 10,
      "/initial_assets": 1000
    }
  },
  "vault_state_changes": {
    "total_shares": {"before": 0, "after": 1000},
    "total_assets": {"before": 0, "after": 1000},
    "position_count": {"before": 0, "after": 1},
    "positions_registry": {
      "positions_by_id/00000000000000000000000000000000": {"block": 2, "tx": 2}
    }
  },
  "tokens_issued": [
    {"token_id": {"block": 2, "tx": 2}, "amount": 1, "type": "position_token"}
  ]
}
```

### 6. Reward Pool Exhaustion (Block 60) - Economic Protection

**Trace Evidence:**
```
EnterCall(...inputs: [1, 1000, 0, 0, 0, 0, 0, 0, 0, 0, 0]...), RevertContext(TraceResponse { ...data: [ALKANES: revert: Error: Reward pool exhausted - no rewards available for new deposits]...})
```

**JSON Translation:**
```json
{
  "operation_type": "deposit_attempt",
  "block": 60, 
  "deposit_request": {
    "opcode": 1,
    "amount_requested": 1000,
    "user": "New User"
  },
  "execution_result": {
    "status": "REVERTED",
    "error_type": "reward_pool_exhausted",
    "error_message": "Reward pool exhausted - no rewards available for new deposits",
    "protection_mechanism": "WORKING_AS_DESIGNED"
  },
  "economic_protection": {
    "remaining_rewards": 0,
    "new_deposits_blocked": true,
    "refund_processed": true,
    "refund_address": "2N94cRAKAA8mjvVNPCUF4Pp9RgcxSUJFLqF"
  },
  "trace_validation": {
    "revert_context_detected": true,
    "error_data_decoded": true,
    "ascii_error_message": "ALKANES: revert: Error: Reward pool exhausted - no rewards available for new deposits"
  }
}
```

### 7. Debug Query Operations (Block 55)

**Trace Evidence:**
```
EnterCall(...inputs: [99, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]...), ReturnContext(...data: [57, 48, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]...)
```

**Parameter Decoding:**
- `inputs[0] = 99` → TestPing opcode

**JSON Translation:**
```json
{
  "operation_type": "debug_query",
  "block": 55,
  "debug_request": {
    "opcode": 99,
    "query_type": "test_ping"
  },
  "execution_result": {
    "status": "SUCCESS",
    "response_data": [57, 48, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    "decoded_response": 12345,
    "encoding": "u128_little_endian"
  },
  "vault_health_check": {
    "contract_responsive": true,
    "post_withdrawal_status": "operational"
  }
}
```

## Storage State Decoding Patterns

### Little-Endian U128 Decoder
```json
{
  "decoder_examples": {
    "/remaining_rewards": {
      "raw_bytes": "[136, 19, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]",
      "decoded_value": 5000,
      "calculation": "136 + (19 << 8) = 136 + 4864 = 5000",
      "encoding": "u128_little_endian"
    },
    "/reward_per_block": {
      "raw_bytes": "[32, 161, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]", 
      "decoded_value": 500000,
      "calculation": "32 + (161 << 8) + (7 << 16) = 32 + 41216 + 458752 = 500000",
      "encoding": "u128_little_endian"
    },
    "/total_reward_pool": {
      "raw_bytes": "[128, 150, 152, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]",
      "decoded_value": 10000000,
      "calculation": "128 + (150 << 8) + (152 << 16) = 128 + 38400 + 9961472 = 10000000",
      "encoding": "u128_little_endian"
    }
  }
}
```

### ASCII String Decoder
```json
{
  "string_decoder_examples": {
    "/name": {
      "raw_bytes": "[65, 65, 65]",
      "decoded_value": "AAA",
      "encoding": "ascii_string"
    },
    "/symbol": {
      "raw_bytes": "[85, 85, 85]", 
      "decoded_value": "UUU",
      "encoding": "ascii_string"
    }
  }
}
```

### AlkaneId Decoder
```json
{
  "alkane_id_decoder": {
    "/deposit_token_id": {
      "raw_bytes": "[2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]",
      "decoded_value": {"block": 2, "tx": 1},
      "structure": "first_16_bytes=block, second_16_bytes=tx",
      "encoding": "alkane_id_32_bytes"
    }
  }
}
```

## Success/Failure Pattern Recognition

### Success Indicators
```json
{
  "success_patterns": {
    "trace_indicators": [
      "ReturnContext(TraceResponse { ... })",
      "AlkaneTransfer tokens returned",
      "StorageMap populated with expected keys",
      "data: [] (empty error data)",
      "fuel_used: 0 (efficient execution)"
    ],
    "test_output_indicators": [
      "✅ DEPOSIT SUCCESSFUL",
      "✅ WITHDRAWAL SUCCESSFUL", 
      "test result: ok. X passed; 0 failed"
    ]
  }
}
```

### Failure Indicators  
```json
{
  "failure_patterns": {
    "trace_indicators": [
      "RevertContext(TraceResponse { ... })",
      "data: [8, 195, 121, 160, ...] (error message bytes)",
      "alkanes: AlkaneTransferParcel([]) (no tokens returned)",
      "storage: StorageMap({}) (no storage changes)"
    ],
    "common_error_messages": {
      "parameter_mismatch": "Reward token amount (X) doesn't match preloaded_rewards parameter (Y)",
      "reward_exhaustion": "Reward pool exhausted - no rewards available for new deposits",
      "authentication_failure": "Must preload reward pool with tokens",
      "wasm_panic": "wasm unreachable instruction executed"
    },
    "test_output_indicators": [
      "❌ DEPOSIT REVERTED",
      "❌ WITHDRAWAL ERROR",
      "ALKANES: revert: Error:"
    ]
  }
}
```

## Comprehensive Trace Analysis Workflow

### Step 1: Identify Operation Type
```json
{
  "operation_identification": {
    "by_trace_pattern": {
      "CreateAlkane(...)": "contract_deployment",
      "EnterCall + CreateAlkane + EnterCall": "contract_creation_with_subcall",
      "EnterCall + ReturnContext": "simple_function_call",
      "EnterCall + RevertContext": "failed_function_call"
    },
    "by_opcode": {
      "0": "initialize",
      "1": "deposit", 
      "2": "withdraw",
      "77": "mint_tokens",
      "99": "test_ping"
    }
  }
}
```

### Step 2: Decode Parameters
```json
{
  "parameter_decoding": {
    "input_array_parsing": {
      "extract_opcode": "inputs[0]",
      "decode_based_on_opcode": "different_structure_per_operation",
      "validate_parameter_count": "ensure_correct_length"
    },
    "alkane_id_parsing": {
      "two_consecutive_u128": "[block_u128, tx_u128]",
      "storage_as_32_bytes": "16_bytes_block + 16_bytes_tx"
    }
  }
}
```

### Step 3: Analyze Token Flows
```json
{
  "token_flow_analysis": {
    "incoming_tokens": "incoming_alkanes field",
    "outgoing_tokens": "AlkaneTransferParcel in response",
    "token_validation": "match_incoming_with_parameters",
    "custody_tracking": "who_holds_what_after_operation"
  }
}
```

### Step 4: Decode Storage Changes
```json
{
  "storage_analysis": {
    "decode_all_storage_keys": "convert_bytes_to_meaningful_values",
    "track_state_transitions": "before_and_after_values",
    "validate_consistency": "ensure_storage_makes_sense"
  }
}
```

### Step 5: Validate Business Logic
```json
{
  "business_logic_validation": {
    "mathematical_checks": "verify_calculations_correct",
    "economic_invariants": "total_shares_vs_total_assets_consistency",
    "security_validations": "authentication_and_authorization_checks",
    "state_consistency": "all_storage_updates_atomic"
  }
}
```

## Key Insights from Trace Analysis

### 1. Parameter Validation is Critical
- **The $50 Lesson**: `preloaded_rewards` parameter must EXACTLY match `incoming_tokens` amount
- All Alkane contracts validate parameters against actual token transfers
- Mismatches cause immediate reverts with specific error messages

### 2. Token Transfer Mechanics
- Referencing an outpoint in transaction input transfers ALL tokens at that outpoint
- Edict amounts must match what's actually being transferred
- Partial transfers require token splitting operations first

### 3. Storage Consistency Patterns
- Successful operations update ALL related storage keys atomically
- Failed operations leave storage unchanged (atomic rollback)
- Storage keys use consistent naming patterns (e.g., `/remaining_rewards`)

### 4. Authentication Patterns
- Auth tokens prove successful completion of operations
- Position tokens authenticate ownership of vault positions
- Input-based authentication (parameters) vs edict-based (token transfers)

### 5. Economic Protection Mechanisms
- Reward pool exhaustion blocks new deposits (prevents over-commitment)
- Fee extraction maintains vault custody of collected fees
- Share-based reward distribution ensures proportional fairness

This framework enables complete interpretation of any blockchain trace in this system, providing the granular understanding needed for debugging, verification, and system comprehension.
