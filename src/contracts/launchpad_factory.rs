use std::collections::HashMap;
use crate::contracts::orbital_bond_collection::OrbitalBondCollection;
use crate::utils::{BlockContext, TransactionContextExt};
use anyhow::{Result, anyhow};

#[derive(Clone)]
pub struct LaunchpadFactory {
    pub version: String,
    pub default_maturity_blocks: u64,
    pub default_interest_rate_bps: u16, // basis points
    pub collections: HashMap<String, OrbitalBondCollection>,
}

impl LaunchpadFactory {
    pub fn new(version: String, default_maturity_blocks: u64, default_interest_rate_bps: u16, _context: &impl BlockContext) -> Self {
        Self {
            version,
            default_maturity_blocks,
            default_interest_rate_bps,
            collections: HashMap::new(),
        }
    }
    
    pub fn create_collection(
        &mut self,
        name: String,
        symbol: String,
        description: Option<String>,
        interest_rate_bps: Option<u64>,
        maturity_blocks: Option<u64>,
        context: &impl BlockContext
    ) -> String {
        // Generate collection ID based on name and timestamp
        let collection_id = format!("col-{}-{}", name.to_lowercase().replace(" ", "-"), context.get_current_block_height());
        
        // Use provided values or defaults
        let interest_rate = interest_rate_bps
            .map(|r| r as u16)
            .unwrap_or(self.default_interest_rate_bps);
            
        let maturity = maturity_blocks
            .unwrap_or(self.default_maturity_blocks);
        
        // Create a new collection
        let mut collection = OrbitalBondCollection::new(
            collection_id.clone(),
            name,
            symbol,
            interest_rate,
            maturity,
            context,
        );
        
        // Add description if provided
        if let Some(desc) = description {
            collection = collection.with_description(desc);
        }
        
        // Store in the factory
        self.collections.insert(collection_id.clone(), collection);
        
        collection_id
    }
    
    pub fn get_collection(&self, collection_id: &str) -> Option<&OrbitalBondCollection> {
        self.collections.get(collection_id)
    }
    
    pub fn mint_bond(
        &mut self,
        collection_id: &str,
        orbital_token_id: String,
        amount: u64,
        owner_id: String,
        context: &impl BlockContext
    ) -> Result<String> {
        // Get the collection
        let collection = self.collections.get_mut(collection_id)
            .ok_or_else(|| anyhow!("Collection not found"))?;
        
        // Delegate to collection's mint_bond method
        collection.mint_bond(orbital_token_id, amount, owner_id, context)
    }
    
    pub fn redeem_bond_secure(
        &mut self,
        collection_id: &str,
        tx_context: &impl TransactionContextExt,
        redeemer_id: &str,
        context: &impl BlockContext
    ) -> Result<u64> {
        // Get the collection
        let collection = self.collections.get_mut(collection_id)
            .ok_or_else(|| anyhow!("Collection not found"))?;
        
        // Delegate to collection's redeem_bond_secure method
        collection.redeem_bond_secure(tx_context, redeemer_id, context)
    }
    
    // Utility methods
    pub fn collection_count(&self) -> usize {
        self.collections.len()
    }
    
    pub fn get_collections(&self) -> Vec<&OrbitalBondCollection> {
        self.collections.values().collect()
    }
}
