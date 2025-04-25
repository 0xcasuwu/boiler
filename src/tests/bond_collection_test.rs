use anyhow::Result;
use crate::contracts::OrbitalBondCollection;
use crate::models::BondStatus;

use super::mock::{MockBlockContext, MockTransactionContext, TransactionContextExt};
use std::error::Error as StdError;

#[test]
fn test_collection_creation() -> Result<()> {
    // Create a mock block context
    let block_context = MockBlockContext::new().with_block_height(100);

    // Create a new OrbitalBondCollection
    let collection = OrbitalBondCollection::new(
        "collection-1".to_string(),
        "Test Collection".to_string(),
        "TCOL".to_string(),
        500, // 5.00% interest rate in basis points
        100, // 100 blocks to maturity
        &block_context
    );

    // Verify the collection was created correctly
    assert_eq!(collection.id, "collection-1");
    assert_eq!(collection.name, "Test Collection");
    assert_eq!(collection.symbol, "TCOL");
    assert_eq!(collection.interest_rate_bps, 500);
    assert_eq!(collection.maturity_blocks, 100);
    assert_eq!(collection.creation_block, 100); // From our mock block context
    assert!(collection.is_active());

    // Test with optional description
    let collection_with_desc = OrbitalBondCollection::new(
        "collection-2".to_string(),
        "Test Collection 2".to_string(),
        "TC2".to_string(),
        500,
        100,
        &block_context
    ).with_description("Test description".to_string());

    assert_eq!(collection_with_desc.description, Some("Test description".to_string()));

    Ok(())
}

#[test]
fn test_bond_minting() -> Result<()> {
    // Create a mock block context
    let block_context = MockBlockContext::new().with_block_height(100);

    // Create a new OrbitalBondCollection
    let mut collection = OrbitalBondCollection::new(
        "collection-1".to_string(),
        "Test Collection".to_string(),
        "TCOL".to_string(),
        500, // 5.00% interest rate
        100, // 100 blocks to maturity
        &block_context
    );

    // Mock orbital token IDs
    let orbital_id_1 = "orbital-123".to_string();
    let orbital_id_2 = "orbital-456".to_string();

    // Mint a bond
    let bond_id_1 = collection.mint_bond(
        orbital_id_1.clone(),
        1000, // 1000 units
        &block_context
    ).map_err(|e| anyhow::anyhow!(e))?;

    // Verify the bond was created correctly
    let bond_1 = collection.get_bond(&bond_id_1).unwrap();
    assert_eq!(bond_1.orbital_token_id, orbital_id_1);
    assert_eq!(bond_1.amount, 1000);
    assert_eq!(bond_1.creation_block, 100);
    assert_eq!(bond_1.maturity_block, 200); // 100 blocks later
    assert_eq!(bond_1.status, BondStatus::Active);

    // Mint another bond
    let bond_id_2 = collection.mint_bond(
        orbital_id_2.clone(),
        2000, // 2000 units
        &block_context
    ).map_err(|e| anyhow::anyhow!(e))?;

    // Verify the second bond
    let bond_2 = collection.get_bond(&bond_id_2).unwrap();
    assert_eq!(bond_2.orbital_token_id, orbital_id_2);
    assert_eq!(bond_2.amount, 2000);

    // Verify total bonds count
    assert_eq!(collection.total_bonds(), 2);

    // Try to mint a bond with the same orbital ID (should fail)
    let duplicate_result = collection.mint_bond(
        orbital_id_1.clone(),
        3000,
        &block_context
    );

    assert!(duplicate_result.is_err());

    // Verify we can look up bonds by orbital ID
    let bond_by_orbital = collection.get_bond_by_orbital(&orbital_id_1).unwrap();
    assert_eq!(bond_by_orbital.id, bond_id_1);

    Ok(())
}

#[test]
fn test_bond_redemption() -> Result<()> {
    // Create a mock block context at block 100
    let mut block_context = MockBlockContext::new().with_block_height(100);

    // Create a new OrbitalBondCollection
    let mut collection = OrbitalBondCollection::new(
        "collection-1".to_string(),
        "Test Collection".to_string(),
        "TCOL".to_string(),
        500, // 5.00% interest rate
        100, // 100 blocks to maturity
        &block_context
    );

    // Mint a bond
    let orbital_id = "orbital-123".to_string();
    let bond_id = collection.mint_bond(
        orbital_id.clone(),
        1000, // 1000 units
        &block_context
    ).map_err(|e| anyhow::anyhow!(e))?;

    // Create a transaction context with the orbital ID
    let tx_context = MockTransactionContext::new()
        .with_orbital_token(&orbital_id)
        .with_transaction_id("tx-123");

    // Try to redeem before maturity (should fail)
    let result = collection.redeem_bond(&orbital_id, &block_context);
    assert!(result.is_err());

    // Advance block height to maturity
    block_context.advance_blocks(101); // Now at block 201 (past maturity)

    // Redeem the bond
    let redemption_amount = collection.redeem_bond(&orbital_id, &block_context).map_err(|e| anyhow::anyhow!(e))?;

    // Verify the redemption amount (principal + 5% interest)
    assert_eq!(redemption_amount, 1050);

    // Check bond status
    let bond = collection.get_bond(&bond_id).unwrap();
    assert_eq!(bond.status, BondStatus::Redeemed);

    // Try to redeem again (should fail)
    let result = collection.redeem_bond(&orbital_id, &block_context);
    assert!(result.is_err());

    Ok(())
}

