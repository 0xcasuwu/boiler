use std::time::{SystemTime, UNIX_EPOCH};
use anyhow::{Result, anyhow};

/// Scaling factor used to maintain precision in integer calculations
const SCALING_FACTOR: u128 = 1_000_000;

/// # Pricing Structure
///
/// Holds the configuration parameters for the bond curve's pricing mechanism.
/// These parameters control how bond prices evolve over time using a time-decay model.
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

/// # Bond Curve
///
/// Implements time-decay pricing for bonds using virtual reserves and a constant-product
/// market maker formula. This pricing model ensures bond prices react to both market
/// activity and time elapsed since the last update.
///
/// The curve uses virtual reserves combined with actual liquidity to determine prices,
/// with input reserves decaying over time according to a configurable half-life.
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
    /// Create a new bond curve with the specified parameters
    pub fn new(
        virtual_input_reserves: u128,
        virtual_output_reserves: u128,
        half_life: u64,
        level_bips: u64,
        term_blocks: u64,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        let pricing = Pricing {
            virtual_input_reserves,
            virtual_output_reserves,
            last_update: now as u64,
            half_life,
            level_bips,
        };
        
        Self {
            pricing,
            total_debt: 0,
            term_blocks,
        }
    }
    
    /// Get the current price of the token based on available debt
    pub fn get_current_price(&self, available_debt: u128) -> u128 {
        // Calculate decay factor based on time elapsed since last update
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as u64;
            
        let time_elapsed = now.saturating_sub(self.pricing.last_update);
        
        // Calculate decayed input reserves (x)
        let decayed_input_reserves = self.calculate_decayed_reserves(time_elapsed);
        
        // Constant-product market maker formula: price = y / x
        // where x = decayed_input_reserves, y = virtual_output_reserves
        if decayed_input_reserves == 0 {
            return u128::MAX; // Avoid division by zero
        }
        
        // Adjust price based on available debt - more debt = lower price
        let adjusted_input = decayed_input_reserves.saturating_add(available_debt);
        
        (self.pricing.virtual_output_reserves * SCALING_FACTOR) / adjusted_input
    }
    
    /// Calculate how many tokens can be minted for a given input amount
    pub fn get_amount_out(&self, input_amount: u128, available_debt: u128) -> u128 {
        let current_price = self.get_current_price(available_debt);
        
        // Output amount = input amount * current price / scaling factor
        // This preserves precision in integer math
        (input_amount * current_price) / SCALING_FACTOR
    }
    
    /// Calculate the amount to redeem based on owed amount, already redeemed amount,
    /// and time elapsed since creation
    pub fn get_redeem_amount(&self, owed: u128, redeemed: u128, creation_block: u64) -> Result<u128> {
        // Calculate how much is still owed
        let remaining = owed.saturating_sub(redeemed);
        if remaining == 0 {
            return Ok(0); // Nothing left to redeem
        }
        
        // Get current block height (approximated from time)
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as u64 / 10; // Simple approximation: 1 block = 10 seconds
            
        let blocks_elapsed = now.saturating_sub(creation_block);
        
        // Check if bond has matured
        if blocks_elapsed < self.term_blocks {
            return Err(anyhow!("Bond has not reached maturity yet"));
        }
        
        // For matured bonds, return the full remaining amount
        Ok(remaining)
    }
    
    /// Update the curve parameters
    pub fn update_parameters(
        &mut self,
        virtual_input_reserves: Option<u128>,
        virtual_output_reserves: Option<u128>,
        half_life: Option<u64>,
        level_bips: Option<u64>,
        term_blocks: Option<u64>,
    ) {
        // Update pricing parameters if provided
        if let Some(vir) = virtual_input_reserves {
            self.pricing.virtual_input_reserves = vir;
        }
        
        if let Some(vor) = virtual_output_reserves {
            self.pricing.virtual_output_reserves = vor;
        }
        
        if let Some(hl) = half_life {
            self.pricing.half_life = hl;
        }
        
        if let Some(lb) = level_bips {
            self.pricing.level_bips = lb;
        }
        
        if let Some(tb) = term_blocks {
            self.term_blocks = tb;
        }
        
        // Update last_update timestamp
        self.pricing.last_update = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as u64;
    }
    
    // Internal helper methods
    
    // Calculate the decayed reserves based on time elapsed
    fn calculate_decayed_reserves(&self, time_elapsed: u64) -> u128 {
        if time_elapsed == 0 || self.pricing.half_life == 0 {
            return self.pricing.virtual_input_reserves;
        }
        
        // Calculate the number of half-lives that have elapsed
        let half_lives = time_elapsed as f64 / self.pricing.half_life as f64;
        
        // Calculate the decay factor using 2^(-half_lives)
        let decay_factor = 2.0_f64.powf(-half_lives);
        
        // Apply the decay factor to the virtual input reserves
        let level_floor = self.pricing.level_bips as f64 / 10000.0;
        let decay_amount = self.pricing.virtual_input_reserves as f64 * (1.0 - level_floor) * (1.0 - decay_factor);
        
        self.pricing.virtual_input_reserves - decay_amount as u128
    }
}
