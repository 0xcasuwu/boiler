use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::models::bond::{Bond, BondStatus};
use crate::utils::BlockContext;
use crate::utils::transaction_context::TransactionContextExt;
use crate::alkanes_support::parcel::AlkaneTransfer;

/// # OrbitalBondCollection
/// 
/// A collection manager for bonds backed by orbital tokens that provides:
///
/// - Bond creation and management (minting, redeeming, canceling)
/// - Orbital token integration for authentication
/// - Interest calculation and maturity tracking
/// - Collection-wide operations and queries
///
/// This collection allows orbital tokens to serve as both bonds and authentication tokens,
/// creating a secure bond system with built-in ownership verification.
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
    /// Creates a new OrbitalBondCollection with the specified parameters
    ///
    /// # Parameters
    /// * `id` - Unique identifier for this collection
    /// * `name` - Human-readable name for the collection
    /// * `symbol` - Short ticker symbol for the collection
    /// * `interest_rate_bps` - Interest rate in basis points (1/100th of a percent, e.g. 500 = 5%)
    /// * `maturity_blocks` - Number of blocks that must pass before bonds can be redeemed
    /// * `block_context` - Context providing current block information
    ///
    /// # Returns
    /// A new OrbitalBondCollection instance with no bonds
    ///
/// # Example
/// ```
/// use slop::contracts::OrbitalBondCollection;
/// use slop::utils::StandaloneBlockContext;
///
/// let context = StandaloneBlockContext::new();
/// let collection = OrbitalBondCollection::new(
///     "collection-1".to_string(),
///     "Test Bonds".to_string(),
///     "TBND".to_string(),
///     500, // 5% interest
///     100, // 100 blocks to mature
///     &context
/// );
/// ```
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
    
    /// Sets the collection description and returns self for method chaining
    ///
    /// # Parameters
    /// * `description` - Text description of this bond collection
    ///
    /// # Returns
    /// Self with the description added, allowing for method chaining
    ///
    /// # Example
    /// ```
    /// use slop::contracts::OrbitalBondCollection;
    /// use slop::utils::StandaloneBlockContext;
    /// 
    /// let context = StandaloneBlockContext::new();
    /// let collection = OrbitalBondCollection::new(
    ///     "collection-1".to_string(),
    ///     "Test Bonds".to_string(),
    ///     "TBND".to_string(),
    ///     500,
    ///     100,
    ///     &context
    /// ).with_description("Corporate bonds with 5% interest rate".to_string());
    /// ```
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
    
    /// Creates a new bond with an orbital token as authentication
    ///
    /// # Parameters
    /// * `orbital_token_id` - Unique identifier for the orbital token backing this bond
    /// * `amount` - Principal amount for the bond
    /// * `block_context` - Context providing current block information
    ///
    /// # Returns
    /// * `Ok((String, String, AlkaneTransfer))` - Returns the bond ID, alkane token ID, and an AlkaneTransfer object
    /// * `Err(&'static str)` - Error message if the operation failed
    ///
    /// # Errors
    /// * "Collection is inactive" - If the collection has been deactivated
    /// * "Orbital token already has a bond" - If the orbital token is already used for another bond
    ///
    /// # Example
    /// ```
    /// use slop::contracts::OrbitalBondCollection;
    /// use slop::utils::StandaloneBlockContext;
    /// 
    /// let context = StandaloneBlockContext::new();
    /// let mut collection = OrbitalBondCollection::new(
    ///     "collection-1".to_string(),
    ///     "Test Bonds".to_string(),
    ///     "TBND".to_string(),
    ///     500, // 5% interest
    ///     100, // 100 blocks to mature
    ///     &context
    /// );
    /// 
    /// // Mint a new bond
    /// let result = collection.mint_bond(
    ///     "orbital-123".to_string(),  // orbital token backing this bond
    ///     1000,                       // 1000 tokens as principal
    ///     "owner-abc".to_string(),    // owner who will receive the alkane token
    ///     &context
    /// );
    /// 
    /// if let Ok((bond_id, alkane_token_id, alkane_transfer)) = result {
    ///     println!("Created bond: {}", bond_id);
    ///     println!("Alkane token: {}", alkane_token_id);
    ///     println!("Transfer amount: {}", alkane_transfer.value);
    /// }
    /// ```
    pub fn mint_bond<T: BlockContext>(
        &mut self,
        orbital_token_id: String,
        amount: u64,
        owner_id: String,
        block_context: &T,
    ) -> Result<(String, String, AlkaneTransfer), &'static str> {
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
        
        // Generate alkane token ID based on bond ID
        let alkane_token_id = format!("alkane-{}", bond_id);
        
        // Store mappings for orbital token and alkane token
        self.orbital_to_bond.insert(orbital_token_id.clone(), bond_id.clone());
        
        // Create an AlkaneTransfer object representing the token
        // Convert string IDs to Vec<u8> as expected by AlkaneTransfer
        let token_bytes = alkane_token_id.clone().into_bytes();
        
        // Create the transfer object
        // The owner_id specifies who will receive the token in the transaction
        let alkane_transfer = AlkaneTransfer {
            id: token_bytes,
            value: amount as u128,
            from: None,
            to: Some(owner_id.clone()), // Use owner_id as the recipient (the person who brought the diesel)
        };
        
        // Return the bond ID, alkane token ID, and transfer object
        Ok((bond_id, alkane_token_id, alkane_transfer))
    }
    
    /// Gets bond details by bond ID
    ///
    /// # Parameters
    /// * `bond_id` - The unique identifier of the bond to retrieve
    ///
    /// # Returns
    /// * `Some(&Bond)` - Reference to the bond if found
    /// * `None` - If no bond exists with the given ID
    ///
    /// # Example
    /// ```
    /// use slop::contracts::OrbitalBondCollection;
    /// use slop::utils::StandaloneBlockContext;
    /// 
    /// let context = StandaloneBlockContext::new();
    /// let mut collection = OrbitalBondCollection::new(
    ///     "collection-1".to_string(),
    ///     "Test Bonds".to_string(),
    ///     "TBND".to_string(),
    ///     500,
    ///     100,
    ///     &context
    /// );
    /// 
    /// // Mint a bond to have something to query
    /// let (bond_id, _, _) = collection.mint_bond(
    ///     "orbital-xyz".to_string(), 
    ///     1000,
    ///     "owner-123".to_string(),
    ///     &context
    /// ).unwrap();
    /// 
    /// // Look up the bond by ID
    /// if let Some(bond) = collection.get_bond(&bond_id) {
    ///     println!("Bond amount: {}", bond.amount);
    /// }
    /// ```
    pub fn get_bond(&self, bond_id: &str) -> Option<&Bond> {
        self.bonds.get(bond_id)
    }
    
    /// Gets bond details by orbital token ID
    ///
    /// # Parameters
    /// * `orbital_token_id` - The orbital token ID associated with the bond
    ///
    /// # Returns
    /// * `Some(&Bond)` - Reference to the bond if found
    /// * `None` - If no bond exists for the given orbital token
    ///
    /// # Example
    /// ```
    /// use slop::contracts::OrbitalBondCollection;
    /// use slop::utils::StandaloneBlockContext;
    /// 
    /// let context = StandaloneBlockContext::new();
    /// let mut collection = OrbitalBondCollection::new(
    ///     "collection-1".to_string(),
    ///     "Test Bonds".to_string(),
    ///     "TBND".to_string(),
    ///     500,
    ///     100,
    ///     &context
    /// );
    /// 
    /// // Mint a bond with a specific orbital token ID
    /// collection.mint_bond(
    ///     "orbital-xyz".to_string(), 
    ///     1000,
    ///     "owner-123".to_string(),
    ///     &context
    /// ).unwrap();
    /// 
    /// // Look up the bond by orbital token ID
    /// if let Some(bond) = collection.get_bond_by_orbital("orbital-xyz") {
    ///     println!("Bond ID: {}", bond.id);
    /// }
    /// ```
    pub fn get_bond_by_orbital(&self, orbital_token_id: &str) -> Option<&Bond> {
        self.orbital_to_bond.get(orbital_token_id)
            .and_then(|bond_id| self.bonds.get(bond_id))
    }
    
    /// Retrieves the alkane token ID associated with a bond
    ///
    /// # Parameters
    /// * `bond_id` - The ID of the bond
    ///
    /// # Returns
    /// * `Some(String)` - The alkane token ID if the bond exists
    /// * `None` - If no bond exists with the given ID
    ///
    /// # Example
    /// ```
    /// use slop::contracts::OrbitalBondCollection;
    /// use slop::utils::StandaloneBlockContext;
    /// 
    /// let context = StandaloneBlockContext::new();
    /// let mut collection = OrbitalBondCollection::new(
    ///     "collection-1".to_string(),
    ///     "Test Bonds".to_string(),
    ///     "TBND".to_string(),
    ///     500,
    ///     100,
    ///     &context
    /// );
    /// 
    /// // Mint a bond to get a valid bond ID
    /// let (bond_id, _, _) = collection.mint_bond(
    ///     "orbital-xyz".to_string(), 
    ///     1000,
    ///     "owner-123".to_string(),
    ///     &context
    /// ).unwrap();
    /// 
    /// // Get the corresponding alkane token ID
    /// if let Some(alkane_id) = collection.get_alkane_token_for_bond(&bond_id) {
    ///     println!("Alkane token ID: {}", alkane_id);
    /// }
    /// ```
    pub fn get_alkane_token_for_bond(&self, bond_id: &str) -> Option<String> {
        if self.bonds.contains_key(bond_id) {
            Some(format!("alkane-{}", bond_id))
        } else {
            None
        }
    }
    
    /// Redeems a bond using a transaction context with verified token ownership
    ///
    /// This secure method extracts the orbital token directly from the transaction context,
    /// ensuring that only the rightful owner of the token can redeem the bond.
    ///
    /// # Parameters
    /// * `tx_context` - Transaction context containing the orbital token (proves ownership)
    /// * `redeemer_id` - Identifier of the person redeeming the bond
    /// * `block_context` - Context providing current block information
    ///
    /// # Returns
    /// * `Ok(u64)` - The redemption amount (principal + interest) if successful
    /// * `Err(&'static str)` - Error message if the redemption failed
    ///
    /// # Errors
    /// * "No orbital token in transaction context" - If the context doesn't contain a token
    /// * "Orbital token has no associated bond" - If the token doesn't exist in this collection
    /// * "Bond not found" - If the bond record is missing (should not happen)
    /// * "Bond is not active" - If the bond has already been redeemed or canceled
    /// * "Bond has not reached maturity" - If the bond's maturity period hasn't elapsed
    ///
    /// # Example
    /// ```
    /// use slop::contracts::OrbitalBondCollection;
    /// use slop::utils::StandaloneBlockContext;
    /// use slop::tests::mock::{MockTransactionContext, TransactionContextExt};
    /// 
    /// let context = StandaloneBlockContext::new();
    /// let mut collection = OrbitalBondCollection::new(
    ///     "collection-1".to_string(),
    ///     "Test Bonds".to_string(),
    ///     "TBND".to_string(),
    ///     500, // 5% interest
    ///     50,  // 50 blocks to mature
    ///     &context
    /// );
    /// 
    /// // Mint a bond
    /// let orbital_id = "orbital-xyz".to_string();
    /// let (bond_id, alkane_token_id, _) = collection.mint_bond(
    ///     orbital_id.clone(),
    ///     1000,
    ///     "owner-123".to_string(),
    ///     &context
    /// ).unwrap();
    /// 
    /// // Create a context that's past the maturity date
    /// let mature_context = StandaloneBlockContext::new().with_offset(100);
    /// 
    /// // Create a transaction context that proves token ownership
    /// let tx_context = MockTransactionContext::new()
    ///     .with_orbital_token(&orbital_id)
    ///     .with_transaction_id("tx-123");
    /// 
    /// // Attempt redemption with verified token ownership
    /// let redemption_result = collection.redeem_bond_secure(
    ///     &tx_context,
    ///     "redeemer-456",
    ///     &mature_context
    /// );
    /// 
    /// if let Ok(redemption_amount) = redemption_result {
    ///     println!("Redeemed for {} tokens", redemption_amount);
    /// }
    /// ```
    pub fn redeem_bond_secure<T: BlockContext, C: TransactionContextExt>(
        &mut self,
        tx_context: &C,
        redeemer_id: &str,
        block_context: &T,
    ) -> Result<u64, &'static str> {
        // Extract the orbital token from the transaction context (proves ownership)
        let orbital_token_id = tx_context.orbital_token_id()
            .map_err(|_| "No orbital token in transaction context")?;
            
        self.redeem_bond_internal(&orbital_token_id, redeemer_id, block_context)
    }
    
    /// Private internal method for bond redemption with a verified orbital token ID
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
    
    
    /// Returns all bonds in the collection
    ///
    /// # Returns
    /// A vector containing references to all bonds in the collection
    ///
    /// # Example
    /// ```
    /// use slop::contracts::OrbitalBondCollection;
    /// use slop::utils::StandaloneBlockContext;
    /// 
    /// let context = StandaloneBlockContext::new();
    /// let mut collection = OrbitalBondCollection::new(
    ///     "collection-1".to_string(),
    ///     "Test Bonds".to_string(),
    ///     "TBND".to_string(),
    ///     500,
    ///     100,
    ///     &context
    /// );
    /// 
    /// // Mint a few bonds
    /// collection.mint_bond("orbital-1".to_string(), 1000, "owner-1".to_string(), &context).unwrap();
    /// collection.mint_bond("orbital-2".to_string(), 2000, "owner-2".to_string(), &context).unwrap();
    /// 
    /// // Get all bonds regardless of status
    /// let all_bonds = collection.get_all_bonds();
    /// println!("Total bonds: {}", all_bonds.len());
    /// ```
    pub fn get_all_bonds(&self) -> Vec<&Bond> {
        self.bonds.values().collect()
    }
    
    /// Returns only active bonds (not redeemed or canceled)
    ///
    /// # Returns
    /// A vector containing references to all active bonds in the collection
    ///
    /// # Example
    /// ```
    /// use slop::contracts::OrbitalBondCollection;
    /// use slop::utils::StandaloneBlockContext;
    /// 
    /// let context = StandaloneBlockContext::new();
    /// let mut collection = OrbitalBondCollection::new(
    ///     "collection-1".to_string(),
    ///     "Test Bonds".to_string(),
    ///     "TBND".to_string(),
    ///     500,
    ///     100,
    ///     &context
    /// );
    /// 
    /// // Mint a few bonds
    /// collection.mint_bond("orbital-1".to_string(), 1000, "owner-1".to_string(), &context).unwrap();
    /// collection.mint_bond("orbital-2".to_string(), 2000, "owner-2".to_string(), &context).unwrap();
    /// 
    /// // Cancel one bond
    /// let bond_id = collection.get_bond_by_orbital("orbital-2").unwrap().id.clone();
    /// collection.cancel_bond(&bond_id).unwrap();
    /// 
    /// // Get only active bonds
    /// let active_bonds = collection.get_active_bonds();
    /// println!("Active bonds: {}", active_bonds.len()); // Should be 1
    /// ```
    pub fn get_active_bonds(&self) -> Vec<&Bond> {
        self.bonds.values()
            .filter(|b| b.status == BondStatus::Active)
            .collect()
    }
    
    /// Returns bonds that have reached maturity and can be redeemed
    ///
    /// # Parameters
    /// * `block_context` - Context providing current block information
    ///
    /// # Returns
    /// A vector containing references to all mature bonds that can be redeemed
    ///
    /// # Example
    /// ```
    /// use slop::contracts::OrbitalBondCollection;
    /// use slop::utils::StandaloneBlockContext;
    /// 
    /// let context = StandaloneBlockContext::new();
    /// let mut collection = OrbitalBondCollection::new(
    ///     "collection-1".to_string(),
    ///     "Test Bonds".to_string(),
    ///     "TBND".to_string(),
    ///     500, // 5% interest
    ///     50, // 50 blocks to mature
    ///     &context
    /// );
    /// 
    /// // Mint a few bonds
    /// collection.mint_bond("orbital-1".to_string(), 1000, "owner-1".to_string(), &context).unwrap();
    /// collection.mint_bond("orbital-2".to_string(), 2000, "owner-2".to_string(), &context).unwrap();
    /// 
    /// // Create a context for current time (no bonds mature yet)
    /// let current_context = StandaloneBlockContext::new();
    /// let current_mature_bonds = collection.get_mature_bonds(&current_context);
    /// println!("Currently mature bonds: {}", current_mature_bonds.len()); // 0
    /// 
    /// // Create a context for future time (bonds should be mature)
    /// let future_context = StandaloneBlockContext::new().with_offset(100);
    /// let future_mature_bonds = collection.get_mature_bonds(&future_context);
    /// println!("Mature bonds in the future: {}", future_mature_bonds.len()); // 2
    /// ```
    pub fn get_mature_bonds<T: BlockContext>(&self, block_context: &T) -> Vec<&Bond> {
        self.bonds.values()
            .filter(|b| b.is_mature(block_context))
            .collect()
    }
    
    /// Calculates the total value of all active bonds in the collection
    ///
    /// # Parameters
    /// * `block_context` - Context providing current block information
    ///
    /// # Returns
    /// The sum of the current values of all active bonds, including accrued interest
    ///
    /// # Example
    /// ```
    /// use slop::contracts::OrbitalBondCollection;
    /// use slop::utils::StandaloneBlockContext;
    /// 
    /// let context = StandaloneBlockContext::new();
    /// let mut collection = OrbitalBondCollection::new(
    ///     "collection-1".to_string(),
    ///     "Test Bonds".to_string(),
    ///     "TBND".to_string(),
    ///     500, // 5% interest
    ///     100, // 100 blocks to mature
    ///     &context
    /// );
    /// 
    /// // Mint a few bonds with different values
    /// collection.mint_bond("orbital-1".to_string(), 1000, "owner-1".to_string(), &context).unwrap();
    /// collection.mint_bond("orbital-2".to_string(), 2000, "owner-2".to_string(), &context).unwrap();
    /// 
    /// // Calculate the total value
    /// let total_value = collection.total_value(&context);
    /// println!("Total portfolio value: {}", total_value); // 3000
    /// 
    /// // With a more advanced context (accrued interest)
    /// let future_context = StandaloneBlockContext::new().with_offset(50); // Halfway to maturity
    /// let future_value = collection.total_value(&future_context);
    /// println!("Future portfolio value: {}", future_value); // 3075 (3000 + half interest)
    /// ```
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
    
    /// Deactivates the collection, preventing new bonds from being created
    ///
    /// This does not affect existing bonds, which can still be redeemed when mature.
    ///
    /// # Example
    /// ```
    /// use slop::contracts::OrbitalBondCollection;
    /// use slop::utils::StandaloneBlockContext;
    /// 
    /// let context = StandaloneBlockContext::new();
    /// let mut collection = OrbitalBondCollection::new(
    ///     "collection-1".to_string(),
    ///     "Test Bonds".to_string(),
    ///     "TBND".to_string(),
    ///     500,
    ///     100,
    ///     &context
    /// );
    /// 
    /// // Deactivate the collection
    /// collection.deactivate();
    /// 
    /// // Verify collection is inactive
    /// assert!(!collection.is_active());
    /// ```
    pub fn deactivate(&mut self) {
        self.active = false;
    }
    
    /// Reactivates a previously deactivated collection
    ///
    /// # Example
    /// ```
    /// use slop::contracts::OrbitalBondCollection;
    /// use slop::utils::StandaloneBlockContext;
    /// 
    /// let context = StandaloneBlockContext::new();
    /// let mut collection = OrbitalBondCollection::new(
    ///     "collection-1".to_string(),
    ///     "Test Bonds".to_string(),
    ///     "TBND".to_string(),
    ///     500,
    ///     100,
    ///     &context
    /// );
    /// 
    /// // First deactivate the collection
    /// collection.deactivate();
    /// assert!(!collection.is_active());
    /// 
    /// // Later reactivate it
    /// collection.reactivate();
    /// assert!(collection.is_active());
    /// ```
    pub fn reactivate(&mut self) {
        self.active = true;
    }
    
    /// Cancels a specific bond and returns the principal amount
    ///
    /// # Parameters
    /// * `bond_id` - The ID of the bond to cancel
    ///
    /// # Returns
    /// * `Ok(u64)` - The principal amount returned upon cancellation
    /// * `Err(&'static str)` - Error message if cancellation failed
    ///
    /// # Errors
    /// * "Bond not found" - If no bond exists with the given ID
    /// * "Bond is not active" - If the bond has already been redeemed or canceled
    ///
    /// # Example
    /// ```
    /// use slop::contracts::OrbitalBondCollection;
    /// use slop::utils::StandaloneBlockContext;
    /// use slop::models::bond::BondStatus;
    /// 
    /// let context = StandaloneBlockContext::new();
    /// let mut collection = OrbitalBondCollection::new(
    ///     "collection-1".to_string(),
    ///     "Test Bonds".to_string(),
    ///     "TBND".to_string(),
    ///     500,
    ///     100,
    ///     &context
    /// );
    /// 
    /// // Mint a bond
    /// let (bond_id, _, _) = collection.mint_bond(
    ///     "orbital-xyz".to_string(),
    ///     1000,
    ///     "owner-123".to_string(),
    ///     &context
    /// ).unwrap();
    /// 
    /// // Cancel the bond and get the returned amount
    /// let returned_amount = collection.cancel_bond(&bond_id).unwrap();
    /// println!("Returned {} tokens to owner", returned_amount);
    /// 
    /// // Verify the bond is canceled
    /// let bond = collection.get_bond(&bond_id).unwrap();
    /// assert_eq!(bond.status, BondStatus::Canceled);
    /// ```
    pub fn cancel_bond(&mut self, bond_id: &str) -> Result<u64, &'static str> {
        let bond = self.bonds.get_mut(bond_id)
            .ok_or("Bond not found")?;
            
        bond.cancel()
    }
    
    /// Test-only method to update bonds using a provided callback function
    ///
    /// This method is only available in test environments and should not be used in production.
    /// It allows for direct manipulation of bond properties for testing scenarios.
    ///
    /// # Parameters
    /// * `update_fn` - A function that takes a mutable reference to a Bond and modifies it
    ///
    /// # Example
    /// ```
    /// // Make all bonds immediately mature for testing
    /// collection.update_bonds_for_test(|bond| {
    ///     bond.maturity_block = bond.creation_block;
    /// });
    /// ```
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
        // Basic test for collection creation
        let context = StandaloneBlockContext::new();
        let collection = OrbitalBondCollection::new(
            "test-collection".to_string(),
            "Test Collection".to_string(),
            "TEST".to_string(),
            500, // 5% interest
            100, // 100 blocks to mature
            &context
        );
        
        assert_eq!(collection.id, "test-collection");
        assert_eq!(collection.name, "Test Collection");
        assert_eq!(collection.symbol, "TEST");
        assert_eq!(collection.interest_rate_bps, 500);
        assert_eq!(collection.maturity_blocks, 100);
        assert!(collection.is_active());
    }
}
