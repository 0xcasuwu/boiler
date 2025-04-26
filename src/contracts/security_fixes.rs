// Security Fix #1: Remove or fully disable legacy redemption methods
//
// Patch for orbital_bond_collection.rs:
// Replace this:
/*
pub fn redeem_bond<T: BlockContext>(
    &mut self,
    orbital_token_id: &str,
    redeemer_id: &str,
    block_context: &T,
) -> Result<u64, &'static str> {
    // Simply delegate to the internal method
    self.redeem_bond_internal(orbital_token_id, redeemer_id, block_context)
}
*/

// With this:
pub fn redeem_bond<T: BlockContext>(
    &mut self,
    _orbital_token_id: &str,
    _redeemer_id: &str,
    _block_context: &T,
) -> Result<u64, &'static str> {
    // Method disabled for security
    Err("This method is disabled for security reasons. Use redeem_bond_secure instead.")
}

// And similarly for redeem_bond_by_alkane in LaunchpadFactory
