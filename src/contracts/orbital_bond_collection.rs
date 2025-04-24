use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::models::bond::{Bond, BondStatus};
use crate::utils::BlockContext;

/// OrbitalBondCollection manages orbital tokens that double as bonds and authentication tokens
#[derive(Debug, Serialize, Deserialize)]
pub struct OrbitalBondCollection {
    /// Unique collection ID
    pub id: String,
    
    /// Collection name
    pub name: String,
    
    /// Collection symbol
    pub symbol: String,
    
    /// Description of the bond collection
    pub description: Option<String>,
    
    /// Block at which this collection was created
    pub creation_block: u64,
    
    /// Interest rate for bonds in this collection (basis points)
    pub interest_rate_bps: u16,
    
    /// Maturity period in blocks
    pub maturity_blocks: u64,
    
    /// Mapping of bond ID to Bond
    bonds: HashMap<String, Bond>,
    
    /// Mapping of orbital token ID to bond ID
    orbital_to_bond: HashMap<String, String>,
    
    /// Next bond ID (auto-incremented)
    next_bond_id: u64,
    
    /// Whether the collection is active or not
    pub active: bool,
}

impl OrbitalBondCollection {
    /// Create a new OrbitalBondCollection
    pub fn new<T: BlockContext>(
        id: String,
        name: String,
        symbol: String,
        interest_rate_bps: u16,
        maturity_blocks: u64,
        block_context: &T,
    ) -> Self {
        Self {
            id,
            name,
            symbol,
            description: None,
            creation_block: block_context.get_current_block_height(),
            interest_rate_bps,
            maturity_blocks,
            bonds: HashMap::new(),
            orbital_to_bond: HashMap::new(),
            next_bond_id: 1,
            active: true,
        }
    }
    
    /// Set the collection description
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
    
    /// Check if the collection is active
    pub fn is_active(&self) -> bool {
        self.active
    }
    
    /// Get total bonds count
    pub fn total_bonds(&self) -> usize {
        self.bonds.len()
    }
    
    /// Create a new bond with an orbital token as authentication
    pub fn mint_bond<T: BlockContext>(
        &mut self,
        orbital_token_id: String,
        amount: u64,
        block_context: &T,
    ) -> Result<String, &'static str> {
        // Check if collection is active
        if !self.active {
            return Err("Collection is inactive");
        }
        
        // Check if orbital token already has a bond
        if self.orbital_to_bond.contains_key(&orbital_token_id) {
            return Err("Orbital token already has a bond");
        }
        
        // Generate bond ID
        let bond_id = format!("{}-{}", self.id, self.next_bond_id);
        self.next_bond_id += 1;
        
        // Get current block
        let current_block = block_context.get_current_block_height();
        
        // Create the new bond
        let bond = Bond::new(
            bond_id.clone(),
            orbital_token_id.clone(),
            amount,
            current_block,
            self.maturity_blocks,
            self.interest_rate_bps,
        );
        
        // Store mappings
        self.bonds.insert(bond_id.clone(), bond);
        self.orbital_to_bond.insert(orbital_token_id, bond_id.clone());
        
