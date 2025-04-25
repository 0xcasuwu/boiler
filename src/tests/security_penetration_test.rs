//! Security and Penetration Tests
//! 
//! This module contains tests specifically designed to test edge cases
//! and potential security vulnerabilities in the bond system.

use crate::contracts::{LaunchpadFactory, OrbitalBondCollection};
use crate::utils::{BlockContext, StandaloneBlockContext};
use crate::models::bond;
use std::collections::HashSet;

// Custom test context for controlled block height manipulation
struct AttackerBlockContext {
    block_height: u64,
}

impl BlockContext for AttackerBlockContext {
    fn get_current_block_height(&self) -> u64 {
        self.block_height
    }
}

// Helper to create an attacker context at a specific block height
fn attacker_context(block_height: u64) -> AttackerBlockContext {
    AttackerBlockContext { block_height }
}

#[test]
fn test_integer_overflow_in_interest_calculation() {
    // Setup the test with realistic block context
    let context = StandaloneBlockContext::new();
    
    // Create a new collection
    let mut collection = OrbitalBondCollection::new(
        "Overflow Test".to_string(),
        "OVFL".to_string(),
        "".to_string(),
        10000, // 100% interest rate
        50,    // 50 blocks to maturity
        &context,
    );
    
    // Try to mint a bond with maximum u64 value
    // This should either fail safely or handle the max value without overflowing
    let max_amount = u64::MAX;
    let result = collection.mint_bond("overflow-test".to_string(), max_amount, &context);
    
    // It should ideally fail with a specific error rather than panic
    // But if it succeeds, the interest calculation shouldn't overflow
    if let Ok(bond_id) = result {
        // Advance block to maturity
        let mature_context = attacker_context(context.get_current_block_height() + 100);
        
        // Attempt to redeem and check if it handled overflow
        let redemption_result = collection.redeem_bond(&bond_id, &mature_context);
        
        // If it allows redemption, it should return a valid value without overflowing
        if let Ok(value) = redemption_result {
            // The value should be capped or handled properly
            // It should NOT be less than the original amount (which would indicate overflow)
            assert!(value >= max_amount, "Possible overflow detected in interest calculation");
        }
    }
}

#[test]
fn test_double_redemption_attack() {
    // Setup
    let context = StandaloneBlockContext::new();
    let mut collection = OrbitalBondCollection::new(
        "Security Test".to_string(),
        "SEC".to_string(),
        "".to_string(),
        500, // 5% interest
        50,  // 50 blocks to maturity
        &context,
    );
    
    // Mint a bond
    let orbital_id = "double-redeem-target".to_string();
    let bond_id = collection.mint_bond(orbital_id.clone(), 1000, &context).unwrap();
    
    // Move forward to maturity
    let mature_context = attacker_context(context.get_current_block_height() + 100);
    
    // First redemption (legitimate)
    let first_result = collection.redeem_bond(&orbital_id, &mature_context);
    assert!(first_result.is_ok());
    
    // Second redemption attempt (attack)
    let second_result = collection.redeem_bond(&orbital_id, &mature_context);
    
    // Verify that the second redemption fails
    assert!(second_result.is_err());
    
    // Check that the error message indicates the bond is no longer active
    if let Err(error) = second_result {
        assert!(error.contains("Bond is not active") || error.contains("not active"), 
                "Double redemption should be prevented with appropriate error");
    }
}

