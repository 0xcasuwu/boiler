// YieldVault Integration Tests
// Tests for integrated functionality of the YieldVault contract

use crate::YieldVault;
use crate::tests::mock::*;
use alkanes_runtime::runtime::AlkaneResponder;
use std::collections::HashSet;
use serde_json;

// Helper function to set up a test environment with a vault
fn setup_test_vault() -> YieldVault {
    setup_test_environment();
    
    let mut vault = YieldVault::default();
    
    // Initialize the vault
    let init_result = vault.initialize(
        "Bitcoin Yield Vault".to_string(),
        "bYV".to_string(),
        "Bitcoin".to_string(),
        "BTC".to_string(),
        8
    );
    
    assert!(init_result.is_ok());
    
    vault
}

#[test]
fn test_full_lifecycle() {
    let mut vault = setup_test_vault();
    
    // 1. Initial deposits from multiple users
    vault.deposit(generate_tx_hash(), "alice".to_string(), "alice".to_string(), 1000).unwrap();
    vault.deposit(generate_tx_hash(), "bob".to_string(), "bob".to_string(), 2000).unwrap();
    
    // Verify initial state
    assert_eq!(storage::get_u128("/total-assets").unwrap(), 3000);
    assert_eq!(storage::get_u128("/total-supply").unwrap(), 3000);
    assert_eq!(storage::get_u128("/balances/alice").unwrap(), 1000);
    assert_eq!(storage::get_u128("/balances/bob").unwrap(), 2000);
    
    // 2. Set yield rate to 10% (1000 basis points)
    vault.update_yield_rate(1000).unwrap();
    
    // 3. Advance time by 1 year
    advance_time(365 * 24 * 60 * 60);
    
    // 4. Alice deposits more after yield accrual
    vault.deposit(generate_tx_hash(), "alice".to_string(), "alice".to_string(), 1000).unwrap();
    
    // Verify state after yield accrual
    // Total assets should be: 3000 + (3000 * 10%) + 1000 = 4300
    assert_eq!(storage::get_u128("/total-assets").unwrap(), 4300);
    // Original supply was 3000, new deposit should be ~1000 * (3000/3300) = ~909 shares
    let alice_shares = storage::get_u128("/balances/alice").unwrap();
    assert!(alice_shares > 1000 && alice_shares < 2000); // Should be around 1909
    
    // 5. Bob withdraws half his assets
    let bob_assets = vault.convert_to_assets(1000).unwrap().data;
    let bob_withdraw_amount = u128::from_le_bytes(bob_assets[0..16].try_into().unwrap());
    vault.withdraw(generate_tx_hash(), "bob".to_string(), "bob".to_string(), "bob".to_string(), bob_withdraw_amount).unwrap();
    
    // 6. Advance time another year
    advance_time(365 * 24 * 60 * 60);
    
    // 7. Carol joins with a new deposit
    vault.deposit(generate_tx_hash(), "carol".to_string(), "carol".to_string(), 2000).unwrap();
    
    // 8. Everyone withdraws their assets
    
    // Alice withdraws all
    let alice_balance = storage::get_u128("/balances/alice").unwrap();
    vault.redeem(generate_tx_hash(), "alice".to_string(), "alice".to_string(), "alice".to_string(), alice_balance).unwrap();
    assert_eq!(storage::get_u128("/balances/alice").unwrap(), 0);
    
    // Bob withdraws all
    let bob_balance = storage::get_u128("/balances/bob").unwrap();
    vault.redeem(generate_tx_hash(), "bob".to_string(), "bob".to_string(), "bob".to_string(), bob_balance).unwrap();
    assert_eq!(storage::get_u128("/balances/bob").unwrap(), 0);
    
    // Carol withdraws all
    let carol_balance = storage::get_u128("/balances/carol").unwrap();
    vault.redeem(generate_tx_hash(), "carol".to_string(), "carol".to_string(), "carol".to_string(), carol_balance).unwrap();
    assert_eq!(storage::get_u128("/balances/carol").unwrap(), 0);
    
    // Verify final state - vault should be empty
    assert_eq!(storage::get_u128("/total-assets").unwrap(), 0);
    assert_eq!(storage::get_u128("/total-supply").unwrap(), 0);
}

#[test]
fn test_security_properties() {
    let mut vault = setup_test_vault();
    
    // 1. Deposit funds
    vault.deposit(generate_tx_hash(), "alice".to_string(), "alice".to_string(), 1000).unwrap();
    
    // 2. Try transaction replay attack
    let tx_hash = "replay_attack_hash".to_string();
    vault.deposit(tx_hash.clone(), "alice".to_string(), "alice".to_string(), 500).unwrap();
    let replay_result = vault.deposit(tx_hash, "alice".to_string(), "alice".to_string(), 500);
    assert!(replay_result.is_err(), "Transaction replay should fail");
    
    // 3. Try unauthorized withdrawal
    let unauthorized_result = vault.withdraw(
        generate_tx_hash(),
        "attacker".to_string(),
        "attacker".to_string(),
        "alice".to_string(),
        100
    );
    assert!(unauthorized_result.is_err(), "Unauthorized withdrawal should fail");
    assert_eq!(storage::get_u128("/balances/alice").unwrap(), 1500, "Alice's balance should be unchanged");
    
    // 4. Try overflow attack with huge deposit
    let big_number = u128::MAX - 100; // Very large number close to max
    let overflow_result = vault.deposit(generate_tx_hash(), "attacker".to_string(), "attacker".to_string(), big_number);
    assert!(overflow_result.is_err(), "Overflow deposit should fail");
    
    // 5. Verify double-initialization protection
    let reinit_result = vault.initialize(
        "Hacked Vault".to_string(),
        "HACK".to_string(),
        "Hacked Asset".to_string(),
        "HACK".to_string(),
        18
    );
    assert!(reinit_result.is_err(), "Re-initialization should fail");
}

