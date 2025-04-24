use anyhow::Result;
use crate::contracts::LaunchpadFactory;

use super::mock::MockBlockContext;

#[test]
fn test_factory_initialization() -> Result<()> {
    // Create a mock block context
    let block_context = MockBlockContext::new().with_block_height(100);
    
    // Create a new LaunchpadFactory
    let factory = LaunchpadFactory::new(
        "1.0.0".to_string(),
        100, // Default maturity of 100 blocks
        500, // Default interest rate of 5.00% (500 basis points)
        &block_context,
    );
    
    // Verify the factory was created correctly
    assert_eq!(factory.version, "1.0.0");
    assert_eq!(factory.total_collections(), 0);
    
    // We don't test private fields anymore
    // assert_eq!(factory.default_maturity_blocks, 100);
    // assert_eq!(factory.default_interest_rate_bps, 500);
    
    Ok(())
}

#[test]
fn test_collection_creation() -> Result<()> {
    // Create a mock block context
    let block_context = MockBlockContext::new().with_block_height(100);
    
    // Create a new LaunchpadFactory
    let mut factory = LaunchpadFactory::new(
        "1.0.0".to_string(),
        100, // Default maturity of 100 blocks
        500, // Default interest rate of 5.00% (500 basis points)
        &block_context,
    );
    
    // Create a collection with default parameters
    let collection_id_1 = factory.create_collection(
        "Test Collection".to_string(),
        "TCOL".to_string(),
        None, // No description
        None, // Use default interest rate
        None, // Use default maturity
        &block_context,
    );
    
    // Verify the collection was created correctly
    let collection_1 = factory.get_collection(&collection_id_1).unwrap();
    assert_eq!(collection_1.name, "Test Collection");
    assert_eq!(collection_1.symbol, "TCOL");
    assert_eq!(collection_1.interest_rate_bps, 500); // Default
    assert_eq!(collection_1.maturity_blocks, 100); // Default
    assert!(collection_1.description.is_none());
    
    // Create a collection with custom parameters
    let collection_id_2 = factory.create_collection(
        "Custom Collection".to_string(),
        "CCOL".to_string(),
        Some("Custom description".to_string()),
        Some(800), // 8.00% interest rate
        Some(50),  // 50 blocks to maturity
        &block_context,
    );
    
    // Verify the collection was created correctly
    let collection_2 = factory.get_collection(&collection_id_2).unwrap();
    assert_eq!(collection_2.name, "Custom Collection");
    assert_eq!(collection_2.symbol, "CCOL");
    assert_eq!(collection_2.interest_rate_bps, 800); // Custom
    assert_eq!(collection_2.maturity_blocks, 50); // Custom
    assert_eq!(collection_2.description, Some("Custom description".to_string()));
    
    // Verify the total number of collections
    assert_eq!(factory.total_collections(), 2);
    
    Ok(())
}

#[test]
fn test_collection_retrieval() -> Result<()> {
    // Create a mock block context
    let block_context = MockBlockContext::new().with_block_height(100);
    
    // Create a new LaunchpadFactory
    let mut factory = LaunchpadFactory::new(
        "1.0.0".to_string(),
        100,
        500,
        &block_context,
    );
    
    // Create several collections
    let collection_id_1 = factory.create_collection(
        "Collection 1".to_string(),
        "COL1".to_string(),
        None,
        None,
        None,
        &block_context,
    );
    
    let collection_id_2 = factory.create_collection(
        "Collection 2".to_string(),
        "COL2".to_string(),
        None,
        None,
        None,
        &block_context,
    );
    
    let collection_id_3 = factory.create_collection(
        "Collection 3".to_string(),
        "COL3".to_string(),
        None,
        None,
        None,
        &block_context,
    );
    
    // Verify each collection can be retrieved
    assert!(factory.get_collection(&collection_id_1).is_some());
    assert!(factory.get_collection(&collection_id_2).is_some());
    assert!(factory.get_collection(&collection_id_3).is_some());
    
    // Verify a non-existent collection returns None
    assert!(factory.get_collection("non-existent-id").is_none());
    
    // Get all collections and verify their count
    let all_collections = factory.get_all_collections();
    assert_eq!(all_collections.len(), 3);
    
    // Verify an arbitrary collection property to ensure they're retrieved correctly
    assert_eq!(all_collections[0].symbol, "COL1");
    assert_eq!(all_collections[1].symbol, "COL2");
    assert_eq!(all_collections[2].symbol, "COL3");
    
    Ok(())
}

