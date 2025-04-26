// Security Fix #3: Check for Bond Status in redeem_bond_secure
//
// Currently our secure redemption method checks for token ownership,
// but we should also verify the bond has not been previously redeemed.
// Let's ensure our implementation has status checks.

// In orbital_bond_collection.rs, the redeem_bond_internal method should have:
/*
fn redeem_bond_internal<T: BlockContext>(
    &mut self,
    orbital_token_id: &str,
    redeemer_id: &str,
    block_context: &T,
) -> Result<u64, &'static str> {
    // Check if orbital token has a bond
    let bond_id = self.orbital_to_bond.get(orbital_token_id)
        .ok_or("Orbital token has no associated bond")?;

    // Get bond and attempt redemption
    let bond = self.bonds.get_mut(bond_id)
        .ok_or("Bond not found")?;
            
    bond.redeem(block_context)
}
*/

// This is fine as long as bond.redeem() checks status properly, which it should.
// In bond.rs, verify that redeem() properly checks:
/*
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
*/

// ----------------------------------------------------------------
// Security Fix #4: Cleaning Up Mapping on Redemption
//
// After a bond is redeemed, its orbital_token_id should be removed from 
// the orbital_to_bond mapping to prevent any future attempts to use it.

// Add this to orbital_bond_collection.rs:
fn redeem_bond_internal<T: BlockContext>(
    &mut self,
    orbital_token_id: &str,
    redeemer_id: &str,
    block_context: &T,
) -> Result<u64, &'static str> {
    // Check if orbital token has a bond
    let bond_id = self.orbital_to_bond.get(orbital_token_id)
        .ok_or("Orbital token has no associated bond")?;

    // Clone the bond_id to avoid borrow issues
    let bond_id_clone = bond_id.clone();
    
    // Get bond and attempt redemption
    let bond = self.bonds.get_mut(&bond_id_clone)
        .ok_or("Bond not found")?;
            
    // Try redeeming the bond
    match bond.redeem(block_context) {
        Ok(amount) => {
            // On successful redemption, clean up the orbital mapping
            self.orbital_to_bond.remove(orbital_token_id);
            Ok(amount)
        },
        Err(e) => Err(e)
    }
}

// ----------------------------------------------------------------
// Security Fix #5: Protection Against Integer Overflow in total_value
//
// The total_value method in OrbitalBondCollection and LaunchpadFactory 
// could be susceptible to integer overflow if the sum exceeds u64::MAX.

// For OrbitalBondCollection:
pub fn total_value<T: BlockContext>(&self, block_context: &T) -> u64 {
    // Use u128 for accumulation to prevent overflow
    let total = self.bonds.values()
        .filter(|b| b.status == BondStatus::Active)
        .fold(0u128, |acc, bond| acc + bond.current_value(block_context) as u128);
        
    // Check for overflow and saturate if needed
    if total > u64::MAX as u128 {
        return u64::MAX; // Return max value in case of overflow
    }
    
    total as u64
}

// Similar fix should be applied to LaunchpadFactory.total_value