        Ok(bond_id)
    }
    
    /// Get bond details by bond ID
    pub fn get_bond(&self, bond_id: &str) -> Option<&Bond> {
        self.bonds.get(bond_id)
    }
    
    /// Get bond details by orbital token ID
    pub fn get_bond_by_orbital(&self, orbital_token_id: &str) -> Option<&Bond> {
        self.orbital_to_bond.get(orbital_token_id)
            .and_then(|bond_id| self.bonds.get(bond_id))
    }
    
    /// Redeem a bond using its orbital token as authentication
    pub fn redeem_bond<T: BlockContext>(
        &mut self,
        orbital_token_id: &str,
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
    
    /// Get all bonds in the collection
    pub fn get_all_bonds(&self) -> Vec<&Bond> {
        self.bonds.values().collect()
    }
    
    /// Get active bonds (not redeemed or canceled)
    pub fn get_active_bonds(&self) -> Vec<&Bond> {
        self.bonds.values()
            .filter(|b| b.status == BondStatus::Active)
            .collect()
    }
    
    /// Get mature bonds that can be redeemed
    pub fn get_mature_bonds<T: BlockContext>(&self, block_context: &T) -> Vec<&Bond> {
        self.bonds.values()
            .filter(|b| b.is_mature(block_context))
            .collect()
    }
    
    /// Calculate total value of all active bonds
    pub fn total_value<T: BlockContext>(&self, block_context: &T) -> u64 {
        self.bonds.values()
            .filter(|b| b.status == BondStatus::Active)
            .fold(0, |acc, bond| acc + bond.current_value(block_context))
    }
    
    /// Deactivate the collection (no new bonds can be created)
    pub fn deactivate(&mut self) {
        self.active = false;
    }
    
    /// Reactivate the collection
    pub fn reactivate(&mut self) {
        self.active = true;
    }
    
    /// Cancel a specific bond
    pub fn cancel_bond(&mut self, bond_id: &str) -> Result<u64, &'static str> {
        let bond = self.bonds.get_mut(bond_id)
            .ok_or("Bond not found")?;
            
        bond.cancel()
    }
    
    /// Test-only method to update bonds using a provided callback function
    /// This should ONLY be used for testing purposes
    #[cfg(any(test, feature = "testing"))]
    pub fn update_bonds_for_test<F>(&mut self, update_fn: F)
    where
        F: Fn(&mut Bond),
    {
        // Apply the provided callback to each bond in the collection
        for bond in self.bonds.values_mut() {
            update_fn(bond);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::StandaloneBlockContext;
    
    #[test]
    fn test_collection_creation() {
        let context = StandaloneBlockContext::new();
        let collection = OrbitalBondCollection::new(
            "collection-1".to_string(),
            "Test Bonds".to_string(),
            "TBND".to_string(),
            500, // 5% interest
            100, // 100 blocks to mature
            &context,
        );
        
        assert_eq!(collection.id, "collection-1");
        assert_eq!(collection.name, "Test Bonds");
        assert_eq!(collection.symbol, "TBND");
        assert_eq!(collection.interest_rate_bps, 500);
        assert_eq!(collection.maturity_blocks, 100);
        assert!(collection.is_active());
        assert_eq!(collection.total_bonds(), 0);
    }
    
    #[test]
    fn test_bond_minting() {
        let context = StandaloneBlockContext::new();
        let mut collection = OrbitalBondCollection::new(
            "collection-1".to_string(),
            "Test Bonds".to_string(),
            "TBND".to_string(),
            500,
            100,
            &context,
        );
        
        // Mint a bond
        let bond_result = collection.mint_bond(
            "orbital-1".to_string(),
            1000,
            &context,
        );
        
        assert!(bond_result.is_ok());
        let bond_id = bond_result.unwrap();
        assert_eq!(collection.total_bonds(), 1);
        
        // Try to mint another bond with the same orbital
        let duplicate_result = collection.mint_bond(
            "orbital-1".to_string(),
            2000,
            &context,
        );
        
        assert!(duplicate_result.is_err());
        assert_eq!(collection.total_bonds(), 1);
        
        // Mint another bond with a different orbital
        let bond_result2 = collection.mint_bond(
            "orbital-2".to_string(),
            2000,
            &context,
        );
        
        assert!(bond_result2.is_ok());
        assert_eq!(collection.total_bonds(), 2);
        
        // Check bond retrieval
        let bond = collection.get_bond(&bond_id);
        assert!(bond.is_some());
        assert_eq!(bond.unwrap().amount, 1000);
        
        // Verify bond can be retrieved by orbital ID
        let bond_by_orbital = collection.get_bond_by_orbital("orbital-1");
        assert!(bond_by_orbital.is_some());
        assert_eq!(bond_by_orbital.unwrap().id, bond_id);
    }
    
    #[test]
    fn test_bond_redemption() {
        let mut context = StandaloneBlockContext::new();
        let current_block = context.get_current_block_height();
        
        let mut collection = OrbitalBondCollection::new(
            "collection-1".to_string(),
            "Test Bonds".to_string(),
            "TBND".to_string(),
            500, // 5% interest
            50,  // 50 blocks to mature
            &context,
        );
        
        // Mint a bond
        let bond_id = collection.mint_bond(
            "orbital-1".to_string(),
            1000,
            &context,
        ).unwrap();
        
        // Try to redeem before maturity
        let early_redemption = collection.redeem_bond("orbital-1", &context);
        assert!(early_redemption.is_err());
        
        // Advance block context to maturity
        let maturity_context = StandaloneBlockContext::with_seconds_per_block(1)
            .with_offset(current_block as u64 + 51); // Just past maturity
        
        // Redeem the bond
        let redemption_result = collection.redeem_bond("orbital-1", &maturity_context);
        assert!(redemption_result.is_ok());
        assert_eq!(redemption_result.unwrap(), 1050); // Principal + 5% interest
        
        // Get the bond and verify its status
        let bond = collection.get_bond(&bond_id).unwrap();
        assert_eq!(bond.status, BondStatus::Redeemed);
        
        // Try to redeem again
        let second_redemption = collection.redeem_bond("orbital-1", &maturity_context);
        assert!(second_redemption.is_err());
    }
    
    #[test]
    fn test_collection_deactivation() {
        let context = StandaloneBlockContext::new();
        let mut collection = OrbitalBondCollection::new(
            "collection-1".to_string(),
            "Test Bonds".to_string(),
            "TBND".to_string(),
            500,
            100,
            &context,
        );
        
        // Mint a bond while active
        let mint_result = collection.mint_bond("orbital-1".to_string(), 1000, &context);
        assert!(mint_result.is_ok());
        
        // Deactivate collection
        collection.deactivate();
        assert!(!collection.is_active());
        
        // Try to mint when inactive
        let inactive_mint = collection.mint_bond("orbital-2".to_string(), 1000, &context);
        assert!(inactive_mint.is_err());
        
        // Reactivate and try again
        collection.reactivate();
        assert!(collection.is_active());
        
        let reactivated_mint = collection.mint_bond("orbital-2".to_string(), 1000, &context);
        assert!(reactivated_mint.is_ok());
    }
    
    #[test]
    fn test_bond_cancellation() {
        let context = StandaloneBlockContext::new();
        let mut collection = OrbitalBondCollection::new(
            "collection-1".to_string(),
            "Test Bonds".to_string(),
            "TBND".to_string(),
            500,
            100,
            &context,
        );
        
        // Mint a bond
        let bond_id = collection.mint_bond("orbital-1".to_string(), 1000, &context).unwrap();
        
        // Cancel the bond
        let cancel_result = collection.cancel_bond(&bond_id);
        assert!(cancel_result.is_ok());
        assert_eq!(cancel_result.unwrap(), 1000); // Original amount returned
        
        // Verify bond status
        let bond = collection.get_bond(&bond_id).unwrap();
        assert_eq!(bond.status, BondStatus::Canceled);
        
        // Try to redeem a canceled bond
        let redemption_result = collection.redeem_bond("orbital-1", &context);
        assert!(redemption_result.is_err());
    }
    
    #[test]
    fn test_collection_queries() {
        let mut context = StandaloneBlockContext::new();
        let starting_block = context.get_current_block_height();
        
        let mut collection = OrbitalBondCollection::new(
            "collection-1".to_string(),
            "Test Bonds".to_string(),
            "TBND".to_string(),
            500, // 5% interest
            100, // 100 blocks to mature
            &context,
        );
        
        // Mint a few bonds
        collection.mint_bond("orbital-1".to_string(), 1000, &context).unwrap();
        collection.mint_bond("orbital-2".to_string(), 2000, &context).unwrap();
        collection.mint_bond("orbital-3".to_string(), 3000, &context).unwrap();
        
        // Cancel one bond
        let bond_id = collection.get_bond_by_orbital("orbital-2").unwrap().id.clone();
        collection.cancel_bond(&bond_id).unwrap();
        
        // Advance to make one bond mature
        let mature_context = StandaloneBlockContext::with_seconds_per_block(1)
            .with_offset(starting_block as u64 + 101);
            
        // Test various queries
        assert_eq!(collection.get_all_bonds().len(), 3);
        assert_eq!(collection.get_active_bonds().len(), 2);
        assert_eq!(collection.get_mature_bonds(&context).len(), 0);
        assert_eq!(collection.get_mature_bonds(&mature_context).len(), 2);
        
        // Test total value
        let total_value = collection.total_value(&context);
        assert_eq!(total_value, 4000); // 1000 + 3000 (only active bonds)
    }
}
