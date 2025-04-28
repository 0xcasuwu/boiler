use crate::contracts::{LaunchpadFactory, OrbitalBondCollection};
use crate::utils::{BlockContext, StandaloneBlockContext};
use crate::tests::mock::{MockBlockContext, MockTransactionContext};
use crate::models::orbital_provenance::{OrbitalCreationData, generate_orbital_fingerprint};
use std::collections::HashMap;

/// # Orbital Provenance Verification System Tests
///
/// This module tests the security properties of our orbital token provenance system,
/// which ensures that orbital tokens presented to the contract were actually created
/// by our factory and cannot be forged.

/// Creates a mock context with a specified block height
fn context_at_height(height: u64) -> MockBlockContext {
    MockBlockContext::new().with_block_height(height)
}

/// Creates a transaction context that appears to own the specified orbital token
fn create_tx_context(orbital_id: &str) -> MockTransactionContext {
    MockTransactionContext::new()
        .with_orbital_token(orbital_id)
        .with_transaction_id(&format!("tx-{}", orbital_id))
}

/// # Test: Orbital Token Provenance Verification
///
/// This test verifies that our orbital token provenance system correctly
/// detects forged orbital tokens and prevents their use in our system.
#[test]
fn test_orbital_provenance_verification() {
    // Setup a factory with provenance tracking
    let context = StandaloneBlockContext::new();
    let mut factory = LaunchpadFactory::new(
        "1.0.0".to_string(), // Factory version
        50,                  // Default maturity blocks
        500,                 // Default interest rate (5%)
        &context
    );
    
    // Create a collection
    let collection_id = factory.create_collection(
        "Secure Collection".to_string(),
        "SECR".to_string(),
        Some("Collection with provenance verification".to_string()),
        None, None,
        &context
    );
    
    // Create a legitimate bond/orbital
    let legitimate_orbital_id = "legitimate-orbital-123".to_string();
    let (bond_id, _, _) = factory.mint_bond(
        &collection_id,
        legitimate_orbital_id.clone(),
        10000,
        "legitimate-owner".to_string(),
        &context
    ).unwrap();
    
    // Get the collection to examine
    let collection = factory.get_collection(&collection_id).unwrap();
    
    // Make the bond mature for testing
    let mature_context = context_at_height(context.get_current_block_height() + 100);
    
    println!("=== Testing Orbital Provenance Security ===");
    
    // TEST 1: Verify legitimate orbital succeeds verification
    println!("Test 1: Legitimate orbital verification");
    let result = factory.verify_orbital_provenance(
        &legitimate_orbital_id,
        Some(&collection_id)
    );
    
    assert!(result.is_ok(), "Security failure: Legitimate orbital with proper provenance failed verification");
    println!("✓ Legitimate orbital passed provenance verification");
    
    // TEST 2: Non-existent orbital should fail verification
    println!("\nTest 2: Non-existent orbital verification");
    let fake_orbital_id = "non-existent-orbital-456".to_string();
    let fake_result = factory.verify_orbital_provenance(
        &fake_orbital_id,
        Some(&collection_id)
    );
    
    assert!(fake_result.is_err(), "Security vulnerability: Non-existent orbital passed provenance check");
    println!("✓ Non-existent orbital correctly failed verification: {:?}", fake_result.err());
    
    // TEST 3: Cross-collection orbital verification
    println!("\nTest 3: Cross-collection verification");
    
    // Create second collection
    let collection2_id = factory.create_collection(
        "Another Collection".to_string(),
        "OTHR".to_string(),
        None, None, None,
        &context
    );
    
    // Verify with wrong collection
    let cross_collection_result = factory.verify_orbital_provenance(
        &legitimate_orbital_id,  // Legitimate orbital
        Some(&collection2_id)    // But wrong collection
    );
    
    assert!(cross_collection_result.is_err(), 
           "Security vulnerability: Orbital passed verification with wrong collection");
    println!("✓ Cross-collection verification correctly failed: {:?}", cross_collection_result.err());
    
    // TEST 4: Redemption should use provenance verification
    println!("\nTest 4: Redemption with provenance verification");
    
    // Create a legitimate transaction context
    let tx_context = create_tx_context(&legitimate_orbital_id);
    
    // Attempt redemption with correct provenance
    let redemption_result = factory.redeem_bond_secure(
        &collection_id,
        &tx_context,
        "legitimate-owner",
        &mature_context
    );
    
    // This should succeed as all provenance checks pass
    assert!(redemption_result.is_ok(), 
           "Legitimate redemption with proper provenance failed: {:?}", 
           redemption_result.err());
    
    println!("✓ Legitimate redemption with proper provenance succeeded");
    
    // TEST 5: Create another collection and try to mint same orbital ID (should fail)
    println!("\nTest 5: Orbital ID uniqueness enforced");
    
    let reuse_result = factory.mint_bond(
        &collection2_id,
        legitimate_orbital_id.clone(), // Try to reuse same orbital ID
        5000,
        "malicious-actor".to_string(),
        &context
    );
    
    assert!(reuse_result.is_err(),
           "Security vulnerability: System allowed reuse of an existing orbital ID");
    println!("✓ Attempt to reuse orbital ID correctly failed: {:?}", reuse_result.err());
    
    println!("\n=== All Orbital Provenance Security Tests Passed ===");
}

