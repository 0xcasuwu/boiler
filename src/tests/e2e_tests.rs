use std::sync::Arc;

use crate::storage::Storage;
use crate::security::Security;
use crate::utils::Conversion;
use crate::asset_management::AssetManagement;
use alkanes_runtime::storage::StoragePointer;
use alkanes_support::context::Context;
use alkanes_support::id::AlkaneId;
use alkanes_support::parcel::{AlkaneTransfer, AlkaneTransferParcel};
use wasm_bindgen_test::wasm_bindgen_test;
use metashrew_support::index_pointer::KeyValuePointer; // Add this import for from_keyword

// Reset all storage keys used in tests
fn reset_test_storage() {
    // Clear all storage keys used in tests
    StoragePointer::from_keyword("/initialized").set(Arc::new(Vec::new()));
    StoragePointer::from_keyword("/name").set(Arc::new(Vec::new()));
    StoragePointer::from_keyword("/symbol").set(Arc::new(Vec::new()));
    StoragePointer::from_keyword("/asset-name").set(Arc::new(Vec::new()));
    StoragePointer::from_keyword("/asset-symbol").set(Arc::new(Vec::new()));
    StoragePointer::from_keyword("/decimals").set(Arc::new(Vec::new()));
    StoragePointer::from_keyword("/total-supply").set(Arc::new(Vec::new()));
    StoragePointer::from_keyword("/total-assets").set(Arc::new(Vec::new()));
    StoragePointer::from_keyword("/yield-rate").set(Arc::new(Vec::new()));
    StoragePointer::from_keyword("/last-yield-update").set(Arc::new(Vec::new()));
    StoragePointer::from_keyword("/tx-hashes").set(Arc::new(Vec::new()));
}

// A custom vault implementation that allows us to simulate a complete e2e interaction
// This approach lets us control the incoming alkanes and verify outgoing alkanes
struct E2EVault {
    vault_id: AlkaneId,
    asset_id: AlkaneId,
    mock_context: Option<Context>,
}

impl E2EVault {
    fn new() -> Self {
        let vault_id = AlkaneId::default(); // This represents the contract itself
        
        // Use default for the asset ID since the constructor with array is not accessible
        let asset_id = AlkaneId::default();
        
        Self {
            vault_id,
            asset_id,
            mock_context: None,
        }
    }
    
    // Setup a mock context with specified incoming assets
    fn setup_context_with_assets(&mut self, amount: u128) {
        // Create an incoming asset transfer
        let incoming_transfer = AlkaneTransfer {
            id: self.asset_id.clone(), 
            value: amount,
        };
        
        // Create the context with incoming assets
        let mut context = Context::default();
        context.incoming_alkanes = AlkaneTransferParcel(vec![incoming_transfer]);
        context.myself = self.vault_id.clone();
        
        self.mock_context = Some(context);
    }
    
    // Get stored asset ID - this implementation matches the token ID we set up
    fn get_asset_id(&self) -> AlkaneId {
        self.asset_id.clone()
    }
    
    // Helper to assert the outgoing transfers in a response
    fn assert_outgoing_transfer(response: &anyhow::Result<alkanes_support::response::CallResponse>, 
                           expected_id: &AlkaneId,
                           expected_amount: u128) {
        // Verify response exists
        assert!(response.is_ok(), "Response should be successful");
        
        let call_response = response.as_ref().unwrap();
        
        // Find matching transfer
        let matching_transfer = call_response.alkanes.0.iter()
            .find(|transfer| transfer.id == *expected_id && transfer.value == expected_amount);
            
        // Assert matching transfer was found
        assert!(matching_transfer.is_some(), 
                "Expected transfer of {} units with ID {:?} not found in response", 
                expected_amount, expected_id);
    }
}

// Implement the Storage trait
impl Storage for E2EVault {
    fn name_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/name")
    }

    fn symbol_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/symbol")
    }

    fn asset_name_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/asset-name")
    }

    fn asset_symbol_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/asset-symbol")
    }

    fn decimals_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/decimals")
    }

    fn total_supply_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/total-supply")
    }

    fn total_assets_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/total-assets")
    }

    fn yield_rate_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/yield-rate")
    }

    fn last_yield_update_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/last-yield-update")
    }

    fn tx_hashes_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/tx-hashes")
    }
    
    fn initialized_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/initialized")
    }
    
    fn asset_id_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/asset-id")
    }
}

// Implement the Security trait
impl Security for E2EVault {}

// Implement the asset management trait
impl AssetManagement for E2EVault {
    fn context(&self) -> anyhow::Result<Context> {
        match &self.mock_context {
            Some(context) => Ok(context.clone()),
            None => Err(anyhow::anyhow!("No mock context provided"))
        }
    }
    
    fn get_timestamp(&self) -> u64 {
        1000 // Constant timestamp for testing
    }
}

