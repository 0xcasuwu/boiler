use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::contracts::orbital_bond_collection::OrbitalBondCollection;
use crate::utils::BlockContext;

/// LaunchpadFactory creates and manages OrbitalBondCollection instances
/// Following a factory pattern where each new bond collection is a separate issuance
#[derive(Debug, Serialize, Deserialize)]
pub struct LaunchpadFactory {
    /// Factory version
    pub version: String,
    
    /// Block when the factory was created
    pub creation_block: u64,
    
    /// Map of collection ID to OrbitalBondCollection
    collections: HashMap<String, OrbitalBondCollection>,
    
    /// Counter for generating unique collection IDs
    next_collection_id: u64,
    
    /// Default parameters for new collections
    default_maturity_blocks: u64,
    default_interest_rate_bps: u16,
}

impl LaunchpadFactory {
    /// Create a new LaunchpadFactory
    pub fn new<T: BlockContext>(
        version: String,
        default_maturity_blocks: u64,
        default_interest_rate_bps: u16,
        block_context: &T,
    ) -> Self {
        Self {
            version,
            creation_block: block_context.get_current_block_height(),
            collections: HashMap::new(),
            next_collection_id: 1,
            default_maturity_blocks,
            default_interest_rate_bps,
        }
    }
    
    /// Create a new bond collection
    pub fn create_collection<T: BlockContext>(
        &mut self,
        name: String,
        symbol: String,
        description: Option<String>,
        interest_rate_bps: Option<u16>,
        maturity_blocks: Option<u64>,
        block_context: &T,
    ) -> String {
        // Generate unique collection ID
        let collection_id = format!("collection-{}", self.next_collection_id);
        self.next_collection_id += 1;
        
        // Use provided parameters or defaults
        let interest_rate = interest_rate_bps.unwrap_or(self.default_interest_rate_bps);
        let maturity = maturity_blocks.unwrap_or(self.default_maturity_blocks);
        
        // Create the new collection
        let mut collection = OrbitalBondCollection::new(
            collection_id.clone(),
            name,
            symbol,
            interest_rate,
            maturity,
            block_context,
        );
        
        // Set description if provided
        if let Some(desc) = description {
            collection = collection.with_description(desc);
        }
        
        // Store the collection
        self.collections.insert(collection_id.clone(), collection);
        
        collection_id
    }
    
    /// Get a collection by ID
    pub fn get_collection(&self, collection_id: &str) -> Option<&OrbitalBondCollection> {
        self.collections.get(collection_id)
    }
    
    /// Get a mutable reference to a collection
    pub fn get_collection_mut(&mut self, collection_id: &str) -> Option<&mut OrbitalBondCollection> {
        self.collections.get_mut(collection_id)
    }
    
    /// Get all collections
    pub fn get_all_collections(&self) -> Vec<&OrbitalBondCollection> {
        self.collections.values().collect()
    }
    
    /// Get active collections
    pub fn get_active_collections(&self) -> Vec<&OrbitalBondCollection> {
        self.collections.values()
            .filter(|c| c.is_active())
            .collect()
    }
    
    /// Count total collections
    pub fn total_collections(&self) -> usize {
        self.collections.len()
    }
    
    /// Count active collections
    pub fn active_collections_count(&self) -> usize {
        self.get_active_collections().len()
    }
    
    /// Calculate total value of all collections
    pub fn total_value<T: BlockContext>(&self, block_context: &T) -> u64 {
        self.collections.values()
            .filter(|c| c.is_active())
            .fold(0, |acc, collection| acc + collection.total_value(block_context))
    }
    
    /// Mint a bond in a specific collection
    pub fn mint_bond<T: BlockContext>(
        &mut self,
        collection_id: &str,
        orbital_token_id: String,
        amount: u64,
        block_context: &T,
    ) -> Result<String, &'static str> {
        let collection = self.collections.get_mut(collection_id)
            .ok_or("Collection not found")?;
            