#[test]
fn test_bond_minting() -> Result<()> {
    // Create a mock block context
    let block_context = MockBlockContext::new().with_block_height(100);
    
    // Create a new LaunchpadFactory
    let mut factory = LaunchpadFactory::new(
        "1.0.0".to_string(),
        100,
        500,
        &block_context,
    );
    
    // Create a collection
    let collection_id = factory.create_collection(
        "Test Collection".to_string(),
        "TCOL".to_string(),
        None,
        None,
        None,
        &block_context,
    );
    
    // Mint bonds through the factory
    let orbital_id_1 = "orbital-123".to_string();
    let bond_id_1 = factory.mint_bond(
        &collection_id,
        orbital_id_1.clone(),
        1000, // 1000 units
        &block_context,
    )?;
    
    let orbital_id_2 = "orbital-456".to_string();
    let bond_id_2 = factory.mint_bond(
        &collection_id,
        orbital_id_2.clone(),
        2000, // 2000 units
        &block_context,
    )?;
    
    // Verify the bonds were created correctly
    let collection = factory.get_collection(&collection_id).unwrap();
    assert_eq!(collection.total_bonds(), 2);
    
    // Verify the bonds can be retrieved from the collection
    let bond_1 = collection.get_bond(&bond_id_1).unwrap();
    let bond_2 = collection.get_bond(&bond_id_2).unwrap();
    
    assert_eq!(bond_1.amount, 1000);
    assert_eq!(bond_2.amount, 2000);
    
    Ok(())
}

#[test]
fn test_bond_redemption() -> Result<()> {
    // Create a mock block context
    let mut block_context = MockBlockContext::new().with_block_height(100);
    
    // Create a new LaunchpadFactory
    let mut factory = LaunchpadFactory::new(
        "1.0.0".to_string(),
        100,
        500,
        &block_context,
    );
    
    // Create a collection
    let collection_id = factory.create_collection(
        "Test Collection".to_string(),
        "TCOL".to_string(),
        None,
        None,
        None,
        &block_context,
    );
    
    // Mint a bond
    let orbital_id = "orbital-123".to_string();
    factory.mint_bond(
        &collection_id,
        orbital_id.clone(),
        1000, // 1000 units
        &block_context,
    )?;
    
    // Try to redeem before maturity (should fail)
    let early_redemption = factory.redeem_bond(
        &collection_id,
        &orbital_id,
        &block_context,
    );
    assert!(early_redemption.is_err());
    
    // Advance block height to maturity
    block_context.advance_blocks(101); // Now at block 201 (past maturity)
    
    // Redeem the bond
    let redemption_amount = factory.redeem_bond(
        &collection_id,
        &orbital_id,
        &block_context,
    )?;
    
    // Verify the redemption amount (principal + 5% interest)
    assert_eq!(redemption_amount, 1050);
    
    Ok(())
}

#[test]
fn test_total_value() -> Result<()> {
    // Create a mock block context
    let block_context = MockBlockContext::new().with_block_height(100);
    
    // Create a new LaunchpadFactory
    let mut factory = LaunchpadFactory::new(
        "1.0.0".to_string(),
        100,
        500,
        &block_context,
    );
    
    // Create two collections
    let collection_id_1 = factory.create_collection(
        "Collection 1".to_string(),
        "COL1".to_string(),
        None,
        None,
        None,
        &block_context,
    );
    
    let collection_id_2 = factory.create_collection(
        "Collection 2".to_string(),
        "COL2".to_string(),
        None,
        None,
        None,
        &block_context,
    );
    
    // Mint bonds in both collections
    factory.mint_bond(&collection_id_1, "orbital-1".to_string(), 1000, &block_context)?;
    factory.mint_bond(&collection_id_1, "orbital-2".to_string(), 2000, &block_context)?;
    factory.mint_bond(&collection_id_2, "orbital-3".to_string(), 3000, &block_context)?;
    factory.mint_bond(&collection_id_2, "orbital-4".to_string(), 4000, &block_context)?;
    
    // Calculate total value across all collections
    let total_value = factory.total_value(&block_context);
    
    // Verify the total value (should be sum of all bond values)
    assert_eq!(total_value, 10000); // 1000 + 2000 + 3000 + 4000
    
    Ok(())
}

#[test]
fn test_collection_activation_status() -> Result<()> {
    // Create a mock block context
    let block_context = MockBlockContext::new().with_block_height(100);
    
    // Create a new LaunchpadFactory
    let mut factory = LaunchpadFactory::new(
        "1.0.0".to_string(),
        100,
        500,
        &block_context,
    );
    
    // Create a collection
    let collection_id = factory.create_collection(
        "Test Collection".to_string(),
        "TCOL".to_string(),
        None,
        None,
        None,
        &block_context,
    );
    
    // Get the number of active collections
    assert_eq!(factory.active_collections_count(), 1);
    
    // Deactivate the collection through the factory
    factory.deactivate_collection(&collection_id)?;
    
    // Verify the collection is inactive
    let collection = factory.get_collection(&collection_id).unwrap();
    assert!(!collection.is_active());
    
    // Verify the active collections count
    assert_eq!(factory.active_collections_count(), 0);
    
    // Reactivate the collection
    factory.reactivate_collection(&collection_id)?;
    
    // Verify the collection is active again
    let collection = factory.get_collection(&collection_id).unwrap();
    assert!(collection.is_active());
    
    // Verify the active collections count
    assert_eq!(factory.active_collections_count(), 1);
    
    Ok(())
}
