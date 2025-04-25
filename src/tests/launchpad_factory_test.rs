//! Tests for the LaunchpadFactory
//! These tests verify all functionality of the orbital bond launchpad factory

use crate::contracts::LaunchpadFactory;
use crate::utils::{BlockContext, StandaloneBlockContext};

// Custom test context for controlling block height
struct TestBlockContext {
    block_height: u64,
}

impl BlockContext for TestBlockContext {
    fn get_current_block_height(&self) -> u64 {
        self.block_height
    }
}

#[test]
fn test_factory_creation() {
    let context = StandaloneBlockContext::new();
    
    let factory = LaunchpadFactory::new(
        "1.0.0".to_string(),
        100, // default maturity
        500, // default interest rate (5%)
        &context,
    );
    
    assert_eq!(factory.version, "1.0.0");
    assert_eq!(factory.total_collections(), 0);
    assert_eq!(factory.active_collections_count(), 0);
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
        "Test Collection".to_string(),
        "TEST".to_string(),
        None,
        None,
        None,
        &context,
    );
    
    assert!(factory.collection_exists(&collection_id));
    assert_eq!(factory.total_collections(), 1);
    assert_eq!(factory.active_collections_count(), 1);
    
    // Check collection properties use defaults
    let collection = factory.get_collection(&collection_id).unwrap();
    assert_eq!(collection.name, "Test Collection");
    assert_eq!(collection.symbol, "TEST");
    assert_eq!(collection.interest_rate_bps, 500); // Default interest rate
    assert_eq!(collection.maturity_blocks, 100); // Default maturity
    assert!(collection.is_active());
    
    // Create a collection with custom parameters
    let custom_collection_id = factory.create_collection(
        "Custom Collection".to_string(),
        "CUST".to_string(),
        Some("A custom bond collection".to_string()),
        Some(800), // 8% interest
        Some(200), // 200 blocks maturity
        &context,
    );
    
    assert!(factory.collection_exists(&custom_collection_id));
    assert_eq!(factory.total_collections(), 2);
    
    // Check custom collection properties
    let custom_collection = factory.get_collection(&custom_collection_id).unwrap();
    assert_eq!(custom_collection.name, "Custom Collection");
    assert_eq!(custom_collection.symbol, "CUST");
    assert_eq!(custom_collection.description, Some("A custom bond collection".to_string()));
    assert_eq!(custom_collection.interest_rate_bps, 800); // Custom interest rate
    assert_eq!(custom_collection.maturity_blocks, 200); // Custom maturity
}

#[test]
fn test_bond_minting_through_factory() {
    let context = StandaloneBlockContext::new();
    let mut factory = LaunchpadFactory::new(
        "1.0.0".to_string(),
        100,
        500,
        &context,
    );
    
    // Create a collection
    let collection_id = factory.create_collection(
        "Test Collection".to_string(),
        "TEST".to_string(),
        None,
        None,
        None,
        &context,
    );
    
    // Mint a bond
    let orbital_id = "orbital-1".to_string();
    let bond_id = factory.mint_bond(
        &collection_id,
        orbital_id.clone(),
        1000,
        &context,
    );
    
    assert!(bond_id.is_ok());
    
    // Verify the bond exists in the collection
    let collection = factory.get_collection(&collection_id).unwrap();
    let bond = collection.get_bond_by_orbital(&orbital_id);
    assert!(bond.is_some());
    assert_eq!(bond.unwrap().amount, 1000);
    
    // Verify total value is correct
    assert_eq!(factory.total_value(&context), 1000);
    
    // Test minting with invalid collection ID
    let invalid_mint = factory.mint_bond(
        "nonexistent-collection",
        "orbital-x".to_string(),
        1000,
        &context,
    );
    assert!(invalid_mint.is_err());
    assert_eq!(invalid_mint.unwrap_err(), "Collection not found");
}

