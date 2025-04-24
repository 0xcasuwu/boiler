use std::time::{SystemTime, UNIX_EPOCH};
use anyhow::{anyhow, Result};

/// Scaling factor used to maintain precision in integer calculations
const SCALING_FACTOR: u128 = 1_000_000;

/// Pricing structure for the bond curve
#[derive(Clone, Debug)]
pub struct Pricing {
    /// Virtual input reserves (decays over time)
    pub virtual_input_reserves: u128,
    /// Virtual output reserves (constant)
    pub virtual_output_reserves: u128,
    /// Last update timestamp/block
    pub last_update: u64,
    /// Half-life in blocks
    pub half_life: u64,
    /// Level in basis points (0-10000)
    pub level_bips: u64,
}

/// Bond curve implementation for time-decay pricing
#[derive(Clone)]
pub struct BondCurve {
    /// Pricing parameters
    pub pricing: Pricing,
    /// Total debt (sum of all unredeemed bonds)
    pub total_debt: u128,
    /// Term for bond maturity in blocks
    pub term_blocks: u64,
}

impl BondCurve {
    /// Create a new bond curve with the given parameters
    pub fn new(
        virtual_input_reserves: u128,
        virtual_output_reserves: u128,
        half_life: u64,
        level_bips: u64,
        term_blocks: u64,
    ) -> Self {
        let current_block = Self::get_current_block();
        
        Self {
            pricing: Pricing {
                virtual_input_reserves,
                virtual_output_reserves,
                last_update: current_block,
                half_life,
                level_bips,
            },
            total_debt: 0,
            term_blocks,
        }
    }
    
    /// Get the current block number
    pub fn get_current_block() -> u64 {
        // In a real implementation, this would get the actual block number
        // For development purposes, we use timestamp divided by 10 (assuming 10 sec blocks)
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() / 10
    }
    
    /// Calculate the exponential decay to level
    /// 
    /// # Formula
    /// 
    /// z = x >> (elapsed / half_life)
    /// z -= z * (elapsed % half_life) / half_life >> 1
    /// z += (x - z) * level_bips / 10000
    pub fn exp_to_level(x: u128, elapsed: u64, half_life: u64, level_bips: u64) -> u128 {
        if half_life == 0 {
            return x; // Prevent division by zero
        }
        
        let mut z = x >> (elapsed / half_life);
        
        // Apply linear interpolation for partial half-life
        let partial = (elapsed % half_life) as u128;
        let half_life_u128 = half_life as u128;
        
        if z > 0 && partial > 0 && half_life_u128 > 0 {
            let adjustment = z.saturating_mul(partial).saturating_div(half_life_u128) >> 1;
            z = z.saturating_sub(adjustment);
        }
        
        // Apply level floor
        if level_bips > 0 && level_bips <= 10000 {
            let level_adjustment = (x.saturating_sub(z)).saturating_mul(level_bips as u128).saturating_div(10000);
            z = z.saturating_add(level_adjustment);
        }
        
        z
    }
    
    /// Calculate the current spot price
    /// 
    /// # Formula
    /// 
    /// price = decayed_virtual_input * SCALING_FACTOR / (available_debt + virtual_output)
    pub fn get_current_price(&self, available_debt: u128) -> u128 {
        let elapsed = Self::get_current_block().saturating_sub(self.pricing.last_update);
        
        let decayed_input = Self::exp_to_level(
            self.pricing.virtual_input_reserves,
            elapsed,
            self.pricing.half_life,
            self.pricing.level_bips
        );
        
        let denominator = available_debt.saturating_add(self.pricing.virtual_output_reserves);
        
        if denominator == 0 {
            return 0; // Prevent division by zero
        }
        
        decayed_input.saturating_mul(SCALING_FACTOR).saturating_div(denominator)
    }
    
