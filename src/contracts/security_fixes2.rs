// Security Fix #2: Proper validation in redeem_bond_by_alkane_secure
//
// Patch for launchpad_factory.rs:
// Replace this:
/*
pub fn redeem_bond_by_alkane_secure<T: BlockContext, C: TransactionContextExt>(
    &mut self,
    collection_id: &str,
    tx_context: &C,
    alkane_token_id: &str,
    redeemer_id: &str,
    block_context: &T,
) -> Result<u64, &'static str> {
    // Verify the transaction context contains the orbital token
    let _ = tx_context.orbital_token_id()
        .map_err(|_| "No orbital token in transaction context")?;
        
    // Continue with redemption...
*/

// With this:
pub fn redeem_bond_by_alkane_secure<T: BlockContext, C: TransactionContextExt>(
    &mut self,
    collection_id: &str,
    tx_context: &C,
    alkane_token_id: &str,
    redeemer_id: &str,
    block_context: &T,
) -> Result<u64, &'static str> {
    // Extract orbital token from transaction context
    let tx_orbital_token_id = tx_context.orbital_token_id()
        .map_err(|_| "No orbital token in transaction context")?;
        
    // Extract bond ID from alkane token ID
    if !alkane_token_id.starts_with("alkane-") {
        return Err("Invalid alkane token format");
    }
    
    let bond_id = &alkane_token_id[7..]; // Skip "alkane-" prefix
    
    // Get the collection
    let collection = self.collections.get(collection_id)
        .ok_or("Collection not found")?;
        
    // Find the bond to get the orbital token ID
    let bond = match collection.get_bond(bond_id) {
        Some(bond) => bond,
        None => return Err("Bond not found")
    };
    
    // Compare the orbital token ID from the transaction with the one from the bond
    if tx_orbital_token_id != bond.orbital_token_id {
        return Err("Orbital token in transaction does not match the bond's orbital token");
    }
    
    // Only now proceed with redemption using mutable collection reference
    let collection = self.collections.get_mut(collection_id)
        .ok_or("Collection not found")?;
        
    collection.redeem_bond_secure(tx_context, redeemer_id, block_context)
}