#[test]
#[wasm_bindgen_test]
fn test_e2e_deposit_and_redeem_flow() {
    // Reset storage
    reset_test_storage();

    // Create the test vault
    let mut vault = E2EVault::new();
    
    // Initialize the vault
    assert!(vault.observe_initialization().is_ok());
    
    // Store the asset ID we'll use
    vault.store_asset_id(&vault.asset_id);
    
    // ------------------------------------------------------------------
    // Step 1: Deposit assets to get shares
    // ------------------------------------------------------------------
    
    // Setup a mock context with incoming assets
    let deposit_amount = 100u128;
    vault.setup_context_with_assets(deposit_amount);
    
    // Create a unique transaction hash for the deposit
    let deposit_tx = "tx_deposit_1";
    
    // Perform the deposit
    let alice = "alice";
    let deposit_result = vault.deposit(
        deposit_tx.to_string(),
        alice.to_string(), 
        alice.to_string(),
        deposit_amount
    );
    
    // Assert the deposit worked and verify outgoing share transfer
    E2EVault::assert_outgoing_transfer(
        &deposit_result, 
        &vault.vault_id,  // Shares have the vault's own ID
        deposit_amount    // First deposit has 1:1 ratio
    );
    
    // Verify accounting state
    assert_eq!(vault.get_balance(alice), deposit_amount);
    assert_eq!(vault.total_assets_pointer().get_value::<u128>(), deposit_amount);
    assert_eq!(vault.total_supply_pointer().get_value::<u128>(), deposit_amount);
    
    // ------------------------------------------------------------------
    // Step 2: Redeem shares to get assets back
    // ------------------------------------------------------------------
    
    // Setup a new empty context for redeeming (no incoming assets needed)
    vault.mock_context = Some(Context::default());
    
    // Create a unique transaction hash for the redemption
    let redeem_tx = "tx_redeem_1";
    
    // Redeem half the shares
    let redeem_amount = deposit_amount / 2;
    let redeem_result = vault.redeem(
        redeem_tx.to_string(),
        alice.to_string(),
        alice.to_string(),
        alice.to_string(),
        redeem_amount
    );
    
    // Assert the redemption worked and verify outgoing asset transfer
    E2EVault::assert_outgoing_transfer(
        &redeem_result, 
        &vault.asset_id, // Assets have the asset ID
        redeem_amount    // Redeeming half the shares
    );
    
    // Verify accounting state after redemption
    assert_eq!(vault.get_balance(alice), deposit_amount - redeem_amount);
    assert_eq!(vault.total_assets_pointer().get_value::<u128>(), deposit_amount - redeem_amount);
    assert_eq!(vault.total_supply_pointer().get_value::<u128>(), deposit_amount - redeem_amount);
}

#[test]
#[wasm_bindgen_test]
fn test_e2e_mint_and_withdraw_flow() {
    // Reset storage
    reset_test_storage();

    // Create the test vault
    let mut vault = E2EVault::new();
    
    // Initialize the vault
    assert!(vault.observe_initialization().is_ok());
    
    // Store the asset ID we'll use
    vault.store_asset_id(&vault.asset_id);
    
    // ------------------------------------------------------------------
    // Step 1: Mint exact shares by providing assets
    // ------------------------------------------------------------------
    
    // Set up 2:1 ratio by initializing assets and supply
    vault.total_assets_pointer().set_value(1000u128);
    vault.total_supply_pointer().set_value(500u128);
    
    // Setup a mock context with incoming assets - we need 2x the shares
    let shares_to_mint = 100u128;
    let assets_required = 200u128; // 2:1 ratio
    vault.setup_context_with_assets(assets_required);
    
    // Create a unique transaction hash for the mint
    let mint_tx = "tx_mint_1";
    
    // Perform the mint
    let bob = "bob";
    let mint_result = vault.mint(
        mint_tx.to_string(),
        bob.to_string(), 
        bob.to_string(),
        shares_to_mint
    );
    
    // Assert the mint worked and verify outgoing share transfer
    E2EVault::assert_outgoing_transfer(
        &mint_result, 
        &vault.vault_id,  // Shares have the vault's own ID
        shares_to_mint
    );
    
    // Verify accounting state
    assert_eq!(vault.get_balance(bob), shares_to_mint);
    assert_eq!(vault.total_assets_pointer().get_value::<u128>(), 1000u128 + assets_required);
    assert_eq!(vault.total_supply_pointer().get_value::<u128>(), 500u128 + shares_to_mint);
    
    // ------------------------------------------------------------------
    // Step 2: Withdraw exact assets by providing shares
    // ------------------------------------------------------------------
    
    // Setup a new empty context for withdrawing (no incoming assets needed)
    vault.mock_context = Some(Context::default());
    
    // Create a unique transaction hash for the withdrawal
    let withdraw_tx = "tx_withdraw_1";
    
    // Withdraw some assets
    let assets_to_withdraw = 100u128;
    // Calculate shares needed - at current ratio with total_assets = 1200, total_supply = 600
    // shares = assets * supply / assets = 100 * 600 / 1200 = 50
    let withdraw_result = vault.withdraw(
        withdraw_tx.to_string(),
        bob.to_string(),
        bob.to_string(),
        bob.to_string(),
        assets_to_withdraw
    );
    
    // Assert the withdrawal worked and verify outgoing asset transfer
    E2EVault::assert_outgoing_transfer(
        &withdraw_result, 
        &vault.asset_id,      // Assets have the asset ID
        assets_to_withdraw    // Withdrawing the exact asset amount
    );
    
    // Verify accounting state after withdrawal
    // Shares consumed = shares_to_mint / 2 = 50
    assert_eq!(vault.get_balance(bob), shares_to_mint - 50);
    assert_eq!(vault.total_assets_pointer().get_value::<u128>(), 1000u128 + assets_required - assets_to_withdraw);
    assert_eq!(vault.total_supply_pointer().get_value::<u128>(), 500u128 + shares_to_mint - 50);
}