    /// Calculate the amount of output tokens to mint for a given input amount
    /// 
    /// # Formula
    /// 
    /// output = input * (available_debt + virtual_output) / (decayed_virtual_input + input)
    pub fn get_amount_out(&self, input: u128, available_debt: u128) -> u128 {
        if input == 0 {
            return 0;
        }
        
        let elapsed = Self::get_current_block().saturating_sub(self.pricing.last_update);
        
        let decayed_input = Self::exp_to_level(
            self.pricing.virtual_input_reserves,
            elapsed,
            self.pricing.half_life,
            self.pricing.level_bips
        );
        
        let numerator = input.saturating_mul(available_debt.saturating_add(self.pricing.virtual_output_reserves));
        let denominator = decayed_input.saturating_add(input);
        
        if denominator == 0 {
            return 0; // Prevent division by zero
        }
        
        let result = numerator.saturating_div(denominator);
        
        // Ensure we return at least 1 for non-zero inputs
        if input > 0 && result == 0 {
            return 1;
        }
        
        result
    }
    
    /// Calculate the amount that can be redeemed from a bond
    /// 
    /// # Formula
    /// 
    /// redeemable = owed * min(elapsed, term) / term - redeemed
    pub fn get_redeem_amount(&self, owed: u128, redeemed: u128, creation_block: u64) -> Result<u128> {
        let current_block = Self::get_current_block();
        let elapsed = current_block.saturating_sub(creation_block);
        let elapsed_capped = if elapsed > self.term_blocks { self.term_blocks } else { elapsed };
        
        if self.term_blocks == 0 {
            return Err(anyhow!("Term blocks cannot be zero")); // Prevent division by zero
        }
        
        let total_redeemable = owed.saturating_mul(elapsed_capped as u128).saturating_div(self.term_blocks as u128);
        
        Ok(total_redeemable.saturating_sub(redeemed))
    }
    
    /// Purchase a bond
    /// Returns the amount of output tokens to be received when the bond fully matures
    pub fn purchase_bond(&mut self, input_amount: u128, available_debt: u128) -> Result<u128> {
        if input_amount == 0 {
            return Err(anyhow!("Input amount cannot be zero"));
        }
        
        let output_amount = self.get_amount_out(input_amount, available_debt);
        
        if output_amount == 0 {
            return Err(anyhow!("Calculated output amount is zero"));
        }
        
        // Update state
        self.pricing.virtual_input_reserves = self.pricing.virtual_input_reserves.saturating_add(input_amount);
        self.pricing.last_update = Self::get_current_block();
        self.total_debt = self.total_debt.saturating_add(output_amount);
        
        Ok(output_amount)
    }
    
    /// Redeem a bond
    /// Returns the amount redeemed
    pub fn redeem_bond(&mut self, owed: u128, redeemed: u128, creation_block: u64) -> Result<u128> {
        let redeemable = self.get_redeem_amount(owed, redeemed, creation_block)?;
        
        if redeemable == 0 {
            return Err(anyhow!("No redeemable amount available"));
        }
        
        // Update state
        self.total_debt = self.total_debt.saturating_sub(redeemable);
        
        Ok(redeemable)
    }
    
