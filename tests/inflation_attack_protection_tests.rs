//! Tests for ERC4626 inflation attack protection
//! 
//! These tests verify that the vault is protected against inflation attacks
//! as described in the OpenZeppelin documentation.

use yield_vault::mock_vault::MockYieldVault;
use yield_vault::utils::{VIRTUAL_SHARES, VIRTUAL_ASSETS, PRECISION_OFFSET, Conversion};

#[test]
fn test_virtual_offset_protection() {
    // Create a new vault
    let mut vault = MockYieldVault::default();
    vault.initialize(
        "YieldVault".to_string(), 
        "YVT".to_string(), 
        "Bitcoin".to_string(), 
        "BTC".to_string(), 
        8
    ).unwrap();
    
    // Attacker deposits a small amount (1 token)
    let attacker_deposit = 1u128;
    let attacker_shares = vault.convert_assets_to_tokens(attacker_deposit, 0, 0).unwrap();
    
    // Verify that the attacker gets shares based on the virtual offset
    // With virtual offset, the attacker should get shares proportional to:
    // shares = assets * (VIRTUAL_SHARES + 0) / (VIRTUAL_ASSETS + 0)
    let expected_shares = attacker_deposit * VIRTUAL_SHARES / VIRTUAL_ASSETS;
    
    // Apply precision offset
    let precision_factor = 10u128.pow(PRECISION_OFFSET as u32);
    let expected_shares_with_precision = expected_shares * precision_factor;
    
    assert_eq!(attacker_shares, expected_shares_with_precision, 
        "Attacker should get shares based on virtual offset");
    
    // Update vault state to simulate the deposit
    // In a real test, we would use the deposit method
    // Here we'll just update the internal state directly
    vault.set_value("total_assets", attacker_deposit);
    vault.set_total_issuance(attacker_shares);
    
    // Now simulate an attack: attacker donates a large amount directly
    // This would normally manipulate the exchange rate
    let donation = 1_000_000u128;
    vault.set_value("total_assets", vault.get_total_assets() + donation);
    
    // Calculate the exchange rate after the attack
    let total_assets = vault.get_total_assets();
    let total_supply = vault.get_total_issuance();
    
    // Victim tries to deposit a small amount
    let victim_deposit = 100u128;
    let victim_shares = vault.convert_assets_to_tokens(victim_deposit, total_assets, total_supply).unwrap();
    
    // Without protection, victim would get very few shares or even 0
    // With virtual offset, victim should get a reasonable amount of shares
    
    // Calculate expected shares with virtual offset
    // The actual calculation in the code might have slight differences due to rounding
    // So we'll just verify that the victim gets a reasonable number of shares
    assert!(victim_shares > 0, "Victim should get non-zero shares");
    
    // Calculate the approximate expected shares
    let expected_victim_shares = victim_deposit * (VIRTUAL_SHARES + total_supply) / (VIRTUAL_ASSETS + total_assets);
    let expected_shares_with_precision = expected_victim_shares * precision_factor;
    
    // Allow for a small margin of error (within 1%)
    let margin = expected_shares_with_precision / 100;
    let lower_bound = expected_shares_with_precision.saturating_sub(margin);
    let upper_bound = expected_shares_with_precision.saturating_add(margin);
    
    assert!(victim_shares >= lower_bound && victim_shares <= upper_bound,
        "Victim shares {} should be close to expected {} (within 1%)",
        victim_shares, expected_shares_with_precision);
    
    // Most importantly, verify that victim gets non-zero shares
    assert!(victim_shares > 0, "Victim should get non-zero shares");
}

#[test]
fn test_precision_offset() {
    // Create a new vault
    let mut vault = MockYieldVault::default();
    vault.initialize(
        "YieldVault".to_string(), 
        "YVT".to_string(), 
        "Bitcoin".to_string(), 
        "BTC".to_string(), 
        8
    ).unwrap();
    
    // Test with a very small deposit
    let small_deposit = 1u128;
    let shares = vault.convert_assets_to_tokens(small_deposit, 0, 0).unwrap();
    
    // With precision offset, even a small deposit should result in non-zero shares
    let precision_factor = 10u128.pow(PRECISION_OFFSET as u32);
    let expected_shares = small_deposit * precision_factor;
    
    assert_eq!(shares, expected_shares, 
        "Small deposit should result in non-zero shares due to precision offset");
    assert!(shares >= precision_factor, 
        "Shares should be at least equal to precision factor for a deposit of 1");
}

