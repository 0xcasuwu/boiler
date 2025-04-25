# SLOP: Smart Contract Launchpad for Orbital Payments

## Core Concept

SLOP implements a bond launchpad system where **orbital tokens themselves ARE the bonds** - not just a reference to them. This fundamental design choice eliminates the need for user address-based authentication, as possession of the orbital token is sufficient proof of bond ownership.

## Project Overview

| Component | Status | Description |
|-----------|--------|-------------|
| Core Implementation | ✅ Complete | Basic data models, factory pattern implementation, and orbital-is-bond model |
| Feature Enhancement | 🔶 In Progress | Adding batch operations, improving error handling, and collection management |
| Production Readiness | 📝 Planned | Security auditing, performance optimization, and integration testing |

## Key Technical Features

- **Orbital-IS-Bond Model**: Orbital tokens function as bonds themselves, with bond attributes and behaviors
- **Factory Pattern**: LaunchpadFactory creates and manages independent OrbitalBondCollection instances
- **Block-Based Time**: System uses block numbers instead of timestamps for maturity tracking
- **Context-Based Ownership**: Authentication based on orbital token possession, not user addresses
- **HashMap-Based State**: Efficient key-value storage for collections and bonds

## System Architecture

```
                  ┌─────────────────┐
                  │LaunchpadFactory │
                  └────────┬────────┘
                           │
                           │ creates
                           ▼
     ┌─────────────┬───────────────┬─────────────┐
     │             │               │             │
┌────▼─────┐ ┌─────▼─────┐   ┌─────▼─────┐ ┌─────▼─────┐
│Collection1│ │Collection2│   │Collection3│ │Collection4│
└────┬─────┘ └─────┬─────┘   └─────┬─────┘ └─────┬─────┘
     │             │               │             │
     │ mints       │ mints         │ mints       │ mints
     ▼             ▼               ▼             ▼
┌───────────┐ ┌───────────┐   ┌───────────┐ ┌───────────┐
│Orbital/   │ │Orbital/   │   │Orbital/   │ │Orbital/   │
│Bond       │ │Bond       │   │Bond       │ │Bond       │
└───────────┘ └───────────┘   └───────────┘ └───────────┘
```

## Primary User Flows

### Collection Creation

- Bond issuer requests new collection from LaunchpadFactory
- Factory creates collection with specified parameters
- Collection is registered and returned to issuer

### Bond Minting

- User deposits diesel tokens to mint a new bond
- System mints an orbital token that IS the bond
- Orbital/bond transferred to user

### Bond Redemption

- User presents orbital token
- System verifies maturity status
- Bond is redeemed, principal + interest transferred

## Current Focus Areas

1. **Enhanced Collection Management**: Improving tools for collection administration
2. **Batch Operations Support**: Adding ability to handle multiple bonds efficiently
3. **Error Handling Improvements**: Better error reporting and recovery options
4. **Advanced Testing**: Moving beyond core tests to performance, security, and integration testing

## Testing Robustness

The SLOP system now has a comprehensive testing infrastructure with several key components:

- **TestBlockContext**: Custom context that provides deterministic block heights for tests
- **51 Passing Tests**: Complete test coverage of core functionality
- **Time-Independent Testing**: Tests are completely reliable and reproducible
- **Edge Case Verification**: Thorough testing of bond lifecycle, maturity conditions, and interest calculations

The test suite provides strong guarantees for our architecture:

1. **Financial Accuracy**: Interest calculations and redemptions are precise
2. **State Transition Integrity**: Bonds correctly transition between states (Active → Mature → Redeemed)
3. **Temporal Logic**: Time-dependent operations execute correctly regardless of execution conditions
4. **Layer Isolation**: Each architectural layer operates independently with clean interfaces

## Key Technical Decisions

| Decision | Rationale |
|----------|-----------|
| Orbital-IS-Bond Model | Simplifies ownership model, eliminates complex mappings |
| Block-Based Time | More deterministic than timestamps, simpler to test |
| TestBlockContext Pattern | Makes tests reliable and deterministic |
| Feature Flag for Blockchain | Allows standalone testing without blockchain dependencies |
| HashMap-Based Storage | Efficient lookups, well-understood behavior |
| Factory Pattern | Better isolation between collections, more modular |

## Technology Stack

- **Rust**: Primary language for contract development
- **Serde**: For serialization and deserialization
- **Feature Flags**: For conditional blockchain functionality
- **HashMaps**: For efficient key-value storage
- **Custom Test Frameworks**: TestBlockContext for deterministic testing

## Known Limitations

- Currently only supports simple interest calculation
- Bonds must be redeemed in full, not partially
- Limited collection management features
- Basic error handling and reporting

## Next Steps

1. ~~Complete testing infrastructure~~ ✅
2. Enhance collection management tools
3. Implement batch operations
4. Improve error handling
5. Complete API documentation
6. Expand to advanced testing (performance, security, integration)

## Core Files Reference

- **projectbrief.md**: Foundation document defining core requirements and goals
- **productContext.md**: Why this project exists and how it should work
- **systemPatterns.md**: System architecture and design patterns
- **techContext.md**: Technologies used and development constraints
- **activeContext.md**: Current focus areas and recent decisions
- **progress.md**: Current status and what's left to build
