use anyhow::Result;
use crate::contracts::LaunchpadFactory;
use crate::utils::BlockContext;
use super::mock::MockBlockContext;
use std::error::Error as StdError;

#[test]
fn test_successful_bond_redemption() -> Result<()> {
    // Create a mock block context with a specific starting height
    let start_block = 100;
    let mut block_context = MockBlockContext::new().with_block_height(start_block);

    // Create a new LaunchpadFactory with a short maturity period (10 blocks)
    let maturity_blocks = 10;
    let interest_rate_bps = 800; // 8.00% interest
    let mut factory = LaunchpadFactory::new(
        "1.0.0".to_string(),
        maturity_blocks,
        interest_rate_bps,
        &block_context
    );

    // Create a collection
    let collection_id = factory.create_collection(
        "Test Collection".to_string(),
        "TCOL".to_string(),
        Some("Test collection for redemption".to_string()),
        None, // Use default interest rate
        None, // Use default maturity
        &block_context
    );

    // Mint a bond
    let bond_amount = 1000;
    let orbital_id = "test-orbital-123".to_string();

    let bond_id = factory.mint_bond(
        &collection_id,
        orbital_id.clone(),
        bond_amount,
        &block_context
    ).map_err(|e| anyhow::anyhow!(e))?;

    // Verify the bond was created
    {
        let collection = factory.get_collection(&collection_id).unwrap();
        let bond = collection.get_bond(&bond_id).unwrap();
        assert_eq!(bond.amount, bond_amount);
        assert_eq!(bond.orbital_token_id, orbital_id);

        // Verify it is not yet mature
        assert!(!bond.is_mature(&block_context));

        // Verify we can't redeem it yet
        let early_redemption = factory.redeem_bond(
            &collection_id,
            &orbital_id,
            &block_context
        );
        assert!(early_redemption.is_err());
    }

    // Now advance the block height to well past maturity
    // Since maturity is 10 blocks we'll advance 20 blocks to be safe
    block_context.advance_blocks(20);

    // Current block should now be start_block + 20 = 120
    assert_eq!(block_context.get_current_block_height(), start_block + 20);

    // The bond should now be mature
    {
        let collection = factory.get_collection(&collection_id).unwrap();
        let mature_bonds = collection.get_mature_bonds(&block_context);
        assert_eq!(mature_bonds.len(), 1);
    }

    // Redeem the bond
    let redemption_amount = factory.redeem_bond(
        &collection_id,
        &orbital_id,
        &block_context
    ).map_err(|e| anyhow::anyhow!(e))?;

    // Verify the redemption amount (principal + 8% interest)
    let expected_amount = bond_amount + (bond_amount * interest_rate_bps as u64 / 10000);
    assert_eq!(redemption_amount, expected_amount);

    // Bond should now be redeemed and not appear in mature bonds
    {
        let collection = factory.get_collection(&collection_id).unwrap();
        let mature_bonds = collection.get_mature_bonds(&block_context);
        assert_eq!(mature_bonds.len(), 0);
    }

    Ok(())
}