#[test]
#[wasm_bindgen_test]
fn test_e2e_yield_accrual_benefits() {
    // Reset storage
    reset_test_storage();

    // Create the test vault
    let mut vault = E2EVault::new();
    
    // Initialize the vault
    assert!(vault.observe_initialization().is_ok());
    
    // Store the asset ID we'll use
    vault.store_asset_id(&vault.asset_id);
    
    // ------------------------------------------------------------------
    // Step 1: Alice deposits initially
    // ------------------------------------------------------------------
    
    // Setup a mock context with incoming assets
    let alice_deposit = 1000u128;
    vault.setup_context_with_assets(alice_deposit);
    
    // Perform Alice's deposit
    let alice = "alice";
    let alice_deposit_result = vault.deposit(
        "alice_tx_1".to_string(),
        alice.to_string(), 
        alice.to_string(),
        alice_deposit
    );
    
    // Assert the deposit worked
    assert!(alice_deposit_result.is_ok());
    
    // ------------------------------------------------------------------
    // Step 2: Simulate yield accrual (10% growth)
    // ------------------------------------------------------------------
    
    // Set a yield rate and update time
    vault.yield_rate_pointer().set_value(1000u128); // 10% annual yield
    vault.last_yield_update_pointer().set_value(1000u64); // Start time
    
    // Manually update assets to simulate yield (+10%)
    let yield_amount = alice_deposit / 10; // 10% of 1000 = 100
    vault.total_assets_pointer().set_value(alice_deposit + yield_amount);
    vault.last_yield_update_pointer().set_value(2000u64); // New time
    
    // ------------------------------------------------------------------
    // Step 3: Bob deposits after yield
    // ------------------------------------------------------------------
    
    // Setup a new context with Bob's deposit
    let bob_deposit = 1000u128;
    vault.setup_context_with_assets(bob_deposit);
    
    // Perform Bob's deposit - he should get fewer shares due to appreciation
    let bob = "bob";
    let bob_deposit_result = vault.deposit(
        "bob_tx_1".to_string(),
        bob.to_string(), 
        bob.to_string(),
        bob_deposit
    );
    
    // Expected shares for Bob: assets * totalSupply / totalAssets
    // 1000 * 1000 / 1100 = 909 shares (rounded down)
    let expected_bob_shares = 909u128; // May vary slightly due to rounding
    
    // Assert Bob's deposit worked and verify outgoing share transfer
    E2EVault::assert_outgoing_transfer(
        &bob_deposit_result, 
        &vault.vault_id,
        expected_bob_shares
    );
    
    // ------------------------------------------------------------------
    // Step 4: Both users redeem to verify increased value for early depositor
    // ------------------------------------------------------------------
    
    // Setup for redemption
    vault.mock_context = Some(Context::default());
    
    // Alice redeems all her shares
    let alice_redeem_result = vault.redeem(
        "alice_redeem".to_string(),
        alice.to_string(),
        alice.to_string(),
        alice.to_string(),
        alice_deposit // Alice has 1000 shares
    );
    
    // Alice should get more than she put in due to yield
    let alice_expected_assets = 1100u128; // Original 1000 + 10% yield
    
    // Assert Alice's redemption and verify outgoing asset transfer
    E2EVault::assert_outgoing_transfer(
        &alice_redeem_result, 
        &vault.asset_id,
        alice_expected_assets
    );
    
    // Bob redeems all his shares
    let bob_redeem_result = vault.redeem(
        "bob_redeem".to_string(),
        bob.to_string(),
        bob.to_string(),
        bob.to_string(),
        expected_bob_shares // Bob has ~909 shares
    );
    
    // Bob should get back what he put in (no yield since his deposit)
    let bob_expected_assets = 1000u128; // Original deposit
    
    // Assert Bob's redemption and verify outgoing asset transfer
    E2EVault::assert_outgoing_transfer(
        &bob_redeem_result, 
        &vault.asset_id,
        bob_expected_assets
    );
    
    // Verify Alice got more assets than her initial deposit due to yield
    assert!(alice_expected_assets > alice_deposit, 
            "Alice should benefit from yield accrual before Bob's deposit");
}