#[test]
fn test_bond_redemption_through_factory() {
    // Create a context where we can control the block height
    let initial_context = TestBlockContext { block_height: 1000 };
    
    let mut factory = LaunchpadFactory::new(
        "1.0.0".to_string(),
        50, // 50 blocks to mature
        500, // 5% interest
        &initial_context,
    );
    
    // Create a collection
    let collection_id = factory.create_collection(
        "Test Collection".to_string(),
        "TEST".to_string(),
        None,
        None,
        None,
        &initial_context,
    );
    
    // Mint a bond
    let orbital_id = "orbital-1".to_string();
    factory.mint_bond(
        &collection_id,
        orbital_id.clone(),
        1000,
        &initial_context,
    ).unwrap();
    
    // Try to redeem too early
    let early_redemption = factory.redeem_bond(
        &collection_id,
        &orbital_id,
        &initial_context,
    );
    assert!(early_redemption.is_err());
    assert_eq!(early_redemption.unwrap_err(), "Bond has not reached maturity");
    
    // Create a mature context (51 blocks later)
    let mature_context = TestBlockContext { block_height: 1051 };
    
    // Now redeem when mature
    let redemption_result = factory.redeem_bond(
        &collection_id,
        &orbital_id,
        &mature_context,
    );
    
    assert!(redemption_result.is_ok());
    assert_eq!(redemption_result.unwrap(), 1050); // Principal + 5% interest
    
    // Verify bond is redeemed
    let redeemed_attempt = factory.redeem_bond(
        &collection_id,
        &orbital_id,
        &mature_context,
    );
    assert!(redeemed_attempt.is_err());
    assert_eq!(redeemed_attempt.unwrap_err(), "Bond has already been redeemed");
    
    // Verify total value is reduced
    assert_eq!(factory.total_value(&mature_context), 0);
}

#[test]
fn test_collection_deactivation() {
    let context = StandaloneBlockContext::new();
    let mut factory = LaunchpadFactory::new(
        "1.0.0".to_string(), 
        100,
        500,
        &context,
    );
    
    // Create two collections
    let collection1_id = factory.create_collection(
        "Collection 1".to_string(),
        "COL1".to_string(),
        None,
        None,
        None,
        &context,
    );
    
    let collection2_id = factory.create_collection(
        "Collection 2".to_string(),
        "COL2".to_string(),
        None, 
        None,
        None,
        &context,
    );
    
    // Initial state - both collections active
    assert_eq!(factory.active_collections_count(), 2);
    
    // Deactivate a collection
    let deactivate_result = factory.deactivate_collection(&collection1_id);
    assert!(deactivate_result.is_ok());
    
    // Verify only one active collection remaining
    assert_eq!(factory.active_collections_count(), 1);
    assert!(!factory.get_collection(&collection1_id).unwrap().is_active());
    assert!(factory.get_collection(&collection2_id).unwrap().is_active());
    
    // Try to mint on deactivated collection
    let mint_result = factory.mint_bond(
        &collection1_id,
        "orbital-x".to_string(),
        1000,
        &context,
    );
    assert!(mint_result.is_err());
    
    // Reactivate the collection
    let reactivate_result = factory.reactivate_collection(&collection1_id);
    assert!(reactivate_result.is_ok());
    
    // Verify both collections active again
    assert_eq!(factory.active_collections_count(), 2);
    assert!(factory.get_collection(&collection1_id).unwrap().is_active());
    
    // Now can mint on reactivated collection
    let mint_after = factory.mint_bond(
        &collection1_id,
        "orbital-y".to_string(),
        1000,
        &context,
    );
    assert!(mint_after.is_ok());
}

#[test]
fn test_nonexistent_collection_operations() {
    let context = StandaloneBlockContext::new();
    let mut factory = LaunchpadFactory::new("1.0.0".to_string(), 100, 500, &context);
    
    // Try to get a nonexistent collection
    let result = factory.get_collection("nonexistent");
    assert!(result.is_none());
    
    // Try to get a mutable reference to nonexistent collection 
    let result_mut = factory.get_collection_mut("nonexistent");
    assert!(result_mut.is_none());
    
    // Try operations on nonexistent collections
    assert!(!factory.collection_exists("nonexistent"));
    
    let deactivate_result = factory.deactivate_collection("nonexistent");
    assert!(deactivate_result.is_err());
    assert_eq!(deactivate_result.unwrap_err(), "Collection not found");
    
    let reactivate_result = factory.reactivate_collection("nonexistent");
    assert!(reactivate_result.is_err());
    assert_eq!(reactivate_result.unwrap_err(), "Collection not found");
}

#[test]
fn test_factory_total_value_calculation() {
    let context = StandaloneBlockContext::new();
    let mut factory = LaunchpadFactory::new("1.0.0".to_string(), 100, 500, &context);
    
    // Create two collections
    let collection1 = factory.create_collection(
        "Collection 1".to_string(),
        "COL1".to_string(), 
        None,
        None, 
        None,
        &context,
    );
    
    let collection2 = factory.create_collection(
        "Collection 2".to_string(),
        "COL2".to_string(),
        None,
        None,
        None,
        &context,
    );
    
    // Initial value is 0
    assert_eq!(factory.total_value(&context), 0);
    
    // Mint bonds in both collections
    factory.mint_bond(&collection1, "orbital-1".to_string(), 1000, &context).unwrap();
    factory.mint_bond(&collection1, "orbital-2".to_string(), 2000, &context).unwrap();
    factory.mint_bond(&collection2, "orbital-3".to_string(), 3000, &context).unwrap();
    
    // Total value should be sum of all active bonds
    assert_eq!(factory.total_value(&context), 6000);
    
    // Deactivate a collection - should still count in total value
    factory.deactivate_collection(&collection1).unwrap();
    assert_eq!(factory.total_value(&context), 6000); // Value unchanged
    
    // Cancel a bond
    let bond_id = factory.get_collection(&collection2).unwrap()
        .get_bond_by_orbital("orbital-3").unwrap().id.clone();
    
    factory.get_collection_mut(&collection2).unwrap()
        .cancel_bond(&bond_id).unwrap();
    
    // Total value should be reduced by canceled bond
    assert_eq!(factory.total_value(&context), 3000);
}

