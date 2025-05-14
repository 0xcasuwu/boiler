# Inflation Attack Protection Implementation

## Overview

This document summarizes the implementation of inflation attack protection mechanisms in the yield vault. These mechanisms protect against various attack vectors that could manipulate the exchange rate between assets and shares.

## Protection Mechanisms

### 1. Virtual Offset

The virtual offset mechanism adds a constant amount to both the total assets and total shares when calculating exchange rates. This limits the impact of large donations or manipulations on the exchange rate.

```rust
// Implementation in convert_assets_to_tokens
let adjusted_total_assets = total_assets.checked_add(VIRTUAL_ASSETS)
    .ok_or("Virtual assets addition overflow")?;
let adjusted_total_supply = total_issuance.checked_add(VIRTUAL_SHARES)
    .ok_or("Virtual shares addition overflow")?;

// Calculate tokens based on the ratio with virtual offset
adjusted_total_supply
    .checked_mul(assets_with_precision)
    .ok_or("Math overflow in assets to tokens conversion")?
    .checked_div(adjusted_total_assets)
    .ok_or("Division by zero in assets to tokens conversion")
```

### 2. Precision Offset

The precision offset multiplies the input assets by a factor (e.g., 10^3) to ensure that even small deposits result in a non-zero number of shares. This prevents rounding to zero for small deposits.

```rust
// Apply precision offset
let precision_factor = 10u128.pow(PRECISION_OFFSET as u32); // 3 decimal places
let assets_with_precision = assets.checked_mul(precision_factor)
    .ok_or("Precision offset multiplication overflow")?;
```

### 3. Direct Donation Prevention

The implementation prevents direct donations by ensuring that all asset transfers are properly accounted for with corresponding share issuance. This is enforced by the deposit and redeem methods, which maintain the relationship between assets and shares.

## Test Cases

Four test cases verify the effectiveness of these protection mechanisms:

1. **Virtual Offset Protection**: Verifies that the virtual offset prevents an attacker from manipulating the exchange rate by making a small deposit followed by a large donation.

2. **Precision Offset**: Ensures that small deposits result in non-zero shares due to the precision offset.

3. **Direct Donation Prevention**: Confirms that the exchange rate is limited by the virtual offset, even when a large donation is made.

4. **Zero Amount Prevention**: Verifies that zero assets result in zero shares and vice versa.

## Implementation Details

### Constants

```rust
// Virtual offset constants
pub const VIRTUAL_SHARES: u128 = 1_000_000;
pub const VIRTUAL_ASSETS: u128 = 1_000_000;
pub const PRECISION_OFFSET: u8 = 3;
```

### Asset to Share Conversion

```rust
pub fn convert_assets_to_tokens(&self, assets: u128, total_assets: u128, total_issuance: u128) -> Result<u128, &'static str> {
    // Apply precision offset
    let precision_factor = 10u128.pow(PRECISION_OFFSET as u32); // 3 decimal places
    let assets_with_precision = assets.checked_mul(precision_factor)
        .ok_or("Precision offset multiplication overflow")?;
    
    // Use virtual offset for the calculation
    let adjusted_total_assets = total_assets.checked_add(VIRTUAL_ASSETS)
        .ok_or("Virtual assets addition overflow")?;
    let adjusted_total_supply = total_issuance.checked_add(VIRTUAL_SHARES)
        .ok_or("Virtual shares addition overflow")?;
    
    // Empty vault case - 1:1 ratio with precision offset
    if total_assets == 0 || total_issuance == 0 {
        return Ok(assets_with_precision);
    }
    
    // Calculate tokens based on the ratio of assets to total_assets
    // tokens = assets * (virtual_shares + total_issuance) / (virtual_assets + total_assets)
    adjusted_total_supply
        .checked_mul(assets_with_precision)
        .ok_or("Math overflow in assets to tokens conversion")?
        .checked_div(adjusted_total_assets)
        .ok_or("Division by zero in assets to tokens conversion")
}
```

### Share to Asset Conversion

```rust
pub fn convert_tokens_to_assets(&self, tokens: u128, total_assets: u128, total_issuance: u128) -> Result<u128, &'static str> {
    // Remove precision offset
    let precision_factor = 10u128.pow(PRECISION_OFFSET as u32); // 3 decimal places
    
    // Use virtual offset for the calculation
    let adjusted_total_assets = total_assets.checked_add(VIRTUAL_ASSETS)
        .ok_or("Virtual assets addition overflow")?;
    let adjusted_total_supply = total_issuance.checked_add(VIRTUAL_SHARES)
        .ok_or("Virtual shares addition overflow")?;
    
    // Empty vault case - return 0 assets as there are none
    if total_issuance == 0 {
        return Ok(0);
    }
    
    // Calculate assets based on the ratio of tokens to total_issuance
    // assets = tokens * (virtual_assets + total_assets) / (virtual_shares + total_issuance)
    let assets_with_precision = adjusted_total_assets
        .checked_mul(tokens)
        .ok_or("Math overflow in tokens to assets conversion")?
        .checked_div(adjusted_total_supply)
        .ok_or("Division by zero in tokens to assets conversion")?;
        
    // Remove precision offset
    assets_with_precision
        .checked_div(precision_factor)
        .ok_or("Precision offset division error")
}
```

## Security Considerations

1. **Overflow Protection**: All arithmetic operations use checked math to prevent overflows.
2. **Division by Zero**: The implementation checks for division by zero scenarios.
3. **Rounding**: The precision offset ensures that small amounts don't round to zero.
4. **Exchange Rate Manipulation**: The virtual offset limits the impact of large donations on the exchange rate.

## References

- [ERC-4626: Tokenized Vault Standard](https://eips.ethereum.org/EIPS/eip-4626)
- [OpenZeppelin ERC4626 Implementation](https://github.com/OpenZeppelin/openzeppelin-contracts/blob/master/contracts/token/ERC20/extensions/ERC4626.sol)
- [Inflation Attacks on ERC4626](https://blog.openzeppelin.com/a-novel-defense-against-erc4626-inflation-attacks)
