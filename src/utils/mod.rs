
// Constants for yield calculations
pub const BASIS_POINTS_DENOMINATOR: u128 = 10000;
// Bitcoin produces ~1 block per 10 minutes = 6 blocks per hour
pub const BLOCKS_PER_DAY: u128 = 144; // 24 hours * 6 blocks per hour
pub const BLOCKS_PER_YEAR: u128 = BLOCKS_PER_DAY * 365; // 52,560 blocks
pub const YIELD_CALCULATION_DENOMINATOR: u128 = BASIS_POINTS_DENOMINATOR * BLOCKS_PER_YEAR;

// Constants for ERC4626 inflation attack protection
pub const VIRTUAL_SHARES: u128 = 1_000_000; // 1M virtual shares
pub const VIRTUAL_ASSETS: u128 = 1_000_000; // 1M virtual assets
pub const PRECISION_OFFSET: u8 = 3; // 3 decimal places offset

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
    /// Apply precision offset to assets when converting to shares
    fn apply_precision_offset(&self, assets: u128) -> Result<u128, &'static str> {
        let precision_factor = 10u128.pow(PRECISION_OFFSET as u32);
        assets
            .checked_mul(precision_factor)
            .ok_or("Precision offset multiplication overflow")
    }
    
    /// Remove precision offset from shares when converting to assets
    fn remove_precision_offset(&self, shares: u128) -> Result<u128, &'static str> {
        let precision_factor = 10u128.pow(PRECISION_OFFSET as u32);
        shares
            .checked_div(precision_factor)
            .ok_or("Precision offset division error")
    }
    
    /// Convert assets to shares with inflation attack protection
    fn convert_assets_to_shares(&self, assets: u128, total_assets: u128, total_supply: u128) 
        -> Result<u128, &'static str> {
        // Apply precision offset to increase share precision
        let assets_with_offset = self.apply_precision_offset(assets)?;
        
        // Use virtual offset for the calculation
        let adjusted_total_assets = total_assets + VIRTUAL_ASSETS;
        let adjusted_total_supply = total_supply + VIRTUAL_SHARES;
        
        // If the vault is empty (only has virtual assets/shares), use 1:1 ratio with precision offset
        if total_assets == 0 || total_supply == 0 {
            return Ok(assets_with_offset);
        }
        
        // shares = assets * adjustedTotalSupply / adjustedTotalAssets
        assets_with_offset
            .checked_mul(adjusted_total_supply)
            .ok_or("Convert to shares multiplication overflow")?
            .checked_div(adjusted_total_assets)
            .ok_or("Convert to shares division error")
    }
    
    /// Convert shares to assets with inflation attack protection
    fn convert_shares_to_assets(&self, shares: u128, total_assets: u128, total_supply: u128) 
        -> Result<u128, &'static str> {
        // Use virtual offset for the calculation
        let adjusted_total_assets = total_assets + VIRTUAL_ASSETS;
        let adjusted_total_supply = total_supply + VIRTUAL_SHARES;
        
        // If the vault is empty (only has virtual assets/shares), use 1:1 ratio with precision offset
        if total_supply == 0 {
            return self.remove_precision_offset(shares);
        }
        
        // assets = shares * adjustedTotalAssets / adjustedTotalSupply
        let assets_with_precision = shares
            .checked_mul(adjusted_total_assets)
            .ok_or("Convert to assets multiplication overflow")?
            .checked_div(adjusted_total_supply)
            .ok_or("Convert to assets division error")?;
            
        // Remove precision offset to get final assets
        self.remove_precision_offset(assets_with_precision)
    }
    
    /// Preview mint result (with ceiling division) with inflation attack protection
    fn preview_mint(&self, shares: u128, total_assets: u128, total_supply: u128) 
        -> Result<u128, &'static str> {
        // Use virtual offset for the calculation
        let adjusted_total_assets = total_assets + VIRTUAL_ASSETS;
        let adjusted_total_supply = total_supply + VIRTUAL_SHARES;
        
        // If the vault is empty (only has virtual assets/shares), use 1:1 ratio with precision offset
        if total_assets == 0 || total_supply == 0 {
            return self.remove_precision_offset(shares);
        }
        
        // assets = shares * adjustedTotalAssets / adjustedTotalSupply (rounded up for mint)
        let numerator = shares.checked_mul(adjusted_total_assets)
            .ok_or("Mint preview multiplication overflow")?;
        
        let assets_with_precision = ceil_div(numerator, adjusted_total_supply)?;
        
        // Remove precision offset to get final assets
        self.remove_precision_offset(assets_with_precision)
    }
    
    /// Preview withdraw result (with ceiling division) with inflation attack protection
    fn preview_withdraw(&self, assets: u128, total_assets: u128, total_supply: u128) 
        -> Result<u128, &'static str> {
        // Apply precision offset to increase share precision
        let assets_with_offset = self.apply_precision_offset(assets)?;
        
        // Use virtual offset for the calculation
        let adjusted_total_assets = total_assets + VIRTUAL_ASSETS;
        let adjusted_total_supply = total_supply + VIRTUAL_SHARES;
        
        // If the vault is empty (only has virtual assets/shares), use 1:1 ratio with precision offset
        if total_assets == 0 || total_supply == 0 {
            return Ok(assets_with_offset);
        }
        
        // shares = assets * adjustedTotalSupply / adjustedTotalAssets (rounded up for withdraw)
        let numerator = assets_with_offset.checked_mul(adjusted_total_supply)
            .ok_or("Withdraw preview multiplication overflow")?;
        
        ceil_div(numerator, adjusted_total_assets)
    }
}

impl<T> Conversion for T {}