#[test]
fn test_batch_operations() {
    let initial_context = TestBlockContext { block_height: 1000 };
    let mut factory = LaunchpadFactory::new("1.0.0".to_string(), 50, 500, &initial_context);
    
    // Create collection and mint multiple bonds
    let collection_id = factory.create_collection(
        "Batch Test".to_string(),
        "BATCH".to_string(),
        None,
        None,
        None,
        &initial_context,
    );
    
    // Mint several bonds
    for i in 1..=5 {
        let orbital_id = format!("orbital-{}", i);
        let amount = i * 1000;
        factory.mint_bond(&collection_id, orbital_id, amount, &initial_context).unwrap();
    }
    
    // Move to maturity
    let mature_context = TestBlockContext { block_height: 1051 };
    
    // Get all mature bonds
    let collection = factory.get_collection(&collection_id).unwrap();
    let mature_bonds = collection.get_mature_bonds(&mature_context);
    assert_eq!(mature_bonds.len(), 5);
    
    // Try redeeming bonds sequentially
    for i in 1..=5 {
        let orbital_id = format!("orbital-{}", i);
        let expected_amount = i * 1000; // Principal
        let expected_interest = (expected_amount as f64 * 0.05) as u64; // 5% interest
        
        let redemption = factory.redeem_bond(&collection_id, &orbital_id, &mature_context);
        assert!(redemption.is_ok());
        assert_eq!(redemption.unwrap(), expected_amount + expected_interest);
    }
    
    // All bonds should now be redeemed
    assert_eq!(factory.total_value(&mature_context), 0);
}

#[test]
fn test_multiple_collections_with_same_orbital_id() {
    let context = StandaloneBlockContext::new();
    let mut factory = LaunchpadFactory::new("1.0.0".to_string(), 100, 500, &context);
    
    // Create two collections
    let collection1 = factory.create_collection(
        "Collection 1".to_string(),
        "COL1".to_string(),
        None,
        None,
        None,
        &context,
    );
    
    let collection2 = factory.create_collection(
        "Collection 2".to_string(), 
        "COL2".to_string(),
        None,
        None,
        None,
        &context,
    );
    
    // Same orbital ID can be used in different collections
    let orbital_id = "shared-orbital-id".to_string();
    
    // Mint in collection 1
    let bond1 = factory.mint_bond(&collection1, orbital_id.clone(), 1000, &context);
    assert!(bond1.is_ok());
    
    // Mint in collection 2 with same orbital ID
    let bond2 = factory.mint_bond(&collection2, orbital_id.clone(), 2000, &context);
    assert!(bond2.is_ok());
    
    // Verify both bonds exist with different amounts
    let col1_bond = factory.get_collection(&collection1).unwrap()
        .get_bond_by_orbital(&orbital_id).unwrap();
    
    let col2_bond = factory.get_collection(&collection2).unwrap()
        .get_bond_by_orbital(&orbital_id).unwrap();
    
    assert_eq!(col1_bond.amount, 1000);
    assert_eq!(col2_bond.amount, 2000);
}

#[test]
fn test_factory_next_collection_id_sequence() {
    let context = StandaloneBlockContext::new();
    let mut factory = LaunchpadFactory::new("1.0.0".to_string(), 100, 500, &context);
    
    // Create several collections in sequence
    let id1 = factory.create_collection(
        "Collection 1".to_string(), "COL1".to_string(), None, None, None, &context
    );
    
    let id2 = factory.create_collection(
        "Collection 2".to_string(), "COL2".to_string(), None, None, None, &context
    );
    
    let id3 = factory.create_collection(
        "Collection 3".to_string(), "COL3".to_string(), None, None, None, &context
    );
    
    // Verify IDs are generated sequentially
    assert_eq!(id1, "collection-1");
    assert_eq!(id2, "collection-2");
    assert_eq!(id3, "collection-3");
}