#[test]
fn test_bond_frontrunning_attack() {
    // Setup with a simulated block context we control
    let context = AttackerBlockContext { block_height: 1000 };
    
    // Create a collection with favorable terms
    let mut collection = OrbitalBondCollection::new(
        "Frontrun Test".to_string(),
        "FRNT".to_string(),
        "".to_string(),
        800, // 8% interest rate (very favorable)
        10,  // Quick maturity
        &context,
    );
    
    // Simulate a situation where the attacker observes a pending transaction
    // that will deactivate the collection or change terms
    
    // Attacker quickly mints a bond before terms change
    let orbital_id = "frontrun-attack".to_string();
    let bond_id = collection.mint_bond(orbital_id.clone(), 5000, &context).unwrap();
    
    // Collection is deactivated (the transaction the attacker frontran)
    collection.deactivate();
    
    // Verify the bond still exists despite deactivation
    let bond = collection.get_bond(&bond_id);
    assert!(bond.is_some());
    
    // Move forward to maturity
    let mature_context = AttackerBlockContext { block_height: 1020 };
    
    // Attacker attempts to redeem
    let result = collection.redeem_bond(&orbital_id, &mature_context);
    
    // The system should still allow redemption of valid bonds
    // from before deactivation (this is correct behavior)
    assert!(result.is_ok());
    let redemption_amount = result.unwrap();
    
    // Verify correct amount with interest
    let expected = 5000 + (5000 * 800 / 10000); // Principal + interest
    assert_eq!(redemption_amount, expected);
}

#[test]
fn test_replay_attack() {
    let context = StandaloneBlockContext::new();
    let mut collection = OrbitalBondCollection::new(
        "Replay Test".to_string(),
        "RPLY".to_string(),
        "".to_string(),
        500,
        50,
        &context,
    );
    
    // Mint an initial bond
    let orbital_id = "replay-target".to_string();
    let bond_id_1 = collection.mint_bond(orbital_id.clone(), 1000, &context).unwrap();
    
    // This implementation should reject duplicate orbital IDs
    let bond_id_2_result = collection.mint_bond(orbital_id.clone(), 1000, &context);
    assert!(bond_id_2_result.is_err(), "Should reject duplicate orbital ID");
    
    // Let's use a different orbital ID to verify bonds work properly
    let orbital_id_2 = "replay-target-2".to_string();
    let bond_id_2 = collection.mint_bond(orbital_id_2.clone(), 1000, &context).unwrap();
    
    // Verify the two bonds are different
    assert_ne!(bond_id_1, bond_id_2, "Created duplicate bonds with same orbital ID");
    
    // Move to maturity
    let mature_context = attacker_context(context.get_current_block_height() + 100);
    
    // Redeem the first bond
    let redeem_1 = collection.redeem_bond(&orbital_id, &mature_context).unwrap();
    
    // Redeem the second bond
    let redeem_2 = collection.redeem_bond(&orbital_id_2, &mature_context).unwrap();
    
    // Both redemptions should have succeeded with proper amounts
    assert_eq!(redeem_1, 1050);
    assert_eq!(redeem_2, 1050);
}

#[test]
fn test_time_manipulation_attack() {
    // Create a collection with a malicious time context
    // that would allow premature redemption
    let context = AttackerBlockContext { block_height: 1000 };
    
    let mut collection = OrbitalBondCollection::new(
        "Time Attack Test".to_string(),
        "TIME".to_string(),
        "".to_string(),
        500,
        100, // 100 blocks to maturity
        &context,
    );
    
    // Mint a bond
    let orbital_id = "time-attack".to_string();
    let bond_id = collection.mint_bond(orbital_id.clone(), 1000, &context).unwrap();
    
    // Attacker tries to manipulate time context to redeem early
    
    // First attempt: slightly before maturity
    let almost_mature_context = AttackerBlockContext { block_height: 1099 };
    let early_result = collection.redeem_bond(&orbital_id, &almost_mature_context);
    assert!(early_result.is_err(), "Should prevent early redemption");
    
    // Second attempt: exactly at maturity
    let mature_context = AttackerBlockContext { block_height: 1100 };
    let mature_result = collection.redeem_bond(&orbital_id, &mature_context);
    assert!(mature_result.is_ok(), "Should allow redemption at maturity");
    
    // Verify correct amount with interest
    let redemption_amount = mature_result.unwrap();
    assert_eq!(redemption_amount, 1050);
}

