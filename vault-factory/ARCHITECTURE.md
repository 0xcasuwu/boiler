# Vault Factory Architecture

This document provides a visual overview of the vault factory architecture and its interaction patterns with position tokens.

## Contract Relationship Diagram

```mermaid
graph TD
    User[User] -->|Deposits assets| VF[Vault Factory]
    VF -->|Creates| PT[Position Token]
    VF -->|Registers| Registry[(Position Registry)]
    PT -->|Returns to| User
    User -->|Presents| PT
    PT -->|Authenticates w/| VF
    VF -->|Verifies against| Registry
    VF -->|Sends rewards| User
    VF -->|Updates state via callback| PT
    PT -->|Tracks| TimeData[Time & Amount Data]
```

## Authentication Flow

```mermaid
sequenceDiagram
    participant User
    participant PT as Position Token
    participant VF as Vault Factory
    
    User->>PT: Sends withdraw/claim request
    PT->>VF: Forwards request w/token auth
    Note over VF: Verifies PT is in registry
    VF->>PT: Calls back to update state w/token auth 
    Note over PT: Verifies VF token ID
    VF->>User: Sends reward tokens
```

## Reward Calculation

```mermaid
graph LR
    subgraph Reward Formula
    A[Amount] --> M((×))
    R[Reward Per Block] --> M
    B[Block Difference] --> M
    M --> D((/))
    P[Precision Factor] --> D
    D --> Rewards[Rewards]
    end
```

## Data Flow Optimization

```mermaid
graph TD
    subgraph Before
    VF1[Vault Factory] -->|5 separate calls| PT1[Position Token]
    end
    
    subgraph After
    VF2[Vault Factory] -->|Single get_all_details call| PT2[Position Token]
    PT2 -->|Returns packed data| VF2
    end
```

## Position Token Storage

```mermaid
graph TD
    subgraph Position Token Storage
    PT[Position Token] --> PID[Position ID]
    PT --> CA[Current Assets]
    PT --> S[Shares]
    PT --> DB[Deposit Block]
    PT --> LCB[Last Claim Block]
    PT --> VID[Vault ID]
    end
```

This architecture follows the factory-child pattern found in the Alkane Pandas project but adapted for a reward vesting system.