/// # Test: Advanced Forgery Attempt Rejection
///
/// This test simulates more sophisticated forgery attempts against our
/// orbital token provenance verification system to ensure it can detect
/// and prevent various types of token forgery attacks.
#[test]
fn test_advanced_forgery_rejection() {
    println!("\n=== Testing Advanced Forgery Rejection ===");
    
    // Setup a factory with provenance tracking
    let context = StandaloneBlockContext::new();
    let mut factory = LaunchpadFactory::new(
        "1.0.0".to_string(), // Factory version
        50,                  // Default maturity blocks
        500,                 // Default interest rate (5%)
        &context
    );
    
    // Create a collection
    let collection_id = factory.create_collection(
        "Secure Collection".to_string(),
        "SECR".to_string(),
        Some("Collection with provenance verification".to_string()),
        None, None,
        &context
    );
    
    // Create a legitimate bond to use as reference
    let legit_orbital_id = "legit-orbital-789".to_string();
    let (_bond_id, _alkane_id, _) = factory.mint_bond(
        &collection_id,
        legit_orbital_id.clone(),
        10000,
        "legitimate-owner".to_string(),
        &context
    ).unwrap();
    
    // ATTACK 1: Test with completely forged orbital token
    println!("\nATTACK 1: Using a completely forged orbital token");
    let forged_orbital_id = "forged-orbital-123".to_string();
    let tx_context = create_tx_context(&forged_orbital_id);
    
    // Attempt redemption with the forged token
    let mature_context = context_at_height(context.get_current_block_height() + 100);
    let forged_result = factory.redeem_bond_secure(
        &collection_id,
        &tx_context,
        "attacker",
        &mature_context
    );
    
    assert!(forged_result.is_err(), 
           "Security vulnerability: System accepted a completely forged orbital token");
    println!("✓ Forged token correctly rejected: {}", forged_result.unwrap_err());
    
    // ATTACK 2: Test with an external collection (not created through factory)
    println!("\nATTACK 2: Using a collection created outside the factory");
    
    // Create a rogue collection directly (bypassing factory)
    let mut rogue_collection = OrbitalBondCollection::new(
        "rogue-collection".to_string(),
        "Rogue Collection".to_string(),
        "ROGUE".to_string(),
        500,
        50,
        &context
    );
    
    // Mint a bond in the rogue collection
    let rogue_orbital_id = "rogue-orbital-456".to_string();
    rogue_collection.mint_bond(
        rogue_orbital_id.clone(),
        5000,
        "rogue-actor".to_string(),
        &context
    ).unwrap();
    
    // Try to verify this rogue orbital token with the factory
    let rogue_verify_result = factory.verify_orbital_provenance(
        &rogue_orbital_id,
        None
    );
    
    assert!(rogue_verify_result.is_err(), 
           "Security vulnerability: Factory accepted an orbital token from an external collection");
    println!("✓ External collection orbital token correctly rejected: {}", 
             rogue_verify_result.unwrap_err());
    
    // ATTACK 3: Test with manual fingerprint creation (mimicking factory)
    println!("\nATTACK 3: Manual fingerprint forgery attempt");
    
    // Try to forge a fingerprint for an orbital token
    let forged_fingerprint = generate_orbital_fingerprint(
        &factory.version,                // Use correct factory version 
        &collection_id,                  // Use legitimate collection
        "forged-orbital-fingerprint",    // Forged orbital ID
        context.get_current_block_height(),
        "attacker",
        "fake-nonce"                     // Different nonce than factory would use
    );
    
    // Create transaction context with this forged ID
    let forged_id = "forged-orbital-fingerprint".to_string();
    let forged_tx = create_tx_context(&forged_id);
    
    // Attempt to use this forged token for redemption
    let forged_fingerprint_result = factory.redeem_bond_secure(
        &collection_id,
        &forged_tx,
        "attacker",
        &mature_context
    );
    
    assert!(forged_fingerprint_result.is_err(),
           "Security vulnerability: Forged fingerprint was accepted");
    println!("✓ Manual fingerprint forgery correctly rejected: {}", 
             forged_fingerprint_result.unwrap_err());
    
    // ATTACK 4: Test a sophisticated cross-collection attack
    println!("\nATTACK 4: Sophisticated cross-collection attack");
    
    // Create a second legitimate collection
    let collection2_id = factory.create_collection(
        "Second Collection".to_string(),
        "SEC2".to_string(),
        None, None, None,
        &context
    );
    
    // Mint a legitimate bond in the second collection
    let orbital2_id = "legitimate-orbital-second".to_string();
    factory.mint_bond(
        &collection2_id,
        orbital2_id.clone(),
        10000,
        "owner-2".to_string(),
        &context
    ).unwrap();
    
    // Try to use this legitimate token from collection2 in collection1
    let cross_tx = create_tx_context(&orbital2_id);
    
    // Attempt cross-collection redemption
    let cross_result = factory.redeem_bond_secure(
        &collection_id,  // First collection
        &cross_tx,       // Transaction with second collection's token
        "attacker",
        &mature_context
    );
    
    assert!(cross_result.is_err(),
           "Security vulnerability: Cross-collection attack succeeded");
    println!("✓ Cross-collection attack correctly prevented: {}", 
             cross_result.unwrap_err());
    
    println!("\n=== All Advanced Forgery Rejection Tests Passed ===");
}
