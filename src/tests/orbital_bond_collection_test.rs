//! Tests for the OrbitalBondCollection module
//! These tests verify all functionality of the orbital bond collection

use crate::contracts::OrbitalBondCollection;
use crate::models::bond::BondStatus;
use crate::utils::{BlockContext, StandaloneBlockContext};
use std::error::Error as StdError;

#[test]
fn test_collection_creation_with_defaults() {
    let context = StandaloneBlockContext::new();
    let collection = OrbitalBondCollection::new(
        "collection-1".to_string(),
        "Test Bonds".to_string(),
        "TBND".to_string(),
        500, // 5% interest
        100, // 100 blocks to mature
        &context
    );

    assert_eq!(collection.id, "collection-1");
    assert_eq!(collection.name, "Test Bonds");
    assert_eq!(collection.symbol, "TBND");
    assert_eq!(collection.interest_rate_bps, 500);
    assert_eq!(collection.maturity_blocks, 100);
    assert!(collection.is_active());
    assert_eq!(collection.total_bonds(), 0);
    assert_eq!(collection.description, None);
}

#[test]
fn test_collection_with_description() {
    let context = StandaloneBlockContext::new();
    let collection = OrbitalBondCollection::new(
        "collection-1".to_string(),
        "Test Bonds".to_string(),
        "TBND".to_string(),
        500,
        100,
        &context
    ).with_description("Test bond collection".to_string());

    assert_eq!(collection.description, Some("Test bond collection".to_string()));
}

#[test]
fn test_bond_minting_and_validation() {
    let context = StandaloneBlockContext::new();
    let mut collection = OrbitalBondCollection::new(
        "collection-1".to_string(),
        "Test Bonds".to_string(),
        "TBND".to_string(),
        500,
        100,
        &context
    );

    // Test minting a valid bond
    let result = collection.mint_bond(
        "orbital-1".to_string(),
        1000,
        &context
    );
    assert!(result.is_ok());

    // Test duplicate orbital ID validation
    let duplicate = collection.mint_bond(
        "orbital-1".to_string(),
        2000,
        &context
    );
    assert!(duplicate.is_err());
    assert_eq!(duplicate.unwrap_err(), "Orbital token already has a bond");

    // Test deactivated collection validation
    collection.deactivate();
    let inactive_result = collection.mint_bond(
        "orbital-2".to_string(),
        3000,
        &context
    );
    assert!(inactive_result.is_err());
    assert_eq!(inactive_result.unwrap_err(), "Collection is inactive");
}

#[test]
fn test_early_redemption_prevention() {
    let context = StandaloneBlockContext::new();
    let mut collection = OrbitalBondCollection::new(
        "collection-1".to_string(),
        "Test Bonds".to_string(),
        "TBND".to_string(),
        500, // 5% interest
        100, // 100 blocks to mature
        &context
    );

    // Mint a bond
    let orbital_id = "orbital-1".to_string();
    collection.mint_bond(
        orbital_id.clone(),
        1000,
        &context
    ).unwrap();

    // Try to redeem before maturity (same block height)
    let early_result = collection.redeem_bond(&orbital_id, &context);
    assert!(early_result.is_err());
    assert_eq!(early_result.unwrap_err(), "Bond has not reached maturity");
}

#[test]
fn test_successful_redemption_with_interest() {
    // Create a struct that allows us to control the block height
    struct TestBlockContext {
        block_height: u64
    }

    impl BlockContext for TestBlockContext {
        fn get_current_block_height(&self) -> u64 {
            self.block_height
        }
    }

    // Create initial context at block 1000
    let initial_context = TestBlockContext { block_height: 1000 };

    let mut collection = OrbitalBondCollection::new(
        "collection-1".to_string(),
        "Test Bonds".to_string(),
        "TBND".to_string(),
        500, // 5% interest
        100, // 100 blocks to mature
        &initial_context
    );

    // Mint a bond at block 1000
    let orbital_id = "orbital-1".to_string();
    collection.mint_bond(
        orbital_id.clone(),
        1000,
        &initial_context
    ).unwrap();

    // Create matured context at block 1101 (past maturity)
    let mature_context = TestBlockContext { block_height: 1101 };

    // Verify bond is mature
    let mature_bonds = collection.get_mature_bonds(&mature_context);
    assert_eq!(mature_bonds.len(), 1);

    // Redeem the bond
    let redemption_result = collection.redeem_bond(&orbital_id, &mature_context);
    assert!(redemption_result.is_ok());

    // Verify redemption amount is principal + interest
    let amount = redemption_result.unwrap();
    assert_eq!(amount, 1050); // 1000 + 5%

    // Verify bond status is now Redeemed
    let bond = collection.get_bond_by_orbital(&orbital_id).unwrap();
    assert_eq!(bond.status, BondStatus::Redeemed);

    // Verify that the bond can't be redeemed again
    let second_redemption = collection.redeem_bond(&orbital_id, &mature_context);
    assert!(second_redemption.is_err());
    assert_eq!(second_redemption.unwrap_err(), "Bond is not active");
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
        &context
    );

    // Mint a bond
    let orbital_id = "orbital-1".to_string();
    let bond_id = collection.mint_bond(
        orbital_id.clone(),
        1000,
        &context
    ).unwrap();

    // Cancel the bond
    let cancel_result = collection.cancel_bond(&bond_id);
    assert!(cancel_result.is_ok());
    assert_eq!(cancel_result.unwrap(), 1000); // Return principal without interest

    // Verify bond status
    let bond = collection.get_bond(&bond_id).unwrap();
    assert_eq!(bond.status, BondStatus::Canceled);

    // Verify that the bond can't be redeemed after cancellation
    let redemption_after_cancel = collection.redeem_bond(&orbital_id, &context);
    assert!(redemption_after_cancel.is_err());
}

