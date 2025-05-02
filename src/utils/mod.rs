use metashrew_support::index_pointer::KeyValuePointer; // Add this import

// Constants for yield calculations
pub const BASIS_POINTS_DENOMINATOR: u128 = 10000;
pub const SECONDS_PER_YEAR: u128 = 365 * 24 * 60 * 60;
pub const YIELD_CALCULATION_DENOMINATOR: u128 = BASIS_POINTS_DENOMINATOR * SECONDS_PER_YEAR;

/// Helper function to calculate ceil division
pub fn ceil_div(numerator: u128, denominator: u128) -> Result<u128, &'static str> {
    if denominator == 0 {
        return Err("Division by zero");
    }
    
    let remainder = numerator % denominator;
    let quotient = numerator / denominator;
    
    if remainder > 0 {
        quotient.checked_add(1).ok_or("Division result overflow")
    } else {
        Ok(quotient)
    }
}

/// Conversion trait for converting between assets and shares
pub trait Conversion {
    /// Convert assets to shares
    fn convert_assets_to_shares(&self, assets: u128, total_assets: u128, total_supply: u128) 
        -> Result<u128, &'static str> {
        // If the vault is empty, use 1:1 ratio
        if total_assets == 0 || total_supply == 0 {
            return Ok(assets);
        }
        
        // shares = assets * totalSupply / totalAssets
        assets
            .checked_mul(total_supply)
            .ok_or("Convert to shares multiplication overflow")?
            .checked_div(total_assets)
            .ok_or("Convert to shares division error")
    }
    
    /// Convert shares to assets
    fn convert_shares_to_assets(&self, shares: u128, total_assets: u128, total_supply: u128) 
        -> Result<u128, &'static str> {
        // If the vault is empty, use 1:1 ratio
        if total_supply == 0 {
            return Ok(shares);
        }
        
        // assets = shares * totalAssets / totalSupply
        shares
            .checked_mul(total_assets)
            .ok_or("Convert to assets multiplication overflow")?
            .checked_div(total_supply)
            .ok_or("Convert to assets division error")
    }
    
    /// Preview mint result (with ceiling division)
    fn preview_mint(&self, shares: u128, total_assets: u128, total_supply: u128) 
        -> Result<u128, &'static str> {
        // If the vault is empty, use 1:1 ratio
        if total_assets == 0 || total_supply == 0 {
            return Ok(shares);
        }
        
        // assets = shares * totalAssets / totalSupply (rounded up for mint)
        let numerator = shares.checked_mul(total_assets)
            .ok_or("Mint preview multiplication overflow")?;
        
        ceil_div(numerator, total_supply)
    }
    
    /// Preview withdraw result (with ceiling division)
    fn preview_withdraw(&self, assets: u128, total_assets: u128, total_supply: u128) 
        -> Result<u128, &'static str> {
        // If the vault is empty, use 1:1 ratio
        if total_assets == 0 || total_supply == 0 {
            return Ok(assets);
        }
        
        // shares = assets * totalSupply / totalAssets (rounded up for withdraw)
        let numerator = assets.checked_mul(total_supply)
            .ok_or("Withdraw preview multiplication overflow")?;
        
        ceil_div(numerator, total_assets)
    }
}

impl<T> Conversion for T {}