    /// Update the pricing parameters
    pub fn update_pricing(
        &mut self,
        new_virtual_input: Option<u128>,
        new_virtual_output: Option<u128>,
        new_half_life: Option<u64>,
        new_level_bips: Option<u64>,
        update_timestamp: bool
    ) {
        if let Some(value) = new_virtual_input {
            self.pricing.virtual_input_reserves = value;
        }
        
        if let Some(value) = new_virtual_output {
            self.pricing.virtual_output_reserves = value;
        }
        
        if let Some(value) = new_half_life {
            self.pricing.half_life = value;
        }
        
        if let Some(value) = new_level_bips {
            // Ensure level_bips is within valid range (0-10000)
            self.pricing.level_bips = value.min(10000);
        }
        
        if update_timestamp {
            self.pricing.last_update = Self::get_current_block();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_exp_to_level() {
        // Test with no decay (elapsed = 0)
        let x = 1000;
        let elapsed = 0;
        let half_life = 3600; // 1 hour (in blocks)
        let level_bips = 5000; // 50%
        
        let result = BondCurve::exp_to_level(x, elapsed, half_life, level_bips);
        assert_eq!(result, x, "No decay should return original value");
        
        // Test with one half-life elapsed
        let elapsed = 3600; // 1 hour (in blocks)
        let result = BondCurve::exp_to_level(x, elapsed, half_life, level_bips);
        assert_eq!(result, 750, "One half-life should reduce to 50% plus level floor");
        
        // Test with two half-lives elapsed
        let elapsed = 7200; // 2 hours (in blocks)
        let result = BondCurve::exp_to_level(x, elapsed, half_life, level_bips);
        assert_eq!(result, 625, "Two half-lives should reduce to 25% plus level floor");
        
        // Test with level_bips = 0 (no floor)
        let level_bips = 0;
        let result = BondCurve::exp_to_level(x, elapsed, half_life, level_bips);
        assert_eq!(result, 250, "With no floor, should decay to 25%");
        
        // Test with level_bips = 10000 (100% floor)
        let level_bips = 10000;
        let result = BondCurve::exp_to_level(x, elapsed, half_life, level_bips);
        assert_eq!(result, 1000, "With 100% floor, should remain at original value");
    }
    
    #[test]
    fn test_get_current_price() {
        // Create a bond curve with initial parameters
        let curve = BondCurve::new(
            1000000, // virtual input reserves
            500000,  // virtual output reserves
            3600,    // half-life (in blocks)
            5000,    // level bips (50%)
            86400    // term (in blocks)
        );
        
        // Test with available debt = 500000
        let available_debt = 500000;
        let price = curve.get_current_price(available_debt);
        
        // Expected price = 1000000 * SCALING_FACTOR / (500000 + 500000) = SCALING_FACTOR
        assert_eq!(price, SCALING_FACTOR, "Initial price should be SCALING_FACTOR");
    }
    
    #[test]
    fn test_purchase_bond() -> Result<()> {
        // Create a bond curve with initial parameters
        let mut curve = BondCurve::new(
            1000000, // virtual input reserves
            500000,  // virtual output reserves
            3600,    // half-life (in blocks)
            5000,    // level bips (50%)
            86400    // term (in blocks)
        );
        
        // Purchase a bond with 100000 input
        let available_debt = 500000;
        let output_amount = curve.purchase_bond(100000, available_debt)?;
        
        // Expected output should be around 90909
        assert!(output_amount > 90000 && output_amount < 91000, "Output should be approximately 90909");
        
        // Verify state updates
        assert_eq!(curve.pricing.virtual_input_reserves, 1100000, "Virtual input reserves should increase");
        assert_eq!(curve.total_debt, output_amount, "Total debt should increase by output amount");
        
        Ok(())
    }
    
    #[test]
    fn test_redeem_bond() -> Result<()> {
        // Create a bond curve with initial parameters
        let mut curve = BondCurve::new(
            1000000, // virtual input reserves
            500000,  // virtual output reserves
            3600,    // half-life (in blocks)
            5000,    // level bips (50%)
            86400    // term (in blocks)
        );
        
        // Current time will be used as creation_block, so we can't directly test maturity
        // Instead, we'll test the calculation logic
        
        // Set up parameters for a half-matured bond
        let owed = 100000;
        let redeemed = 0;
        let creation_block = BondCurve::get_current_block() - 43200; // half the term
        
        // Set initial total debt
        curve.total_debt = 100000;
        
        // Redeem the bond
        let redeemed_amount = curve.redeem_bond(owed, redeemed, creation_block)?;
        
        // Expected redeemed = 100000 / 2 = 50000 (approximately)
        assert!(redeemed_amount >= 49000 && redeemed_amount <= 51000, 
                "Should redeem approximately half of the bond: {}", redeemed_amount);
        
        // Verify state updates
        assert_eq!(curve.total_debt, 100000 - redeemed_amount, "Total debt should decrease by redeemed amount");
        
        Ok(())
    }
}