#[test]
fn test_collection_queries_and_filtering() {
    let context = StandaloneBlockContext::new();
    let mut collection = OrbitalBondCollection::new(
        "collection-1".to_string(),
        "Test Bonds".to_string(),
        "TBND".to_string(),
        500,
        100,
        &context
    );

    // Mint multiple bonds
    let bond1 = collection.mint_bond("orbital-1".to_string(), 1000, &context).unwrap();
    let bond2 = collection.mint_bond("orbital-2".to_string(), 2000, &context).unwrap();
    let bond3 = collection.mint_bond("orbital-3".to_string(), 3000, &context).unwrap();

    // Cancel one bond
    collection.cancel_bond(&bond2).unwrap();

    // Test get_all_bonds
    let all_bonds = collection.get_all_bonds();
    assert_eq!(all_bonds.len(), 3);

    // Test get_active_bonds
    let active_bonds = collection.get_active_bonds();
    assert_eq!(active_bonds.len(), 2);

    // Test total_value
    let total_value = collection.total_value(&context);
    assert_eq!(total_value, 4000); // 1000 + 3000 (only active bonds)
}

#[test]
fn test_update_bonds_for_test() {
    let context = StandaloneBlockContext::new();
    let mut collection = OrbitalBondCollection::new(
        "collection-1".to_string(),
        "Test Bonds".to_string(),
        "TBND".to_string(),
        500,
        100,
        &context
    );

    // Mint bonds
    collection.mint_bond("orbital-1".to_string(), 1000, &context).unwrap();
    collection.mint_bond("orbital-2".to_string(), 2000, &context).unwrap();

    // Use the update_bonds_for_test method to modify bond properties
    let original_creation_block = context.get_current_block_height();
    collection.update_bonds_for_test(|bond| {
        // Set maturity to be immediate (same as creation block)
        bond.maturity_block = original_creation_block;
    });

    // Verify changes were applied to all bonds
    for bond in collection.get_all_bonds() {
        assert_eq!(bond.maturity_block, original_creation_block);
    }

    // Verify bonds can now be redeemed with the same context
    let redemption = collection.redeem_bond("orbital-1", &context);
    assert!(redemption.is_ok());
}

#[test]
fn test_nonexistent_orbital_and_bond() {
    let context = StandaloneBlockContext::new();
    let mut collection = OrbitalBondCollection::new(
        "collection-1".to_string(),
        "Test Bonds".to_string(),
        "TBND".to_string(),
        500,
        100,
        &context
    );

    // Test get_bond with nonexistent bond ID
    let nonexistent_bond = collection.get_bond("nonexistent-bond");
    assert!(nonexistent_bond.is_none());

    // Test get_bond_by_orbital with nonexistent orbital ID
    let nonexistent_orbital = collection.get_bond_by_orbital("nonexistent-orbital");
    assert!(nonexistent_orbital.is_none());

    // Test redeeming a nonexistent bond
    let redemption = collection.redeem_bond("nonexistent-orbital", &context);
    assert!(redemption.is_err());
    assert_eq!(redemption.unwrap_err(), "Orbital token has no associated bond");

    // Test canceling a nonexistent bond
    let cancellation = collection.cancel_bond("nonexistent-bond");
    assert!(cancellation.is_err());
    assert_eq!(cancellation.unwrap_err(), "Bond not found");
}

#[test]
fn test_collection_activation_lifecycle() {
    let context = StandaloneBlockContext::new();
    let mut collection = OrbitalBondCollection::new(
        "collection-1".to_string(),
        "Test Bonds".to_string(),
        "TBND".to_string(),
        500,
        100,
        &context
    );

    // Initial state is active
    assert!(collection.is_active());

    // Deactivate
    collection.deactivate();
    assert!(!collection.is_active());

    // Can't mint bonds on inactive collection
    let mint_result = collection.mint_bond("orbital-1".to_string(), 1000, &context);
    assert!(mint_result.is_err());

    // Reactivate
    collection.reactivate();
    assert!(collection.is_active());

    // Can mint after reactivation
    let mint_after = collection.mint_bond("orbital-1".to_string(), 1000, &context);
    assert!(mint_after.is_ok());
}

#[test]
fn test_maturity_calculations() {
    // Setup - create a bond with 100 block maturity
    let creation_context = StandaloneBlockContext::with_seconds_per_block(1)
        .with_offset(1000);

    let mut collection = OrbitalBondCollection::new(
        "collection-1".to_string(),
        "Test Bonds".to_string(),
        "TBND".to_string(),
        500,
        100, // 100 blocks maturity
        &creation_context
    );

    // Create a bond
    collection.mint_bond("orbital-1".to_string(), 1000, &creation_context).unwrap();

    // Test various times relative to maturity

    // 1. Before maturity (99 blocks later)
    let before_maturity = StandaloneBlockContext::with_seconds_per_block(1)
        .with_offset(1099);
    let mature_bonds_before = collection.get_mature_bonds(&before_maturity);
    assert_eq!(mature_bonds_before.len(), 0);

    // 2. At exact maturity (100 blocks later)
    let at_maturity = StandaloneBlockContext::with_seconds_per_block(1)
        .with_offset(1100);
    let mature_bonds_at = collection.get_mature_bonds(&at_maturity);
    assert_eq!(mature_bonds_at.len(), 1);

    // 3. After maturity (101 blocks later)
    let after_maturity = StandaloneBlockContext::with_seconds_per_block(1)
        .with_offset(1101);
    let mature_bonds_after = collection.get_mature_bonds(&after_maturity);
    assert_eq!(mature_bonds_after.len(), 1);
}