#[test]
fn test_factory_collection_manipulation() {
    let context = StandaloneBlockContext::new();
    let mut factory = LaunchpadFactory::new(
        "1.0.0".to_string(),
        100,
        500,
        &context,
    );
    
    // Create a legitimate collection
    let collection_id = factory.create_collection(
        "Legitimate Collection".to_string(),
        "LEGIT".to_string(),
        None, 
        Some(500),
        Some(50),
        &context,
    );
    
    // Try to operate on a non-existent collection
    let fake_id = "fake-collection-id";
    let mint_result = factory.mint_bond(
        fake_id,
        "orbital-id".to_string(),
        1000,
        &context,
    );
    
    // Should fail with a clear error, not panic
    assert!(mint_result.is_err());
    
    // Try to activate/deactivate non-existent collection
    let deactivate_result = factory.deactivate_collection(fake_id);
    assert!(deactivate_result.is_err());
    
    // Try to redeem from non-existent collection
    let redeem_result = factory.redeem_bond(
        fake_id,
        "orbital-id",
        &context,
    );
    assert!(redeem_result.is_err());
}

#[test]
fn test_invalid_orbital_id_attack() {
    let context = StandaloneBlockContext::new();
    let mut collection = OrbitalBondCollection::new(
        "ID Test".to_string(),
        "IDTST".to_string(),
        "".to_string(),
        500,
        50,
        &context,
    );
    
    // Try various potentially problematic IDs
    let long_string = "a".repeat(1000);
    let test_ids = vec![
        "",                           // Empty string
        " ",                          // Space
        &long_string,                 // Very long string
        "DROP TABLE bonds;--",        // SQL injection style
        "<script>alert('xss');</script>", // XSS style
        "{\"malformed\":\"json\"",    // Broken JSON
        "null",                       // Special value
        "undefined",                  // Special value
    ];
    
    for id in test_ids {
        let result = collection.mint_bond(id.to_string(), 1000, &context);
        
        // It should either handle these safely or reject with a clear error
        if result.is_ok() {
            let bond_id = result.unwrap();
            
            // If it accepted the ID, verify the bond was created properly
            let bond = collection.get_bond(&bond_id);
            assert!(bond.is_some(), "Bond wasn't properly saved despite successful mint");
            
            // Move to maturity
            let mature_context = attacker_context(context.get_current_block_height() + 100);
            
            // Check that redemption works too
            let redemption = collection.redeem_bond(&id.to_string(), &mature_context);
            assert!(redemption.is_ok(), "Redemption should work for bond with unusual ID");
        }
        // No assertion for error case - valid to reject problematic IDs
    }
}

#[test]
fn test_zero_value_attack() {
    let context = StandaloneBlockContext::new();
    let mut collection = OrbitalBondCollection::new(
        "Zero Test".to_string(),
        "ZERO".to_string(),
        "".to_string(),
        500,
        50,
        &context,
    );
    
    // Try to mint a bond with zero value
    let orbital_id = "zero-value".to_string();
    let result = collection.mint_bond(orbital_id.clone(), 0, &context);
    
    // Should either be rejected or handled safely
    if result.is_ok() {
        let _bond_id = result.unwrap();
        
        // Move to maturity
        let mature_context = attacker_context(context.get_current_block_height() + 100);
        
            // Try to redeem
            let redemption = collection.redeem_bond(&orbital_id, &mature_context);
        
        // If zero-value bonds are allowed, redemption should work correctly
        assert!(redemption.is_ok());
        
        // The redeemed value should be zero plus any applicable interest
        // (which would also be zero in this case)
        let redeemed_amount = redemption.unwrap();
        assert_eq!(redeemed_amount, 0);
    } else {
        // If zero-value bonds are prohibited, it should fail with clear error
        let error = result.unwrap_err();
        assert!(error.contains("zero") || 
                error.contains("invalid") || 
                error.contains("amount"),
                "Should reject zero amount with clear error");
    }
}

