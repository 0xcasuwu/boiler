//! # Launchpad Factory Module
//!
//! This module provides a factory pattern implementation for creating and managing
//! multiple bond collections. The factory acts as the coordinator between users and 
//! individual bond collections, managing issuance, redemption, and collection lifecycle.
//!
//! The LaunchpadFactory serves as the primary entry point for:
//! - Creating new bond collections with configurable parameters
//! - Managing the lifecycle of bond collections (activation/deactivation)
//! - Minting and redeeming bonds across multiple collections
//! - Searching and querying collections based on various criteria

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::contracts::OrbitalBondCollection;
use crate::utils::BlockContext;
use crate::utils::transaction_context::TransactionContextExt;
use crate::alkanes_support::parcel::AlkaneTransfer;

/// # Launchpad Factory
///
/// Factory for creating and managing OrbitalBondCollection instances.
/// Each collection represents a separate bond issuance with its own parameters.
///
/// The factory maintains a registry of all bond collections and provides
/// unified methods for interacting with them, while maintaining proper
/// authentication and validation.
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
    /// # Create a New LaunchpadFactory
    ///
    /// Creates a new factory with default parameters for future collections.
    ///
    /// # Parameters
    /// * `version` - Version identifier for the factory
    /// * `default_maturity_blocks` - Default maturity period (in blocks) for new collections
    /// * `default_interest_rate_bps` - Default interest rate in basis points for new collections
    /// * `block_context` - Context providing current block height information
    ///
    /// # Returns
    /// A new LaunchpadFactory instance
    ///
    /// # Example
    /// ```
    /// use slop::contracts::LaunchpadFactory;
    /// use slop::utils::StandaloneBlockContext;
    ///
    /// let context = StandaloneBlockContext::new();
    /// let factory = LaunchpadFactory::new(
    ///     "1.0.0".to_string(),
    ///     86400,  // Default maturity: 10 days (assuming 10s blocks)
    ///     500,    // Default interest: 5%
    ///     &context
    /// );
    /// ```
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

    /// # Create a New Bond Collection
    ///
    /// Creates a new bond collection with the specified parameters and registers it
    /// in the factory. If optional parameters are not provided, factory defaults are used.
    ///
    /// # Parameters
    /// * `name` - Display name for the collection
    /// * `symbol` - Symbol/ticker for the collection
    /// * `description` - Optional description of the collection
    /// * `interest_rate_bps` - Optional interest rate in basis points (1/100 of a percent)
    /// * `maturity_blocks` - Optional maturity period in blocks
    /// * `block_context` - Context providing current block height information
    ///
    /// # Returns
    /// The unique ID of the created collection
    ///
    /// # Example
    /// ```
    /// use slop::contracts::LaunchpadFactory;
    /// use slop::utils::StandaloneBlockContext;
    ///
    /// let context = StandaloneBlockContext::new();
    /// let mut factory = LaunchpadFactory::new(
    ///     "1.0.0".to_string(), 86400, 500, &context
    /// );
    ///
    /// // Create a collection with custom parameters
    /// let collection_id = factory.create_collection(
    ///     "Premium Bonds".to_string(),
    ///     "PBND".to_string(),
    ///     Some("High-interest premium bonds".to_string()),
    ///     Some(800),   // 8% interest
    ///     Some(43200), // 5-day maturity
    ///     &context
    /// );
    ///
    /// // Verify collection was created
    /// assert!(factory.collection_exists(&collection_id));
    /// ```
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

    /// # Get Collection by ID
    ///
    /// Retrieves a reference to a collection by its unique identifier.
    ///
    /// # Parameters
    /// * `collection_id` - The unique identifier of the collection to retrieve
    ///
    /// # Returns
    /// * `Some(&OrbitalBondCollection)` - Reference to the collection if found
    /// * `None` - If no collection exists with the given ID
    ///
    /// # Example
    /// ```
    /// use slop::contracts::LaunchpadFactory;
    /// use slop::utils::StandaloneBlockContext;
    ///
    /// // Create factory and a collection
    /// let context = StandaloneBlockContext::new();
    /// let mut factory = LaunchpadFactory::new(
    ///     "1.0.0".to_string(), 86400, 500, &context
    /// );
    /// let collection_id = factory.create_collection(
    ///     "Test Bonds".to_string(),
    ///     "TBND".to_string(),
    ///     None, None, None,
    ///     &context
    /// );
    ///
    /// // Retrieve the collection
    /// if let Some(collection) = factory.get_collection(&collection_id) {
    ///     println!("Found collection: {}", collection.name);
    /// }
    /// ```
    pub fn get_collection(&self, collection_id: &str) -> Option<&OrbitalBondCollection> {
        self.collections.get(collection_id)
    }

    /// # Get Mutable Collection Reference
    ///
    /// Retrieves a mutable reference to a collection for direct modification.
    /// Use with caution as it bypasses factory-level validation.
    ///
    /// # Parameters
    /// * `collection_id` - The unique identifier of the collection to retrieve
    ///
    /// # Returns
    /// * `Some(&mut OrbitalBondCollection)` - Mutable reference to the collection if found
    /// * `None` - If no collection exists with the given ID
    ///
    /// # Example
    /// ```
    /// use slop::contracts::LaunchpadFactory;
    /// use slop::utils::StandaloneBlockContext;
    ///
    /// // Create factory and a collection
    /// let context = StandaloneBlockContext::new();
    /// let mut factory = LaunchpadFactory::new(
    ///     "1.0.0".to_string(), 86400, 500, &context
    /// );
    /// let collection_id = factory.create_collection(
    ///     "Test Bonds".to_string(),
    ///     "TBND".to_string(),
    ///     None, None, None,
    ///     &context
    /// );
    ///
    /// // Get mutable reference and modify the collection
    /// if let Some(collection) = factory.get_collection_mut(&collection_id) {
    ///     collection.deactivate();
    ///     assert!(!collection.is_active());
    /// }
    /// ```
    pub fn get_collection_mut(&mut self, collection_id: &str) -> Option<&mut OrbitalBondCollection> {
        self.collections.get_mut(collection_id)
    }

    /// # Get All Collections
    ///
    /// Retrieves all bond collections registered with this factory.
    ///
    /// # Returns
    /// A vector of references to all bond collections
    ///
    /// # Example
    /// ```
    /// use slop::contracts::LaunchpadFactory;
    /// use slop::utils::StandaloneBlockContext;
    ///
    /// let context = StandaloneBlockContext::new();
    /// let mut factory = LaunchpadFactory::new("1.0.0".to_string(), 86400, 500, &context);
    ///
    /// // Create a few collections
    /// factory.create_collection("Alpha Bonds".to_string(), "ABND".to_string(), None, None, None, &context);
    /// factory.create_collection("Beta Bonds".to_string(), "BBND".to_string(), None, None, None, &context);
    ///
    /// // Get all collections
    /// let all_collections = factory.get_all_collections();
    /// assert_eq!(all_collections.len(), 2);
    /// ```
    pub fn get_all_collections(&self) -> Vec<&OrbitalBondCollection> {
        self.collections.values().collect()
    }

    /// # Get Active Collections
    ///
    /// Retrieves only the active bond collections registered with this factory.
    /// Collections can be deactivated but remain in the registry, this method
    /// filters out deactivated collections.
    ///
    /// # Returns
    /// A vector of references to active bond collections
    ///
    /// # Example
    /// ```
    /// use slop::contracts::LaunchpadFactory;
    /// use slop::utils::StandaloneBlockContext;
    ///
    /// let context = StandaloneBlockContext::new();
    /// let mut factory = LaunchpadFactory::new("1.0.0".to_string(), 86400, 500, &context);
    ///
    /// // Create collections
    /// let c1 = factory.create_collection(
    ///     "Alpha Bonds".to_string(), "ABND".to_string(), None, None, None, &context
    /// );
    /// let c2 = factory.create_collection(
    ///     "Beta Bonds".to_string(), "BBND".to_string(), None, None, None, &context
    /// );
    ///
    /// // Deactivate one collection
    /// factory.deactivate_collection(&c1).unwrap();
    ///
    /// // Get active collections
    /// let active = factory.get_active_collections();
    /// assert_eq!(active.len(), 1);
    /// assert_eq!(active[0].symbol, "BBND");
    /// ```
    pub fn get_active_collections(&self) -> Vec<&OrbitalBondCollection> {
        self.collections.values()
            .filter(|c| c.is_active())
            .collect()
    }

    /// # Count Total Collections
    ///
    /// Returns the total number of bond collections registered with this factory,
    /// including both active and inactive collections.
    ///
    /// # Returns
    /// The total number of collections as `usize`
    ///
    /// # Example
    /// ```
    /// use slop::contracts::LaunchpadFactory;
    /// use slop::utils::StandaloneBlockContext;
    ///
    /// let context = StandaloneBlockContext::new();
    /// let mut factory = LaunchpadFactory::new("1.0.0".to_string(), 86400, 500, &context);
    ///
    /// // Create collections
    /// factory.create_collection("Alpha".to_string(), "A".to_string(), None, None, None, &context);
    /// factory.create_collection("Beta".to_string(), "B".to_string(), None, None, None, &context);
    ///
    /// assert_eq!(factory.total_collections(), 2);
    /// ```
    pub fn total_collections(&self) -> usize {
        self.collections.len()
    }

    /// # Count Active Collections
    ///
    /// Returns the number of currently active bond collections.
    ///
    /// # Returns
    /// The number of active collections as `usize`
    ///
    /// # Example
    /// ```
    /// use slop::contracts::LaunchpadFactory;
    /// use slop::utils::StandaloneBlockContext;
    ///
    /// let context = StandaloneBlockContext::new();
    /// let mut factory = LaunchpadFactory::new("1.0.0".to_string(), 86400, 500, &context);
    ///
    /// // Create collections
    /// let c1 = factory.create_collection(
    ///     "Alpha".to_string(), "A".to_string(), None, None, None, &context
    /// );
    /// factory.create_collection("Beta".to_string(), "B".to_string(), None, None, None, &context);
    ///
    /// // Initially all collections are active
    /// assert_eq!(factory.active_collections_count(), 2);
    ///
    /// // Deactivate one collection
    /// factory.deactivate_collection(&c1).unwrap();
    /// assert_eq!(factory.active_collections_count(), 1);
    /// ```
    pub fn active_collections_count(&self) -> usize {
        self.get_active_collections().len()
    }

    /// # Check If Collection Exists
    ///
    /// Determines whether a collection with the specified ID exists in the factory.
    ///
    /// # Parameters
    /// * `collection_id` - The unique identifier of the collection to check
    ///
    /// # Returns
    /// `true` if the collection exists, `false` otherwise
    ///
    /// # Example
    /// ```
    /// use slop::contracts::LaunchpadFactory;
    /// use slop::utils::StandaloneBlockContext;
    ///
    /// let context = StandaloneBlockContext::new();
    /// let mut factory = LaunchpadFactory::new("1.0.0".to_string(), 86400, 500, &context);
    ///
    /// // Create a collection
    /// let cid = factory.create_collection(
    ///     "Test Bonds".to_string(), "TBND".to_string(), None, None, None, &context
    /// );
    ///
    /// // Check if collections exist
    /// assert!(factory.collection_exists(&cid));
    /// assert!(!factory.collection_exists("nonexistent-id"));
    /// ```
    pub fn collection_exists(&self, collection_id: &str) -> bool {
        self.collections.contains_key(collection_id)
    }

    /// # Calculate Total Value of All Collections
    ///
    /// Calculates the combined total value of all active bond collections.
    /// This represents the total debt obligation across all collections.
    ///
    /// # Parameters
    /// * `block_context` - Context providing current block information
    ///
    /// # Returns
    /// The total value as a `u64`
    ///
    /// # Example
    /// ```
    /// use slop::contracts::LaunchpadFactory;
    /// use slop::utils::StandaloneBlockContext;
    ///
    /// let context = StandaloneBlockContext::new();
    /// let mut factory = LaunchpadFactory::new("1.0.0".to_string(), 86400, 500, &context);
    ///
    /// // Create collections and mint bonds
    /// let c1 = factory.create_collection(
    ///     "Alpha".to_string(), "A".to_string(), None, None, None, &context
    /// );
    /// let c2 = factory.create_collection(
    ///     "Beta".to_string(), "B".to_string(), None, None, None, &context
    /// );
    ///
    /// // Mint bonds in both collections
    /// factory.mint_bond(&c1, "orbital-1".to_string(), 1000, "owner-1".to_string(), &context).unwrap();
    /// factory.mint_bond(&c2, "orbital-2".to_string(), 2000, "owner-2".to_string(), &context).unwrap();
    ///
    /// // Get total value (principal only, as interest accumulates over time)
    /// let total = factory.total_value(&context);
    /// assert!(total >= 3000); // At least the principal amount
    /// ```
    pub fn total_value<T: BlockContext>(&self, block_context: &T) -> u64 {
        // Use u128 for accumulation to prevent overflow
        let total = self.collections.values()
            .filter(|c| c.is_active())
            .fold(0u128, |acc, collection| acc + collection.total_value(block_context) as u128);
        
        // Check for overflow and saturate if needed
        if total > u64::MAX as u128 {
            return u64::MAX; // Return max value in case of overflow
        }
        
        total as u64
    }

    /// # Mint Bond in Collection
    ///
    /// Creates a new bond in the specified collection and returns the necessary
    /// token information. This is the primary method for bond issuance.
    ///
    /// # Parameters
    /// * `collection_id` - ID of the collection to mint the bond in
    /// * `orbital_token_id` - ID of the orbital token backing this bond
    /// * `amount` - Principal amount for the bond
    /// * `owner_id` - ID of the entity that will own the bond
    /// * `block_context` - Context providing current block information
    ///
    /// # Returns
    /// * `Ok((bond_id, alkane_token_id, AlkaneTransfer))` - On success, returns:
    ///   - The bond's unique identifier
    ///   - The alkane token identifier for redemption
    ///   - An AlkaneTransfer object representing the token
    /// * `Err(&'static str)` - Error message if minting fails
    ///
    /// # Errors
    /// * "Collection not found" - If the specified collection ID is invalid
    /// * Other errors propagated from the collection's mint_bond method
    ///
    /// # Example
    /// ```
    /// use slop::contracts::LaunchpadFactory;
    /// use slop::utils::StandaloneBlockContext;
    ///
    /// let context = StandaloneBlockContext::new();
    /// let mut factory = LaunchpadFactory::new(
    ///     "1.0.0".to_string(), 86400, 500, &context
    /// );
    ///
    /// // Create a collection
    /// let collection_id = factory.create_collection(
    ///     "Test Bonds".to_string(),
    ///     "TBND".to_string(),
    ///     None, None, None,
    ///     &context
    /// );
    ///
    /// // Mint a bond
    /// let result = factory.mint_bond(
    ///     &collection_id,
    ///     "orbital-123".to_string(),
    ///     1000,  // 1000 tokens as principal
    ///     "user-xyz".to_string(),
    ///     &context
    /// );
    ///
    /// if let Ok((bond_id, alkane_id, transfer)) = result {
    ///     println!("Bond created: {}", bond_id);
    ///     println!("AlkaneTransfer value: {}", transfer.value);
    /// }
    /// ```
    pub fn mint_bond<T: BlockContext>(
        &mut self,
        collection_id: &str,
        orbital_token_id: String,
        amount: u64,
        owner_id: String,
        block_context: &T,
    ) -> Result<(String, String, AlkaneTransfer), &'static str> {
        let collection = self.collections.get_mut(collection_id)
            .ok_or("Collection not found")?;

        // Pass all parameters including owner_id to the collection
        collection.mint_bond(orbital_token_id, amount, owner_id, block_context)
    }

    /// # Securely Redeem Bond with Transaction Context
    ///
    /// Redeems a bond using transaction context to verify token ownership.
    /// This is the most secure way to redeem a bond as it ensures the redeemer
    /// actually possesses the orbital token.
    ///
    /// # Parameters
    /// * `collection_id` - ID of the collection containing the bond
    /// * `tx_context` - Transaction context that proves orbital token ownership
    /// * `redeemer_id` - ID of the entity redeeming the bond
    /// * `block_context` - Context providing current block information
    ///
    /// # Returns
    /// * `Ok(u64)` - The redeemed amount (principal + interest) if successful
    /// * `Err(&'static str)` - Error message if redemption fails
    ///
    /// # Errors
    /// * "Collection not found" - If the specified collection ID is invalid
    /// * "No orbital token in transaction context" - If the tx context doesn't contain a valid orbital token
    /// * Other errors propagated from the collection's secure redeem_bond method
    ///
    /// # Example
    /// ```
    /// use slop::contracts::LaunchpadFactory;
    /// use slop::utils::StandaloneBlockContext;
    /// use slop::tests::mock::{MockTransactionContext, TransactionContextExt};
    ///
    /// // Create factory, collection and bond
    /// let context = StandaloneBlockContext::new();
    /// let mut factory = LaunchpadFactory::new(
    ///     "1.0.0".to_string(), 50, 500, &context
    /// );
    /// let collection_id = factory.create_collection(
    ///     "Test".to_string(), "TEST".to_string(),
    ///     None, None, None, &context
    /// );
    ///
    /// // Mint a bond with a specific orbital ID
    /// let orbital_id = "orbital-123".to_string();
    /// factory.mint_bond(
    ///     &collection_id, orbital_id.clone(), 1000, "owner-1".to_string(), &context
    /// ).unwrap();
    ///
    /// // Create a context for maturity verification
    /// let mature_context = StandaloneBlockContext::new().with_offset(context.get_current_block_height() + 51);
    ///
    /// // Create transaction context with the orbital token (proves ownership)
    /// let tx_context = MockTransactionContext::new()
    ///     .with_orbital_token(&orbital_id)
    ///     .with_transaction_id("tx-123");
    ///
    /// // Redeem the bond with secure verification
    /// let result = factory.redeem_bond_secure(
    ///     &collection_id, 
    ///     &tx_context,
    ///     "redeemer-1", 
    ///     &mature_context
    /// );
    /// if let Ok(amount) = result {
    ///     println!("Redeemed {} tokens", amount);
    /// }
    /// ```
    pub fn redeem_bond_secure<T: BlockContext, C: TransactionContextExt>(
        &mut self,
        collection_id: &str,
        tx_context: &C,
        redeemer_id: &str,
        block_context: &T,
    ) -> Result<u64, &'static str> {
        let collection = self.collections.get_mut(collection_id)
            .ok_or("Collection not found")?;
        
        // Use the secure redemption method that verifies token ownership
        collection.redeem_bond_secure(tx_context, redeemer_id, block_context)
    }

    
    /// # Securely Redeem Bond using Alkane Token ID with Transaction Context
    ///
    /// Redeems a bond using its alkane token ID and transaction context to verify token ownership.
    /// This is the secure version of redeem_bond_by_alkane.
    ///
    /// # Parameters
    /// * `collection_id` - ID of the collection containing the bond
    /// * `tx_context` - Transaction context that proves token ownership
    /// * `alkane_token_id` - ID of the alkane token representing the bond
    /// * `redeemer_id` - ID of the entity redeeming the bond
    /// * `block_context` - Context providing current block information
    ///
    /// # Returns
    /// * `Ok(u64)` - The redeemed amount (principal + interest) if successful
    /// * `Err(&'static str)` - Error message if redemption fails
    ///
    /// # Errors
    /// * "Collection not found" - If the specified collection ID is invalid
    /// * "Invalid alkane token format" - If the alkane token ID format is incorrect
    /// * "Bond not found" - If the bond associated with the alkane token doesn't exist
    /// * "No orbital token in transaction context" - If the tx context doesn't contain a valid orbital token
    /// * Other errors propagated from the collection's secure redeem_bond method
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

    /// # Deactivate a Collection
    ///
    /// Deactivates a bond collection, preventing further bond minting while still
    /// allowing existing bonds to mature and be redeemed.
    ///
    /// # Parameters
    /// * `collection_id` - ID of the collection to deactivate
    ///
    /// # Returns
    /// * `Ok(())` - If the collection was successfully deactivated
    /// * `Err(&'static str)` - Error message if deactivation fails
    ///
    /// # Errors
    /// * "Collection not found" - If the specified collection ID is invalid
    ///
    /// # Example
    /// ```
    /// use slop::contracts::LaunchpadFactory;
    /// use slop::utils::StandaloneBlockContext;
    ///
    /// let context = StandaloneBlockContext::new();
    /// let mut factory = LaunchpadFactory::new("1.0.0".to_string(), 86400, 500, &context);
    ///
    /// // Create a collection
    /// let cid = factory.create_collection(
    ///     "Test".to_string(), "TEST".to_string(), None, None, None, &context
    /// );
    ///
    /// // Deactivate the collection
    /// factory.deactivate_collection(&cid).unwrap();
    ///
    /// // Verify collection is deactivated
    /// let collection = factory.get_collection(&cid).unwrap();
    /// assert!(!collection.is_active());
    ///
    /// // Attempt to mint should fail for deactivated collection
    /// let mint_result = factory.mint_bond(
    ///     &cid, "orbital-1".to_string(), 1000, "owner-1".to_string(), &context
    /// );
    /// assert!(mint_result.is_err());
    /// ```
    pub fn deactivate_collection(&mut self, collection_id: &str) -> Result<(), &'static str> {
        let collection = self.collections.get_mut(collection_id)
            .ok_or("Collection not found")?;

        collection.deactivate();
        Ok(())
    }

    /// # Reactivate a Collection
    ///
    /// Reactivates a previously deactivated bond collection, allowing new bonds to be minted.
    ///
    /// # Parameters
    /// * `collection_id` - ID of the collection to reactivate
    ///
    /// # Returns
    /// * `Ok(())` - If the collection was successfully reactivated
    /// * `Err(&'static str)` - Error message if reactivation fails
    ///
    /// # Errors
    /// * "Collection not found" - If the specified collection ID is invalid
    ///
    /// # Example
    /// ```
    /// use slop::contracts::LaunchpadFactory;
    /// use slop::utils::StandaloneBlockContext;
    ///
    /// let context = StandaloneBlockContext::new();
    /// let mut factory = LaunchpadFactory::new("1.0.0".to_string(), 86400, 500, &context);
    ///
    /// // Create and deactivate a collection
    /// let cid = factory.create_collection(
    ///     "Test".to_string(), "TEST".to_string(), None, None, None, &context
    /// );
    /// factory.deactivate_collection(&cid).unwrap();
    ///
    /// // Reactivate the collection
    /// factory.reactivate_collection(&cid).unwrap();
    ///
    /// // Verify collection is active
    /// let collection = factory.get_collection(&cid).unwrap();
    /// assert!(collection.is_active());
    ///
    /// // Now minting should succeed
    /// let mint_result = factory.mint_bond(
    ///     &cid, "orbital-1".to_string(), 1000, "owner-1".to_string(), &context
    /// );
    /// assert!(mint_result.is_ok());
    /// ```
    pub fn reactivate_collection(&mut self, collection_id: &str) -> Result<(), &'static str> {
        let collection = self.collections.get_mut(collection_id)
            .ok_or("Collection not found")?;

        collection.reactivate();
        Ok(())
    }

    /// # Find Collections by Custom Predicate
    ///
    /// Searches for collections that match a custom filtering criteria.
    /// This generic method allows for flexible collection filtering based on any condition.
    ///
    /// # Parameters
    /// * `predicate` - A function/closure that takes a collection reference and returns a boolean
    ///
    /// # Returns
    /// A vector of references to collections that satisfy the predicate
    ///
    /// # Type Parameters
    /// * `F` - Function type that implements `Fn(&OrbitalBondCollection) -> bool`
    ///
    /// # Example
    /// ```
    /// use slop::contracts::LaunchpadFactory;
    /// use slop::utils::StandaloneBlockContext;
    ///
    /// let context = StandaloneBlockContext::new();
    /// let mut factory = LaunchpadFactory::new("1.0.0".to_string(), 86400, 500, &context);
    ///
    /// // Create collections with different parameters
    /// factory.create_collection(
    ///     "High Yield Bonds".to_string(), "HYB".to_string(), 
    ///     None, Some(1000), None, &context
    /// );
    /// factory.create_collection(
    ///     "Standard Bonds".to_string(), "STD".to_string(), 
    ///     None, Some(500), None, &context
    /// );
    /// factory.create_collection(
    ///     "Low Risk Bonds".to_string(), "LRB".to_string(), 
    ///     None, Some(300), None, &context
    /// );
    ///
    /// // Find collections with interest rate > 500 bps
    /// let high_interest = factory.find_collections(|c| c.interest_rate_bps > 500);
    /// assert_eq!(high_interest.len(), 1);
    /// assert_eq!(high_interest[0].symbol, "HYB");
    /// ```
    pub fn find_collections<F>(&self, predicate: F) -> Vec<&OrbitalBondCollection>
    where
        F: Fn(&OrbitalBondCollection) -> bool,
    {
        self.collections.values()
            .filter(|&collection| predicate(collection))
            .collect()
    }

    /// # Find Collections by Name
    ///
    /// Searches for collections with names containing the specified text (case-insensitive).
    ///
    /// # Parameters
    /// * `name_part` - Text to search for in collection names
    ///
    /// # Returns
    /// A vector of references to collections with matching names
    ///
    /// # Example
    /// ```
    /// use slop::contracts::LaunchpadFactory;
    /// use slop::utils::StandaloneBlockContext;
    ///
    /// let context = StandaloneBlockContext::new();
    /// let mut factory = LaunchpadFactory::new("1.0.0".to_string(), 86400, 500, &context);
    ///
    /// // Create collections with different names
    /// factory.create_collection("Alpha Premium Bonds".to_string(), "APB".to_string(), None, None, None, &context);
    /// factory.create_collection("Beta Standard Bonds".to_string(), "BSB".to_string(), None, None, None, &context);
    /// factory.create_collection("Alpha Plus".to_string(), "APL".to_string(), None, None, None, &context);
    ///
    /// // Find collections with "alpha" in the name
    /// let alpha_collections = factory.find_collections_by_name("alpha");
    /// assert_eq!(alpha_collections.len(), 2);
    /// ```
    pub fn find_collections_by_name(&self, name_part: &str) -> Vec<&OrbitalBondCollection> {
        self.find_collections(|c| c.name.to_lowercase().contains(&name_part.to_lowercase()))
    }

    /// # Find Collections by Symbol
    ///
    /// Searches for collections with the exact symbol specified (case-sensitive).
    ///
    /// # Parameters
    /// * `symbol` - The symbol/ticker to search for
    ///
    /// # Returns
    /// A vector of references to collections with the matching symbol
    ///
    /// # Example
    /// ```
    /// use slop::contracts::LaunchpadFactory;
    /// use slop::utils::StandaloneBlockContext;
    ///
    /// let context = StandaloneBlockContext::new();
    /// let mut factory = LaunchpadFactory::new("1.0.0".to_string(), 86400, 500, &context);
    ///
    /// // Create collections with different symbols
    /// factory.create_collection("Alpha Bonds".to_string(), "ABND".to_string(), None, None, None, &context);
    /// factory.create_collection("Beta Bonds".to_string(), "BBND".to_string(), None, None, None, &context);
    ///
    /// // Find collection by symbol (exact match)
    /// let abnd_collections = factory.find_collections_by_symbol("ABND");
    /// assert_eq!(abnd_collections.len(), 1);
    /// assert_eq!(abnd_collections[0].name, "Alpha Bonds");
    /// ```
    pub fn find_collections_by_symbol(&self, symbol: &str) -> Vec<&OrbitalBondCollection> {
        self.find_collections(|c| c.symbol == symbol)
    }

    /// # Update Default Parameters
    ///
    /// Updates the default parameters used when creating new collections without
    /// explicitly specified parameters.
    ///
    /// # Parameters
    /// * `maturity_blocks` - Optional new default for maturity period in blocks
    /// * `interest_rate_bps` - Optional new default for interest rate in basis points
    ///
    /// # Example
    /// ```
    /// use slop::contracts::LaunchpadFactory;
    /// use slop::utils::StandaloneBlockContext;
    ///
    /// let context = StandaloneBlockContext::new();
    /// let mut factory = LaunchpadFactory::new("1.0.0".to_string(), 86400, 500, &context);
    ///
    /// // Update default interest rate to 800 bps (8%)
    /// factory.update_defaults(None, Some(800));
    ///
    /// // Create a collection with default parameters
    /// let cid = factory.create_collection(
    ///     "New Collection".to_string(), "NEWC".to_string(), 
    ///     None, None, None, &context
    /// );
    ///
    /// // Verify the new default was used
    /// let collection = factory.get_collection(&cid).unwrap();
    /// assert_eq!(collection.interest_rate_bps, 800);
    /// ```
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
            "owner-1".to_string(),
            &context,
        );

        assert!(bond_result.is_ok());
        let (bond_id, alkane_id, alkane_transfer) = bond_result.unwrap();
        
        // Verify the AlkaneTransfer is correctly formed
        assert_eq!(alkane_transfer.value, 1000);
        assert!(alkane_transfer.id.len() > 0);

        // Verify the bond exists in the collection
        let collection = factory.get_collection(&collection_id).unwrap();
        assert_eq!(collection.total_bonds(), 1);
        let bond = collection.get_bond_by_orbital("orbital-1").unwrap();
        assert_eq!(bond.amount, 1000);

        // Create a new context at maturity
        let maturity_context = StandaloneBlockContext::new()
            .with_offset(context.get_current_block_height() as u64 + 51);

        // Redeem the bond
        // Create a mock transaction context with the orbital token
        struct MockTxContext {
            orbital_id: String,
        }
        
        impl TransactionContextExt for MockTxContext {
            fn orbital_token_id(&self) -> Result<String, anyhow::Error> {
                Ok(self.orbital_id.clone())
            }
        }
        
        // Use variable names that are in scope
        let tx_context = MockTxContext {
            orbital_id: "orbital-1".to_string(),
        };
        
        let redemption_result = factory.redeem_bond_secure(
            &collection_id,
            &tx_context,
            "redeemer-id",
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
            "owner-1".to_string(),
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
            "owner-1".to_string(),
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