        collection.mint_bond(orbital_token_id, amount, block_context)
    }
    
    /// Redeem a bond from a collection using orbital token ID
    pub fn redeem_bond<T: BlockContext>(
        &mut self,
        collection_id: &str,
        orbital_token_id: &str,
        block_context: &T,
    ) -> Result<u64, &'static str> {
        let collection = self.collections.get_mut(collection_id)
            .ok_or("Collection not found")?;
            
        collection.redeem_bond(orbital_token_id, block_context)
    }
    
    /// Deactivate a collection
    pub fn deactivate_collection(&mut self, collection_id: &str) -> Result<(), &'static str> {
        let collection = self.collections.get_mut(collection_id)
            .ok_or("Collection not found")?;
            
        collection.deactivate();
        Ok(())
    }
    
    /// Reactivate a collection
    pub fn reactivate_collection(&mut self, collection_id: &str) -> Result<(), &'static str> {
        let collection = self.collections.get_mut(collection_id)
            .ok_or("Collection not found")?;
            
        collection.reactivate();
        Ok(())
    }
    
    /// Find collections that match a specific predicate 
    pub fn find_collections<F>(&self, predicate: F) -> Vec<&OrbitalBondCollection>
    where
        F: Fn(&OrbitalBondCollection) -> bool,
    {
        self.collections.values()
            .filter(|&collection| predicate(collection))
            .collect()
    }
    
    /// Find collections by name (partial match)
    pub fn find_collections_by_name(&self, name_part: &str) -> Vec<&OrbitalBondCollection> {
        self.find_collections(|c| c.name.to_lowercase().contains(&name_part.to_lowercase()))
    }
    
    /// Find collections by symbol (exact match)
    pub fn find_collections_by_symbol(&self, symbol: &str) -> Vec<&OrbitalBondCollection> {
        self.find_collections(|c| c.symbol == symbol)
    }
    
    /// Update default parameters for new collections
    pub fn update_defaults(&mut self, maturity_blocks: Option<u64>, interest_rate_bps: Option<u16>) {
        if let Some(maturity) = maturity_blocks {
            self.default_maturity_blocks = maturity;
        }
        
        if let Some(interest) = interest_rate_bps {
            self.default_interest_rate_bps = interest;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::StandaloneBlockContext;
    
    #[test]
    fn test_factory_creation() {
        let context = StandaloneBlockContext::new();
        let factory = LaunchpadFactory::new(
            "1.0.0".to_string(),
            100,  // Default maturity (100 blocks)
            500,  // Default interest rate (5%)
            &context,
        );
        
        assert_eq!(factory.version, "1.0.0");
        assert_eq!(factory.total_collections(), 0);
        assert_eq!(factory.active_collections_count(), 0);
        assert_eq!(factory.default_maturity_blocks, 100);
        assert_eq!(factory.default_interest_rate_bps, 500);
    }
    
    #[test]
    fn test_collection_creation() {
        let context = StandaloneBlockContext::new();
        let mut factory = LaunchpadFactory::new(
            "1.0.0".to_string(),
            100,
            500,
            &context,
        );
        
        // Create a collection with default parameters
        let collection_id = factory.create_collection(
            "Test Bonds".to_string(),
            "TBND".to_string(),
            Some("Test bond collection".to_string()),
            None,
            None,
            &context,
        );
        
        // Verify collection was created
        assert_eq!(factory.total_collections(), 1);
        assert_eq!(factory.active_collections_count(), 1);
        
        // Get the collection and check its properties
        let collection = factory.get_collection(&collection_id).unwrap();
        assert_eq!(collection.name, "Test Bonds");
        assert_eq!(collection.symbol, "TBND");
        assert_eq!(collection.description, Some("Test bond collection".to_string()));
        assert_eq!(collection.interest_rate_bps, 500);
        assert_eq!(collection.maturity_blocks, 100);
        
        // Create another collection with custom parameters
        let custom_collection_id = factory.create_collection(
            "Custom Bonds".to_string(),
            "CBND".to_string(),
            None,
            Some(1000), // 10% interest
            Some(200),  // 200 blocks maturity
            &context,
        );
        
        // Verify second collection
        assert_eq!(factory.total_collections(), 2);
        
        let custom_collection = factory.get_collection(&custom_collection_id).unwrap();
        assert_eq!(custom_collection.interest_rate_bps, 1000);
        assert_eq!(custom_collection.maturity_blocks, 200);
    }
    
    #[test]
    fn test_bond_operations() {
        let context = StandaloneBlockContext::new();
        let mut factory = LaunchpadFactory::new(
            "1.0.0".to_string(),
            50, // Short maturity for testing
            500,
            &context,
        );
        
        // Create a collection
        let collection_id = factory.create_collection(
            "Test Bonds".to_string(),
            "TBND".to_string(),
            None,
            None,
            None,
            &context,
        );
        
        // Mint some bonds
        let bond_result = factory.mint_bond(
            &collection_id,
            "orbital-1".to_string(),
            1000,
            &context,
        );
        
        assert!(bond_result.is_ok());
        let bond_id = bond_result.unwrap();
        
        // Verify the bond exists in the collection
        let collection = factory.get_collection(&collection_id).unwrap();
        assert_eq!(collection.total_bonds(), 1);
        let bond = collection.get_bond_by_orbital("orbital-1").unwrap();
        assert_eq!(bond.amount, 1000);
        
        // Create a new context at maturity
        let maturity_context = StandaloneBlockContext::new()
            .with_offset(context.get_current_block_height() as u64 + 51);
        
        // Redeem the bond
        let redemption_result = factory.redeem_bond(
            &collection_id,
            "orbital-1",
            &maturity_context,
        );
        
        assert!(redemption_result.is_ok());
        assert_eq!(redemption_result.unwrap(), 1050); // Principal + interest
    }
    
    #[test]
    fn test_collection_management() {
        let context = StandaloneBlockContext::new();
        let mut factory = LaunchpadFactory::new(
            "1.0.0".to_string(),
            100,
            500,
            &context,
        );
        
        // Create collections
        let c1 = factory.create_collection(
            "Alpha Bonds".to_string(),
            "ABND".to_string(),
            None,
            None,
            None,
            &context,
        );
        
        let c2 = factory.create_collection(
            "Beta Bonds".to_string(),
            "BBND".to_string(),
            None,
            None,
            None,
            &context,
        );
        
        // Deactivate a collection
        factory.deactivate_collection(&c1).unwrap();
        
        // Verify active collections count
        assert_eq!(factory.total_collections(), 2);
        assert_eq!(factory.active_collections_count(), 1);
        
        // Try to mint a bond in inactive collection
        let mint_result = factory.mint_bond(
            &c1,
            "orbital-1".to_string(),
            1000,
            &context,
        );
        
        assert!(mint_result.is_err());
        
        // Reactivate and try again
        factory.reactivate_collection(&c1).unwrap();
        assert_eq!(factory.active_collections_count(), 2);
        
        let mint_result2 = factory.mint_bond(
            &c1,
            "orbital-1".to_string(),
            1000,
            &context,
        );
        
        assert!(mint_result2.is_ok());
    }
    
    #[test]
    fn test_collection_search() {
        let context = StandaloneBlockContext::new();
        let mut factory = LaunchpadFactory::new(
            "1.0.0".to_string(),
            100,
            500,
            &context,
        );
        
        // Create collections with different names
        factory.create_collection(
            "Alpha Bonds".to_string(),
            "ABND".to_string(),
            None,
            None,
            None,
            &context,
        );
        
        factory.create_collection(
            "Beta Bonds".to_string(),
            "BBND".to_string(),
            None,
            None,
            None,
            &context,
        );
        
        factory.create_collection(
            "Alpha Plus".to_string(),
            "APLS".to_string(),
            None,
            None,
            None,
            &context,
        );
        
        // Search by partial name
        let alpha_collections = factory.find_collections_by_name("Alpha");
        assert_eq!(alpha_collections.len(), 2);
        
        // Search by symbol
        let bbnd_collections = factory.find_collections_by_symbol("BBND");
        assert_eq!(bbnd_collections.len(), 1);
        assert_eq!(bbnd_collections[0].name, "Beta Bonds");
        
        // Custom search predicate
        let custom_search = factory.find_collections(|c| c.name.contains("Plus"));
        assert_eq!(custom_search.len(), 1);
        assert_eq!(custom_search[0].symbol, "APLS");
    }
    
    #[test]
    fn test_update_defaults() {
        let context = StandaloneBlockContext::new();
        let mut factory = LaunchpadFactory::new(
            "1.0.0".to_string(),
            100,
            500,
            &context,
        );
        
        // Update default parameters
        factory.update_defaults(Some(200), Some(800));
        
        // Create a collection with new defaults
        let collection_id = factory.create_collection(
            "New Default Bonds".to_string(),
            "NDBND".to_string(),
            None,
            None, // Use default interest
            None, // Use default maturity
            &context,
        );
        
        // Verify new defaults were used
        let collection = factory.get_collection(&collection_id).unwrap();
        assert_eq!(collection.interest_rate_bps, 800);
        assert_eq!(collection.maturity_blocks, 200);
    }
}
