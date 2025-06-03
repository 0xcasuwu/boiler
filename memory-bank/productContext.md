# Product Context - ALK4626 Vault Factory

## Why This Project Exists
The ALK4626 vault factory solves critical problems in decentralized finance (DeFi) vault management by creating a true custody architecture where assets and authentication are properly separated.

## Problems Being Solved

### 1. **Auth Token Consumption Crisis**
- **Problem**: Traditional edict-based authentication consumes auth tokens at protocol level
- **Impact**: Vault operators lose control tokens during fee withdrawals
- **Solution**: Input-based authentication eliminates edict dependency completely

### 2. **Unclear Asset Custody**
- **Problem**: Traditional vaults mix user assets with authentication mechanisms
- **Impact**: Difficult to prove who holds what assets
- **Solution**: Clear separation where vault holds fees, position tokens represent users

### 3. **Mathematical Precision Requirements**
- **Problem**: Fee calculations must be exact to avoid exploitation
- **Impact**: Rounding errors can be exploited or cause losses
- **Solution**: Precise basis point calculations (50 basis points = 0.5%)

## How the Product Works

### **User Journey - Deposit**
1. User sends underlying tokens to vault factory
2. Vault calculates shares based on current exchange rate
3. Vault factory creates position token with authentication capability
4. User receives position token representing their vault stake
5. Vault retains full custody of underlying assets

### **User Journey - Withdrawal**
1. User sends position token to vault factory for authentication
2. Vault calculates current asset value + accumulated rewards
3. Vault applies withdrawal fee (e.g., 0.5%)
4. Vault keeps fee tokens in its custody
5. User receives net amount (assets + rewards - fees)
6. Position token is updated or burned

### **Vault Owner Journey - Fee Collection**
1. Vault owner sends single auth token to vault factory
2. Vault verifies auth token type and ownership
3. Vault transfers all collected fees to owner
4. Vault returns auth token perfectly preserved (1:1)
5. Vault resets internal fee counter to zero

## Value Propositions

### **For Users**
- **True Asset Representation**: Position tokens prove vault ownership without holding assets
- **Reward Accumulation**: Automatic reward calculation based on time and amount
- **Mathematical Precision**: Exact fee calculations prevent exploitation
- **Clean UX**: Simple deposit/withdrawal flow with clear custody model

### **For Vault Operators**
- **Perfect Auth Token Preservation**: Zero consumption during operations
- **Guaranteed Fee Collection**: Fees accumulate and can be withdrawn anytime
- **Operational Flexibility**: Single auth token provides full vault control
- **Audit Trail**: Complete trace logs of all custodial operations

### **For DeFi Ecosystem**
- **Custody Clarity**: Provable asset separation and ownership
- **Security Model**: Input-based authentication prevents token loss
- **Composability**: Clean interfaces for integration with other protocols
- **Standards Compliance**: ALK4626 compatibility for ecosystem interoperability

## Success Metrics
- ✅ **Zero auth token consumption** across all operations
- ✅ **Mathematical precision** in all fee and reward calculations
- ✅ **Complete custody separation** between vault and user assets
- ✅ **Trace log validation** of every custodial relationship
