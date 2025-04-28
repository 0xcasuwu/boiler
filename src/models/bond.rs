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

/// # Bond
/// 
/// A financial contract that represents a debt instrument with:
///
/// - A unique identifier and associated orbital token
/// - A principal amount that earns interest over time
/// - A maturity date expressed in block height
/// - Status tracking (Active, Mature, Redeemed, Canceled)
///
/// Bonds are created through minting, earn interest over time until maturity,
/// and can be redeemed for principal plus interest after reaching maturity.
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
    /// Creates a new bond with specified parameters
    ///
    /// # Parameters
    /// * `id` - Unique identifier for this bond
    /// * `orbital_token_id` - ID of the orbital token that backs this bond
    /// * `amount` - Principal amount of the bond
    /// * `creation_block` - Block height when the bond is created
    /// * `maturity_blocks` - Duration in blocks until the bond matures
    /// * `interest_rate_bps` - Interest rate in basis points (1/100th of a percent, e.g. 500 = 5%)
    ///
    /// # Returns
    /// A new Bond instance in Active status
    ///
    /// # Example
    /// ```
    /// use slop::models::Bond;
    /// 
    /// let current_block = 100; // Current block height
    /// let bond = Bond::new(
    ///     "bond-1".to_string(),
    ///     "orbital-token-xyz".to_string(),
    ///     1000,             // 1000 tokens as principal
    ///     current_block,    // creation block
    ///     100,              // matures after 100 blocks
    ///     500               // 5% interest
    /// );
    /// ```
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
    
    /// Checks if the bond has reached maturity and can be redeemed
    ///
    /// A bond is considered mature when:
    /// 1. It is in Active status, and
    /// 2. The current block height has reached or passed the maturity block
    ///
    /// # Parameters
    /// * `block_context` - Context providing current block information
    ///
    /// # Returns
    /// `true` if the bond is mature and can be redeemed, `false` otherwise
    ///
    /// # Example
    /// ```
    /// use slop::models::Bond;
    /// use slop::utils::StandaloneBlockContext;
    /// 
    /// // Create a bond that matures after 50 blocks
    /// let bond = Bond::new(
    ///     "bond-1".to_string(),
    ///     "orbital-xyz".to_string(),
    ///     1000,
    ///     100, // created at block 100
    ///     50,  // matures after 50 blocks (at block 150)
    ///     500  // 5% interest
    /// );
    /// 
    /// // Check at block 140 (not mature yet)
    /// let early_context = StandaloneBlockContext::new().with_offset(140);
    /// if bond.is_mature(&early_context) {
    ///     println!("Bond is ready for redemption!");
    /// } else {
    ///     println!("Bond is not mature yet");
    /// }
    /// 
    /// // Check at block 150 (mature)
    /// let mature_context = StandaloneBlockContext::new().with_offset(150);
    /// if bond.is_mature(&mature_context) {
    ///     println!("Bond is ready for redemption!");
    /// }
    /// ```
    pub fn is_mature<T: BlockContext>(&self, block_context: &T) -> bool {
        // Check active status first
        if self.status != BondStatus::Active {
            return false;
        }
        
        // SECURITY: Add sanity check for block context to prevent time manipulation
        let current_block = block_context.get_current_block_height();
        
        // If the block context appears to be from before the bond's creation,
        // this could indicate manipulation or an invalid context
        if current_block < self.creation_block {
            return false; // Reject suspicious block contexts
        }
        
        // Verify the bond has reached maturity
        current_block >= self.maturity_block
    }
    
    /// Redeems the bond and returns the total amount (principal + interest)
    ///
    /// This method will:
    /// 1. Check if the bond is active and has reached maturity
    /// 2. Change the bond status to Redeemed
    /// 3. Calculate and return the redemption amount (principal + interest)
    ///
    /// # Parameters
    /// * `block_context` - Context providing current block information
    ///
    /// # Returns
    /// * `Ok(u64)` - The redemption amount (principal + interest) if successful
    /// * `Err(&'static str)` - Error message if redemption failed
    ///
    /// # Errors
    /// * "Bond is not active" - If the bond has already been redeemed or canceled
    /// * "Bond has not reached maturity" - If attempting to redeem before maturity date
    ///
    /// # Example
    /// ```
    /// use slop::models::Bond;
    /// use slop::utils::StandaloneBlockContext;
    /// 
    /// // Create a bond that matures after 50 blocks
    /// let mut bond = Bond::new(
    ///     "bond-1".to_string(),
    ///     "orbital-xyz".to_string(),
    ///     1000,
    ///     100, // created at block 100
    ///     50,  // matures after 50 blocks (at block 150)
    ///     500  // 5% interest
    /// );
    /// 
    /// // Try redeeming at block 140 (not mature yet)
    /// let early_context = StandaloneBlockContext::new().with_offset(140);
    /// match bond.redeem(&early_context) {
    ///     Ok(amount) => println!("Redeemed bond for {} tokens", amount),
    ///     Err(e) => println!("Failed to redeem bond: {}", e),
    /// }
    /// 
    /// // Try redeeming at block 150 (mature)
    /// let mature_context = StandaloneBlockContext::new().with_offset(150);
    /// match bond.redeem(&mature_context) {
    ///     Ok(amount) => println!("Redeemed bond for {} tokens", amount),
    ///     Err(e) => println!("Failed to redeem bond: {}", e),
    /// }
    /// ```
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
        
        // Calculate total amount (principal + interest) using u128 to avoid overflow
        let principal = self.amount as u128;
        let interest = principal * self.interest_rate_bps as u128 / 10_000;
        let total = principal + interest;
        
        // Check for overflow before converting back to u64
        if total > u64::MAX as u128 {
            return Err("Integer overflow in interest calculation");
        }
        
        Ok(total as u64)
    }
    
    /// Calculates the number of blocks remaining until this bond matures
    ///
    /// # Parameters
    /// * `block_context` - Context providing current block information
    ///
    /// # Returns
    /// The number of blocks remaining until maturity, or 0 if:
    /// - The bond is already mature
    /// - The bond is not in Active status
    ///
    /// # Example
    /// ```
    /// use slop::models::Bond;
    /// use slop::utils::StandaloneBlockContext;
    /// 
    /// let current_block = 100;
    /// let bond = Bond::new(
    ///     "bond-1".to_string(),
    ///     "orbital-xyz".to_string(),
    ///     1000,
    ///     current_block,
    ///     50,
    ///     500
    /// );
    /// 
    /// let block_context = StandaloneBlockContext::new().with_offset(120);
    /// let blocks_remaining = bond.blocks_until_maturity(&block_context);
    /// println!("Bond matures in {} blocks", blocks_remaining);
    /// ```
    pub fn blocks_until_maturity<T: BlockContext>(&self, block_context: &T) -> u64 {
        if self.status != BondStatus::Active {
            return 0;
        }
        
        block_context.blocks_remaining(self.maturity_block)
    }
    
    /// Calculates the current value of the bond, including accrued interest
    ///
    /// This method implements linear interest accrual based on elapsed time:
    /// - Before maturity: partial interest based on elapsed/total time
    /// - At or after maturity: full interest amount
    /// - If bond is not active: returns 0
    ///
    /// # Parameters
    /// * `block_context` - Context providing current block information
    ///
    /// # Returns
    /// The current value of the bond (principal + accrued interest)
    ///
    /// # Example
    /// ```
    /// use slop::models::Bond;
    /// use slop::utils::StandaloneBlockContext;
    /// 
    /// // Create a bond that matures after 50 blocks
    /// let bond = Bond::new(
    ///     "bond-1".to_string(),
    ///     "orbital-xyz".to_string(),
    ///     1000,
    ///     100, // created at block 100
    ///     50,  // matures after 50 blocks (at block 150)
    ///     500  // 5% interest
    /// );
    /// 
    /// // Check value at creation
    /// let start_context = StandaloneBlockContext::new().with_offset(100);
    /// let start_value = bond.current_value(&start_context);
    /// println!("Bond value at creation: {}", start_value); // 1000
    /// 
    /// // Check value at halfway point (partial interest)
    /// let mid_context = StandaloneBlockContext::new().with_offset(125);
    /// let mid_value = bond.current_value(&mid_context);
    /// println!("Bond value halfway to maturity: {}", mid_value); // ~1025
    /// 
    /// // Check value at maturity (full interest)
    /// let mature_context = StandaloneBlockContext::new().with_offset(150);
    /// let mature_value = bond.current_value(&mature_context);
    /// println!("Bond value at maturity: {}", mature_value); // 1050
    /// ```
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
        
        // If mature, return full amount plus interest using u128 to avoid overflow
        let principal = self.amount as u128;
        let interest = principal * self.interest_rate_bps as u128 / 10_000;
        let total = principal + interest;
        
        // Check for overflow before converting back to u64
        if total > u64::MAX as u128 {
            return u64::MAX; // Return max value if overflow would occur
        }
        
        total as u64
    }
    
    /// Cancels the bond and returns the principal amount without interest
    ///
    /// This method will:
    /// 1. Check if the bond is in Active status
    /// 2. Change the bond status to Canceled
    /// 3. Return the original principal amount (no interest)
    ///
    /// # Returns
    /// * `Ok(u64)` - The principal amount if cancellation is successful
    /// * `Err(&'static str)` - Error message if cancellation failed
    ///
    /// # Errors
    /// * "Bond is not active" - If the bond was already redeemed or canceled
    ///
    /// # Example
    /// ```
    /// use slop::models::Bond;
    /// 
    /// // Create a bond
    /// let mut bond = Bond::new(
    ///     "bond-1".to_string(),
    ///     "orbital-xyz".to_string(),
    ///     1000,
    ///     100,
    ///     50,
    ///     500  // 5% interest
    /// );
    /// 
    /// // Cancel the bond
    /// match bond.cancel() {
    ///     Ok(amount) => println!("Canceled bond, returning {} tokens", amount),
    ///     Err(e) => println!("Failed to cancel bond: {}", e),
    /// }
    /// 
    /// // Try to cancel again (will fail)
    /// match bond.cancel() {
    ///     Ok(amount) => println!("Canceled bond again, returning {} tokens", amount),
    ///     Err(e) => println!("Failed to cancel bond: {}", e),
    /// }
    /// ```
    pub fn cancel(&mut self) -> Result<u64, &'static str> {
        if self.status != BondStatus::Active {
            return Err("Bond is not active");
        }
        
        self.status = BondStatus::Canceled;
        Ok(self.amount) // Return original amount without interest
    }
    
    /// Adds metadata to the bond and returns self for method chaining
    ///
    /// # Parameters
    /// * `metadata` - Any serializable value to associate with this bond
    ///
    /// # Returns
    /// Self with metadata added, allowing for method chaining
    ///
/// # Example
/// ```
/// use slop::models::Bond;
/// use serde_json::json;
///
/// let bond = Bond::new(
///     "bond-1".to_string(),
///     "orbital-xyz".to_string(),
///     1000,
///     100,  // current_block
///     50,   // maturity in 50 blocks
///     500   // 5% interest
/// ).with_metadata(json!({
///     "issuer": "Treasury Department",
///     "purpose": "Infrastructure funding",
///     "series": "A-2023"
/// }));
/// ```
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
