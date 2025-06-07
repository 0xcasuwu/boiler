# Vault Contract Integration Specification

## Overview
This specification defines the complete integration between the OYL frontend and the boiler vault contracts, replacing all mock implementations with actual smart contract calls.

## Contract Architecture

### VaultFactory Contract (alkanes/alk4626-vault-factory)
**Core Principles:**
- **True Custody**: Vault holds ALL deposited assets
- **Position Authentication**: Position tokens provide access without holding assets
- **Block-based Rewards**: Rewards calculated using block heights
- **Input-based Authentication**: Parameters drive operations, not edict consumption
- **Single-point Fee Extraction**: Fees collected on withdrawal only

**Contract Opcodes:**
```rust
Initialize (0)      → Set up vault with tokens, rates, fees
Deposit (1)         → Deposit assets, mint position token
Withdraw (2)        → Process position, return assets + rewards
WithdrawFees (4)    → Fee collection (owner only)
GetTotalAssets (10) → Current vault asset total
GetTotalShares (11) → Current share total
GetPositionCount (12) → Number of positions
GetPositionById (13) → Position token by ID
GetFeePercentage (14) → Current fee rate
CalculateRewards (20) → Reward calculation
ConvertToShares (21) → Asset to share conversion
ConvertToAssets (22) → Share to asset conversion
```

### PositionToken Contract (alkanes/alk4626-position-token)
**Authentication Token**: Purely for tracking and authentication, holds no assets

**Contract Opcodes:**
```rust
Initialize (0)         → Set up position with vault reference
UpdateLastClaimBlock (4) → Update claim tracking (vault-only)
UpdateCurrentAssets (5)  → Update asset tracking (vault-only)
GetPositionId (10)      → Position identifier
GetInitialAssets (11)   → Original deposit amount
GetCurrentAssets (12)   → Current asset value
GetShares (13)          → Share allocation
GetDepositBlock (14)    → When position was created
GetLastClaimBlock (15)  → Last reward claim block
CalculateBlocksStaked (20) → Time staked calculation
GetPendingRewards (21)  → Unclaimed rewards
GetPositionValue (22)   → Current position value
GetAllDetails (23)      → All position data at once
GetDepositTokenId (24)  → Original deposit token
```

## Frontend Integration Implementation

### 1. SDK Integration Layer (`lib/oyl/vault/fetch.ts`)

**Contract Call Infrastructure:**
```typescript
interface ContractCall {
  target: AlkaneId;
  opcode: number;
  inputs: u128[];
  requiresAuth?: boolean;
}

interface VaultResponse {
  success: boolean;
  data?: any;
  error?: string;
  transactionId?: string;
}
```

**Core Functions to Implement:**
1. `executeVaultCall(call: ContractCall): Promise<VaultResponse>`
2. `fetchVaultDetails(vaultId: string): Promise<VaultDetails>`
3. `executeVaultDeposit(vaultAddress: string, amount: number, userAddress: string)`
4. `executeVaultWithdraw(positionTokenId: string, userAddress: string)`
5. `claimPositionRewards(positionTokenId: string, userAddress: string)`
6. `fetchUserPositions(userAddress: string): Promise<PositionTokenDetails[]>`
7. `getVaultRealTimeData(vaultAddress: string)`
8. `calculateDepositPreview(vaultAddress: string, amount: number)`

### 2. State Management Integration (`context/VaultContext.tsx`)

**Real Contract Integration:**
- Replace all mock implementations with actual contract calls
- Handle contract-specific error states
- Implement proper transaction confirmation flows
- Add real-time data synchronization

**Key Implementation Areas:**
1. **refreshVaultData()**: Query contract state directly
2. **executeVaultOperation()**: Handle actual transactions
3. **Error Handling**: Contract-specific error parsing
4. **State Synchronization**: Real-time updates after operations

### 3. Component Integration

**Data Flow:**
```
User Action → VaultContext → SDK Layer → Smart Contract
                ↓
Contract Response → SDK Layer → VaultContext → UI Update
```