#[test]
fn test_collection_deactivation() -> Result<()> {
    // Create a mock block context
    let block_context = MockBlockContext::new().with_block_height(100);

    // Create a new OrbitalBondCollection
    let mut collection = OrbitalBondCollection::new(
        "collection-1".to_string(),
        "Test Collection".to_string(),
        "TCOL".to_string(),
        500, // 5.00% interest rate
        100, // 100 blocks to maturity
        &block_context
    );

    // Verify collection is initially active
    assert!(collection.is_active());

    // Mint a bond while active
    let result = collection.mint_bond(
        "orbital-123".to_string(),
        1000,
        &block_context
    );
    assert!(result.is_ok());

    // Deactivate collection
    collection.deactivate();
    assert!(!collection.is_active());

    // Try to mint a bond while inactive (should fail)
    let result = collection.mint_bond(
        "orbital-456".to_string(),
        2000,
        &block_context
    );
    assert!(result.is_err());

    // Reactivate collection
    collection.reactivate();
    assert!(collection.is_active());

    // Mint a bond after reactivation
    let result = collection.mint_bond(
        "orbital-456".to_string(),
        2000,
        &block_context
    );
    assert!(result.is_ok());

    Ok(())
}

#[test]
fn test_mature_bonds_query() -> Result<()> {
    // Create a mock block context
    let mut block_context = MockBlockContext::new().with_block_height(100);

    // Create a new OrbitalBondCollection
    let mut collection = OrbitalBondCollection::new(
        "collection-1".to_string(),
        "Test Collection".to_string(),
        "TCOL".to_string(),
        500, // 5.00% interest rate
        100, // 100 blocks to maturity
        &block_context
    );

    // Mint several bonds at different times
    collection.mint_bond("orbital-1".to_string(), 1000, &block_context).map_err(|e| anyhow::anyhow!(e))?;

    block_context.advance_blocks(25);
    collection.mint_bond("orbital-2".to_string(), 2000, &block_context).map_err(|e| anyhow::anyhow!(e))?;

    block_context.advance_blocks(25);
    collection.mint_bond("orbital-3".to_string(), 3000, &block_context).map_err(|e| anyhow::anyhow!(e))?;

    // Check mature bonds before any are mature
    let mature_bonds = collection.get_mature_bonds(&block_context);
    assert_eq!(mature_bonds.len(), 0);

    // Advance time to where the first bond should be mature
    block_context.advance_blocks(50);

    let mature_bonds = collection.get_mature_bonds(&block_context);
    assert_eq!(mature_bonds.len(), 1);
    assert_eq!(mature_bonds[0].orbital_token_id, "orbital-1");

    // Advance more to where the second bond should also be mature
    block_context.advance_blocks(25);

    let mature_bonds = collection.get_mature_bonds(&block_context);
    assert_eq!(mature_bonds.len(), 2);

    // Advance to where all bonds are mature
    block_context.advance_blocks(25);

    let mature_bonds = collection.get_mature_bonds(&block_context);
    assert_eq!(mature_bonds.len(), 3);

    Ok(())
}

#[test]
fn test_total_value() -> Result<()> {
    // Create a mock block context
    let block_context = MockBlockContext::new().with_block_height(100);

    // Create a new OrbitalBondCollection
    let mut collection = OrbitalBondCollection::new(
        "collection-1".to_string(),
        "Test Collection".to_string(),
        "TCOL".to_string(),
        500, // 5.00% interest rate
        100, // 100 blocks to maturity
        &block_context
    );

    // Initialize with no bonds
    assert_eq!(collection.total_value(&block_context), 0);

    // Mint several bonds
    collection.mint_bond("orbital-1".to_string(), 1000, &block_context).map_err(|e| anyhow::anyhow!(e))?;
    collection.mint_bond("orbital-2".to_string(), 2000, &block_context).map_err(|e| anyhow::anyhow!(e))?;
    collection.mint_bond("orbital-3".to_string(), 3000, &block_context).map_err(|e| anyhow::anyhow!(e))?;

    // Total value should be sum of bond values
    assert_eq!(collection.total_value(&block_context), 6000);

    // Redeem one bond after maturity
    let mut advanced_context = MockBlockContext::new().with_block_height(201);
    collection.redeem_bond("orbital-1", &advanced_context).map_err(|e| anyhow::anyhow!(e))?;

    // Total value should now exclude the redeemed bond
    assert_eq!(collection.total_value(&advanced_context), 5000);

    Ok(())
}
