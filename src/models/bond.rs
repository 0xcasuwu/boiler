use std::fmt;
use crate::utils::BlockContext;
use serde::{Deserialize, Serialize};

/// Bond status representing the current state of a bond
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BondStatus {
    /// Bond is active but not yet mature
    Active,
    
    /// Bond has reached maturity and can be redeemed
    Mature,
    
    /// Bond has been redeemed
    Redeemed,
    
    /// Bond has been canceled (e.g., by issuer)
    Canceled,
}

impl fmt::Display for BondStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BondStatus::Active => write!(f, "Active"),
            BondStatus::Mature => write!(f, "Mature"),
            BondStatus::Redeemed => write!(f, "Redeemed"),
            BondStatus::Canceled => write!(f, "Canceled"),
        }
    }
}

/// Bond represents a financial contract with a maturity date (in blocks)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bond {
    /// Unique identifier for the bond
    pub id: String,
    
    /// The orbital token ID representing this bond
    pub orbital_token_id: String,
    
    /// The amount of diesel deposited to mint this bond
    pub amount: u64,
    
    /// Block number when the bond was created
    pub creation_block: u64,
    
    /// Block number when the bond will mature
    pub maturity_block: u64,
    
    /// Current status of the bond
    pub status: BondStatus,
    
    /// The interest rate (in basis points - 1/100th of a percent)
    pub interest_rate_bps: u16,
    
    /// Optional bond metadata (could contain name, description, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl Bond {
    /// Create a new bond
    pub fn new(
        id: String,
        orbital_token_id: String,
        amount: u64,
        creation_block: u64,
        maturity_blocks: u64,  // Duration in blocks until maturity
        interest_rate_bps: u16,
    ) -> Self {
        Self {
            id,
            orbital_token_id,
            amount,
            creation_block,
            maturity_block: creation_block + maturity_blocks,
            status: BondStatus::Active,
            interest_rate_bps,
            metadata: None,
        }
    }
    
    /// Check if the bond is mature given the current block context
    pub fn is_mature<T: BlockContext>(&self, block_context: &T) -> bool {
        if self.status != BondStatus::Active {
            return false;
        }
        
        block_context.is_block_height_reached(self.maturity_block)
    }
    
    /// Mark the bond as redeemed, returning the redemption amount including interest
    pub fn redeem<T: BlockContext>(&mut self, block_context: &T) -> Result<u64, &'static str> {
        // Can only redeem active bonds that have reached maturity
        if self.status != BondStatus::Active {
            return Err("Bond is not active");
        }
        
        if !self.is_mature(block_context) {
            return Err("Bond has not reached maturity");
        }
        
        // Update bond status
        self.status = BondStatus::Redeemed;
        
        // Return principal + interest 
        let interest = (self.amount as u128 * self.interest_rate_bps as u128 / 10_000) as u64;
        Ok(self.amount + interest)
    }
    
    /// Calculate the time remaining until maturity in blocks
    pub fn blocks_until_maturity<T: BlockContext>(&self, block_context: &T) -> u64 {
        if self.status != BondStatus::Active {
            return 0;
        }
        
        block_context.blocks_remaining(self.maturity_block)
    }
    
    /// Calculate the current value of the bond
    /// This can include time-based accrual of interest
    pub fn current_value<T: BlockContext>(&self, block_context: &T) -> u64 {
        if self.status != BondStatus::Active {
            return 0;
        }
        
        let current_block = block_context.get_current_block_height();
        
        // If not mature, calculate partial interest based on elapsed time
        if current_block < self.maturity_block {
            let total_duration = self.maturity_block - self.creation_block;
            
            // Handle case where current block is before creation block
            // This can happen with different clock sources in testing
            let elapsed_duration = if current_block > self.creation_block {
                current_block - self.creation_block
            } else {
                0 // No interest accrued yet
            };
            
            // Avoid division by zero
            if total_duration == 0 {
                return self.amount;
            }
            
            // Calculate partial interest based on elapsed time
            let interest_rate = self.interest_rate_bps as u128;
            let amount = self.amount as u128;
            
            // Partial interest: principal * rate * (elapsed/total)
            let partial_interest = 
                amount * interest_rate * elapsed_duration as u128 / 
                (10_000 * total_duration as u128);
                
            return (amount + partial_interest) as u64;
        }
        
        // If mature, return full amount plus interest
        let interest = (self.amount as u128 * self.interest_rate_bps as u128 / 10_000) as u64;
        self.amount + interest
    }
    
    /// Cancel the bond (typically by issuer)
    pub fn cancel(&mut self) -> Result<u64, &'static str> {
        if self.status != BondStatus::Active {
            return Err("Bond is not active");
        }
        
        self.status = BondStatus::Canceled;
        Ok(self.amount) // Return original amount without interest
    }
    
    /// Add metadata to the bond
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::{BlockContext, StandaloneBlockContext};
    
    // A simple mock context for tests that allows setting exact block height
    struct TestBlockContext {
        block_height: u64,
    }
    
    impl BlockContext for TestBlockContext {
        fn get_current_block_height(&self) -> u64 {
            self.block_height
        }
    }
    
    #[test]
    fn test_bond_creation() {
        let bond = Bond::new(
            "bond-1".to_string(),
            "token-1".to_string(),
            1000,
            100,
            50,
            500, // 5% interest
        );
        
        assert_eq!(bond.id, "bond-1");
        assert_eq!(bond.orbital_token_id, "token-1");
        assert_eq!(bond.amount, 1000);
        assert_eq!(bond.creation_block, 100);
        assert_eq!(bond.maturity_block, 150); // 100 + 50
        assert_eq!(bond.status, BondStatus::Active);
        assert_eq!(bond.interest_rate_bps, 500);
    }
    
    #[test]
    fn test_bond_maturity() {
        let bond = Bond::new(
            "bond-1".to_string(),
            "token-1".to_string(),
            1000,
            100,
            50,
            500, // 5% interest
        );
        
        // Test with block before maturity (block 90)
        let early_context = TestBlockContext { block_height: 90 };
        assert!(!bond.is_mature(&early_context));
        
        // Test with block at maturity (block 150)
        let context_at_maturity = TestBlockContext { block_height: 150 };
        assert!(bond.is_mature(&context_at_maturity));
        
        // Test with block after maturity (block 200)
        let late_context = TestBlockContext { block_height: 200 };
        assert!(bond.is_mature(&late_context));
    }
    
    #[test]
    fn test_bond_redemption() {
        // Create a bond with 5% interest
        let mut bond = Bond::new(
            "bond-1".to_string(),
            "token-1".to_string(),
            1000,
            100,
            50,
            500, // 5% interest
        );
        
        // Try to redeem before maturity (block 90)
        let early_context = TestBlockContext { block_height: 90 };
        let early_result = bond.redeem(&early_context);
        assert!(early_result.is_err());
        assert_eq!(bond.status, BondStatus::Active);
        
        // Redeem at maturity (block 150)
        let mature_context = TestBlockContext { block_height: 150 };
        let redemption_result = bond.redeem(&mature_context);
        assert!(redemption_result.is_ok());
        assert_eq!(redemption_result.unwrap(), 1050); // 1000 + 5% interest
        assert_eq!(bond.status, BondStatus::Redeemed);
        
        // Try to redeem again
        let second_redemption = bond.redeem(&mature_context);
        assert!(second_redemption.is_err());
    }
    
    #[test]
    fn test_bond_cancellation() {
        let mut bond = Bond::new(
            "bond-1".to_string(),
            "token-1".to_string(),
            1000,
            100,
            50,
            500, // 5% interest
        );
        
        // Cancel the bond
        let result = bond.cancel();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1000); // Original amount returned
        assert_eq!(bond.status, BondStatus::Canceled);
        
        // Try to cancel again
        let second_cancel = bond.cancel();
        assert!(second_cancel.is_err());
    }
    
    #[test]
    fn test_bond_with_metadata() {
        let metadata = serde_json::json!({
            "name": "Special Bond",
            "description": "A special bond with additional features",
            "issuer": "Treasury Department"
        });
        
        let bond = Bond::new(
            "bond-1".to_string(),
            "token-1".to_string(),
            1000,
            100,
            50,
            500,
        ).with_metadata(metadata.clone());
        
        assert!(bond.metadata.is_some());
        assert_eq!(bond.metadata.unwrap(), metadata);
    }
    
    #[test]
    fn test_bond_current_value() {
        let bond = Bond::new(
            "bond-1".to_string(),
            "token-1".to_string(),
            1000,
            100,
            100, // Mature at block 200
            500, // 5% interest
        );
        
        // At creation block - should be principal only
        let start_context = TestBlockContext { block_height: 100 };
        assert_eq!(bond.current_value(&start_context), 1000);
        
        // Halfway to maturity - should be principal + ~half interest
        let mid_context = TestBlockContext { block_height: 150 };
        let mid_value = bond.current_value(&mid_context);
        assert!(mid_value > 1000 && mid_value < 1050);
        assert_eq!(mid_value, 1025); // Exactly half the interest
        
        // At maturity - should be principal + full interest
        let maturity_context = TestBlockContext { block_height: 200 };
        assert_eq!(bond.current_value(&maturity_context), 1050); // 1000 + 5%
        
        // After maturity - should still be principal + full interest
        let post_maturity_context = TestBlockContext { block_height: 250 };
        assert_eq!(bond.current_value(&post_maturity_context), 1050);
    }
}