**Components Requiring Updates:**
- `VaultInput`: Real deposit/withdraw operations
- `FindVaults`: Live vault discovery and metrics
- `VaultPreview`: Actual preview calculations
- `PositionDashboard`: Real position tracking
- `VaultRow`: Live vault statistics

## Implementation Strategy

### Phase 1: SDK Foundation
1. Create contract call infrastructure
2. Implement basic opcode calling system
3. Add error handling and response parsing
4. Test basic contract connectivity

### Phase 2: Core Operations
1. Implement deposit flow with position token creation
2. Implement withdraw flow with reward calculation
3. Add position tracking and management
4. Implement real-time data fetching

### Phase 3: Advanced Features
1. Add reward calculation and projection
2. Implement fee management
3. Add vault analytics and metrics
4. Optimize performance and caching

### Phase 4: UI Integration
1. Replace all mock data with real contract data
2. Add proper loading and error states
3. Implement transaction confirmation flows
4. Add real-time updates

## Contract Call Patterns

### Deposit Flow
```typescript
// 1. Validate deposit parameters
// 2. Call VaultFactory.Deposit (opcode 1) with assets
// 3. Receive position token in response
// 4. Update frontend state with new position
```

### Withdraw Flow
```typescript
// 1. Authenticate with position token
// 2. Call VaultFactory.Withdraw (opcode 2) 
// 3. Receive assets + rewards
// 4. Update position status to withdrawn
```

### Real-time Data Updates
```typescript
// 1. Call VaultFactory.GetTotalAssets (opcode 10)
// 2. Call VaultFactory.GetTotalShares (opcode 11)
// 3. Calculate current APY from reward rates
// 4. Update UI with live data
```

## Error Handling Strategy

### Contract Error Types
1. **Insufficient Assets**: Not enough tokens for operation
2. **Reward Pool Exhausted**: No rewards available for new deposits
3. **Invalid Authentication**: Position token validation failed
4. **Fee Calculation Errors**: Fee percentage or calculation issues
5. **Block Height Issues**: Timing-related calculation problems

### Frontend Error Handling
```typescript
interface VaultError {
  type: 'INSUFFICIENT_ASSETS' | 'REWARD_POOL_EXHAUSTED' | 'INVALID_AUTH' | 'FEE_ERROR' | 'BLOCK_ERROR';
  message: string;
  contractError?: string;
  retryable: boolean;
}
```

## Performance Considerations

### Caching Strategy
- Cache vault metadata for 5 minutes
- Real-time updates for user positions
- Background refresh for vault statistics
- Optimistic updates for user operations

### Transaction Management
- Queue operations to prevent conflicts
- Provide clear transaction status
- Handle transaction failures gracefully
- Implement retry mechanisms for network issues

## Security Considerations

### Input Validation
- Validate all amounts and addresses
- Check vault and position token authenticity
- Verify contract responses
- Sanitize user inputs

### Authentication
- Position token verification
- User address validation
- Contract address verification
- Prevent replay attacks

## Testing Strategy

### Unit Tests
- Individual contract call functions
- Data transformation utilities
- Error handling scenarios
- State management logic

### Integration Tests
- Complete deposit/withdraw flows
- Real-time data synchronization
- Error recovery scenarios
- Transaction confirmation flows

### End-to-End Tests
- Full user journey testing
- Cross-browser compatibility
- Performance under load
- Security vulnerability testing

## Deployment Checklist

### Pre-deployment
- [ ] All mock implementations replaced
- [ ] Contract addresses configured
- [ ] Error handling tested
- [ ] Performance optimized
- [ ] Security audit completed

### Post-deployment
- [ ] Monitor contract interactions
- [ ] Track error rates
- [ ] Verify transaction success rates
- [ ] Monitor performance metrics
- [ ] Collect user feedback

## Success Metrics

### Technical Metrics
- Transaction success rate > 98%
- Average response time < 2 seconds
- Error rate < 2%
- Zero security incidents

### User Experience Metrics
- Position creation success rate
- Reward claim success rate
- UI responsiveness
- User satisfaction scores

This specification provides the complete blueprint for transitioning from mock implementations to actual vault contract integration, ensuring a production-ready system that fully leverages the boiler contract architecture.