#[test]
fn test_yield_strategies() {
    let mut vault = setup_test_vault();
    
    // 1. Initial deposit
    vault.deposit(generate_tx_hash(), "alice".to_string(), "alice".to_string(), 10000).unwrap();
    
    // 2. Try different yield rates
    
    // First with 5% yield
    vault.update_yield_rate(500).unwrap();
    advance_time(365 * 24 * 60 * 60 / 2); // Half a year
    
    // Should have accrued about 2.5% yield
    vault.update_yield().unwrap();
    let assets_mid_year = storage::get_u128("/total-assets").unwrap();
    assert!(assets_mid_year >= 10250 && assets_mid_year <= 10251, 
           "Assets should be ~10250 but got {}", assets_mid_year);
    
    // Increase to 10% yield
    vault.update_yield_rate(1000).unwrap();
    advance_time(365 * 24 * 60 * 60 / 2); // Another half year
    
    // Should have accrued about 5% more yield
    vault.update_yield().unwrap();
    let assets_end_year = storage::get_u128("/total-assets").unwrap();
    assert!(assets_end_year >= 10762 && assets_end_year <= 10764, 
           "Assets should be ~10763 but got {}", assets_end_year);
    
    // Decrease to 0% yield
    vault.update_yield_rate(0).unwrap();
    advance_time(365 * 24 * 60 * 60); // Full year
    
    // Should have accrued no additional yield
    vault.update_yield().unwrap();
    let assets_no_yield = storage::get_u128("/total-assets").unwrap();
    assert_eq!(assets_no_yield, assets_end_year, 
              "Assets should not change with 0% yield rate");
}

#[test]
fn test_complex_scenario() {
    let mut vault = setup_test_vault();
    
    // Phase 1: Initial deposits and yield setting
    vault.deposit(generate_tx_hash(), "alice".to_string(), "alice".to_string(), 1000).unwrap();
    vault.deposit(generate_tx_hash(), "bob".to_string(), "bob".to_string(), 2000).unwrap();
    vault.update_yield_rate(1000).unwrap(); // 10% annual yield
    
    // Phase 2: Time passes with yield accrual
    advance_time(365 * 24 * 60 * 60 / 4); // 3 months
    
    // Alice adds more funds
    vault.deposit(generate_tx_hash(), "alice".to_string(), "alice".to_string(), 500).unwrap();
    
    // Phase 3: More time passes
    advance_time(365 * 24 * 60 * 60 / 4); // Another 3 months
    
    // Bob withdraws some funds
    let bob_shares_to_redeem = storage::get_u128("/balances/bob").unwrap() / 2; // Half of Bob's shares
    vault.redeem(generate_tx_hash(), "bob".to_string(), "bob".to_string(), "bob".to_string(), bob_shares_to_redeem).unwrap();
    
    // Phase 4: New user enters
    vault.deposit(generate_tx_hash(), "carol".to_string(), "carol".to_string(), 3000).unwrap();
    
    // Phase 5: Final time period passes
    advance_time(365 * 24 * 60 * 60 / 2); // 6 months
    
    // Update yield to reflect final state
    vault.update_yield().unwrap();
    
    // Verify balances reflect proper yield distribution
    // Each user should have proportionally accrued yield based on:
    // - Their deposit amount
    // - How long they've been in the vault
    // - The share price at time of deposit/withdrawal
    
    // Check final state - not asserting specific values since complex calculation
    // but verifying all values are reasonable and consistent
    let total_assets = storage::get_u128("/total-assets").unwrap();
    let total_supply = storage::get_u128("/total-supply").unwrap();
    let alice_shares = storage::get_u128("/balances/alice").unwrap();
    let bob_shares = storage::get_u128("/balances/bob").unwrap();
    let carol_shares = storage::get_u128("/balances/carol").unwrap();
    
    // All values should be non-zero
    assert!(total_assets > 0);
    assert!(total_supply > 0);
    assert!(alice_shares > 0);
    assert!(bob_shares > 0);
    assert!(carol_shares > 0);
    
    // Sum of all shares should equal total supply
    assert_eq!(alice_shares + bob_shares + carol_shares, total_supply);
    
    // Asset value should be greater than initial deposits due to yield
    assert!(total_assets > 1000 + 2000 + 500 + 3000);
}