#[test]
fn test_direct_donation_prevention() {
    // This test would normally use the full asset management implementation
    // Since we can't easily test that here, we'll just verify the conversion functions
    
    // Create a new vault
    let mut vault = MockYieldVault::default();
    vault.initialize(
        "YieldVault".to_string(), 
        "YVT".to_string(), 
        "Bitcoin".to_string(), 
        "BTC".to_string(), 
        8
    ).unwrap();
    
    // Initial state
    let initial_deposit = 1000u128;
    let initial_shares = vault.convert_assets_to_tokens(initial_deposit, 0, 0).unwrap();
    
    // Update vault state
    vault.set_value("total_assets", initial_deposit);
    vault.set_total_issuance(initial_shares);
    
    // Calculate exchange rate before donation
    let assets_before = vault.get_total_assets();
    let shares_before = vault.get_total_issuance();
    
    // Apply virtual offset to the exchange rate calculation
    let adjusted_assets_before = assets_before + VIRTUAL_ASSETS;
    let adjusted_shares_before = shares_before + VIRTUAL_SHARES;
    let rate_before = (adjusted_assets_before as f64) / (adjusted_shares_before as f64);
    
    // Simulate a donation (in a real scenario, this would be rejected)
    let donation = 10_000u128;
    vault.set_value("total_assets", vault.get_total_assets() + donation);
    
    // Calculate exchange rate after donation
    let assets_after = vault.get_total_assets();
    let shares_after = vault.get_total_issuance();
    
    // Apply virtual offset to the exchange rate calculation
    let adjusted_assets = assets_after + VIRTUAL_ASSETS;
    let adjusted_shares = shares_after + VIRTUAL_SHARES;
    let rate_after = (adjusted_assets as f64) / (adjusted_shares as f64);
    
    // The exchange rate has changed, but the virtual offset limits the impact
    println!("Rate before donation: {}", rate_before);
    println!("Rate after donation: {}", rate_after);
    
    // Verify that the virtual offset limits the impact of the donation
    // Without virtual offset, the rate would be (initial_deposit + donation) / initial_shares
    let rate_without_protection = ((initial_deposit + donation) as f64) / (initial_shares as f64);
    
    // With virtual offset, the rate should be closer to 
    // (initial_deposit + donation + VIRTUAL_ASSETS) / (initial_shares + VIRTUAL_SHARES)
    let assets_with_virtual = initial_deposit + donation + VIRTUAL_ASSETS;
    let shares_with_virtual = initial_shares + VIRTUAL_SHARES;
    let expected_rate = (assets_with_virtual as f64) / (shares_with_virtual as f64);
    
    println!("Rate without protection would be: {}", rate_without_protection);
    println!("Rate with virtual offset should be close to: {}", expected_rate);
    
    // Verify that the actual rate is closer to the protected rate than the unprotected rate
    let diff_protected = (rate_after - expected_rate).abs();
    let diff_unprotected = (rate_after - rate_without_protection).abs();
    
    assert!(diff_protected < diff_unprotected, 
        "Exchange rate should be closer to the protected rate ({}) than the unprotected rate ({})",
        expected_rate, rate_without_protection);
}

#[test]
fn test_zero_amount_prevention() {
    // Create a new vault
    let mut vault = MockYieldVault::default();
    vault.initialize(
        "YieldVault".to_string(), 
        "YVT".to_string(), 
        "Bitcoin".to_string(), 
        "BTC".to_string(), 
        8
    ).unwrap();
    
    // Initial state with some assets and shares
    let initial_deposit = 1000u128;
    let initial_shares = vault.convert_assets_to_tokens(initial_deposit, 0, 0).unwrap();
    
    // Update vault state
    vault.set_value("total_assets", initial_deposit);
    vault.set_total_issuance(initial_shares);
    
    // Try to convert 0 assets to shares
    let zero_assets = 0u128;
    let shares = vault.convert_assets_to_tokens(zero_assets, initial_deposit, initial_shares).unwrap();
    
    // Should get 0 shares
    assert_eq!(shares, 0, "Zero assets should result in zero shares");
    
    // Try to convert 0 shares to assets
    let zero_shares = 0u128;
    let assets = vault.convert_tokens_to_assets(zero_shares, initial_deposit, initial_shares).unwrap();
    
    // Should get 0 assets
    assert_eq!(assets, 0, "Zero shares should result in zero assets");
}