#[test]
fn test_multiple_redemption_paths() {
    let context = StandaloneBlockContext::new();
    let mut collection = OrbitalBondCollection::new(
        "Multi Test".to_string(),
        "MULT".to_string(),
        "".to_string(),
        500,
        50,
        &context,
    );
    
    // Create a bond
    let orbital_id = "multi-path-target".to_string();
    let bond_id = collection.mint_bond(orbital_id.clone(), 1000, &context).unwrap();
    
    // Move to maturity
    let mature_context = attacker_context(context.get_current_block_height() + 100);
    
    // Try to redeem using orbital_id
    let direct_result = collection.redeem_bond(&orbital_id, &mature_context);
    assert!(direct_result.is_ok());
    
    // The bond should be marked as redeemed now
    
    // Try to extract the bond again using orbital_id
    let bond = collection.get_bond_by_orbital(&orbital_id);
    
    // It should either return None or a redeemed bond
    if let Some(found_bond) = bond {
        // If it returns a bond, status should be Redeemed
        assert!(found_bond.status != bond::BondStatus::Active);
    }
}

#[test]
fn test_factory_bypass_attack() {
    let context = StandaloneBlockContext::new();
    let mut factory = LaunchpadFactory::new(
        "1.0.0".to_string(),
        100,
        500,
        &context,
    );
    
    // Create a collection through the factory
    let collection_id = factory.create_collection(
        "Factory Test".to_string(),
        "FACT".to_string(),
        None,
        Some(500),
        Some(50),
        &context,
    );
    
    // Try to create a bond in the collection
    let orbital_id = "factory-test".to_string();
    let mint_result = factory.mint_bond(
        &collection_id,
        orbital_id.clone(),
        1000,
        &context,
    );
    assert!(mint_result.is_ok());
    
    // Now try to get the collection directly and modify it
    // (simulating a bypass of factory's access controls)
    if let Some(mut collection) = factory.get_collection_mut(&collection_id) {
        // Collection is accessible through factory - fine as long as
        // critical operations are properly protected
        
        // Try to deactivate the collection
        collection.deactivate();
        
        // Check that the collection is indeed deactivated
        assert!(!collection.is_active());
        
        // Try to mint another bond after deactivation
        // This should fail because collection is deactivated
        let another_mint = collection.mint_bond("bypass-attempt".to_string(), 1000, &context);
        assert!(another_mint.is_err(), "Should prevent minting on deactivated collection");
    }
    
    // Verify that the factory still knows the collection is deactivated
    assert_eq!(factory.active_collections_count(), 0);
}

#[test]
fn test_state_consistency_attack() {
    let context = StandaloneBlockContext::new();
    let mut factory = LaunchpadFactory::new(
        "1.0.0".to_string(),
        100,
        500,
        &context,
    );
    
    // Create multiple collections and bonds to test consistency
    let collection_ids = vec![
        factory.create_collection("C1".to_string(), "C1".to_string(), None, None, None, &context),
        factory.create_collection("C2".to_string(), "C2".to_string(), None, None, None, &context),
        factory.create_collection("C3".to_string(), "C3".to_string(), None, None, None, &context),
    ];
    
    // Mint bonds in each collection
    let mut orbital_ids = Vec::new();
    
    for collection_id in &collection_ids {
        for i in 1..=5 {
            let orbital_id = format!("orbit-{}-{}", collection_id, i);
            
            // Create bond
            factory.mint_bond(
                collection_id,
                orbital_id.clone(),
                i * 1000,
                &context
            ).unwrap();
            
            orbital_ids.push((collection_id.clone(), orbital_id));
        }
    }
    
    // Deactivate a collection but leave bonds active
    factory.deactivate_collection(&collection_ids[1]).unwrap();
    
    // Move to maturity
    let mature_context = attacker_context(context.get_current_block_height() + 200);
    
    // Try to redeem bonds - all should work regardless of collection state
    for (collection_id, orbital_id) in orbital_ids {
        let result = factory.redeem_bond(&collection_id, &orbital_id, &mature_context);
        assert!(result.is_ok(), 
                "Failed to redeem bond after collection state manipulation");
    }
    
    // Verify all bonds are redeemed
    assert_eq!(factory.total_value(&mature_context), 0);
}
