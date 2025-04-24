//! Example showing the Orbital Bond Launchpad in action
//! This demonstrates creating a bond collection and minting orbitals that function as bonds,
//! which can be redeemed at maturity.

use slop::contracts::LaunchpadFactory;
use slop::utils::{BlockContext, StandaloneBlockContext};
use std::thread::sleep;
use std::time::Duration;

fn main() {
    println!("=== Orbital Bond Launchpad Example ===");

    // Create a block context for tracking
    let block_context = StandaloneBlockContext::with_seconds_per_block(2);
    
    println!("Creating LaunchpadFactory...");
    
    // Create a LaunchpadFactory
    let mut factory = LaunchpadFactory::new(
        "1.0.0".to_string(),
        25,    // Default maturity of 25 blocks (50 seconds in this example)
        500,   // Default interest rate of 5% (500 basis points)
        &block_context,
    );
    
    println!("LaunchpadFactory created with version: {}", factory.version);
    
    // Create a bond collection
    let collection_id = factory.create_collection(
        "Example Bonds".to_string(),
        "XBND".to_string(),
        Some("Example bond collection for demonstration".to_string()),
        Some(800),  // 8% interest rate
        None,       // Use default maturity
        &block_context,
    );
    
    println!("Created bond collection: {}", collection_id);
    
    {
        // Print collection details
        let collection = factory.get_collection(&collection_id).unwrap();
        println!("Collection Name: {}", collection.name);
        println!("Collection Symbol: {}", collection.symbol);
        println!("Interest Rate: {}%", collection.interest_rate_bps as f64 / 100.0);
        println!("Maturity Period: {} blocks", collection.maturity_blocks);
    }
    
    println!("\n=== Minting Orbital Bonds ===");
    
    // Mint orbital bond tokens
    let mut orbital_ids = Vec::new();
    for i in 1..=3 {
        let amount = 1000 * i; // Different amounts for each bond
        
        // When minting an orbital bond, the launchpad creates an orbital token
        // that functions as the bond - the orbital itself IS the bond
        let orbital_id = format!("orbital-{}", i);
        let bond_id = factory.mint_bond(
            &collection_id,
            orbital_id.clone(),
            amount,
            &block_context,
        ).unwrap();
        
        orbital_ids.push(orbital_id.clone());
        
        println!("Minted orbital bond {} with amount {} (orbital ID: {})", 
            bond_id, amount, orbital_id);
        println!("  - The orbital token itself functions as the bond and authentication token");
    }
    
    // Display all bonds in the collection
    {
        let collection = factory.get_collection(&collection_id).unwrap();
        println!("\nTotal bonds in collection: {}", collection.total_bonds());
    }
    
    // Calculate total value
    let total_value = factory.total_value(&block_context);
    println!("Total value of all active bonds: {}", total_value);
    
    // Fast-forward time/blocks to bond maturity
    println!("\n=== Fast-forwarding to bond maturity (sleeping for a moment) ===");
    
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    let modify_test = args.len() > 1 && args[1] == "--modify-test";
    
    // In a real blockchain context, we would wait for actual blocks
    // Here we simulate by sleeping and creating a new context at a later time
    sleep(Duration::from_secs(5));
    
    // Create a new context from the baseline of the original context
    // This ensures we're using the same block numbering scheme
    let base_block = block_context.get_current_block_height();
    
    // First get a bond to examine its maturity block
    let mut reference_maturity_block = 0;
    if !orbital_ids.is_empty() {
        let collection = factory.get_collection(&collection_id).unwrap();
        if let Some(bond) = collection.get_bond_by_orbital(&orbital_ids[0]) {
            reference_maturity_block = bond.maturity_block;
            println!("Reference bond matures at block: {}", reference_maturity_block);
        }
    }
    
    // Create a custom test context that returns a very high block number for redemption testing
    struct TestMaturityContext {
        block_height: u64,
    }
    
    impl BlockContext for TestMaturityContext {
        fn get_current_block_height(&self) -> u64 {
            self.block_height
        }
    }
    
    // For test mode, we'll create a context with a guaranteed high block number
    let mature_context = if modify_test {
        println!("Test mode: Creating high block context for testing...");
        
        // First get a reference bond and its maturity block
        let mut highest_maturity_block = 0;
        if let Some(collection) = factory.get_collection(&collection_id) {
            for bond in collection.get_all_bonds() {
                highest_maturity_block = highest_maturity_block.max(bond.maturity_block);
            }
        }
        
        // Create context with block height much higher than maturity
        let test_block = highest_maturity_block + 1000;
        println!("Using test context with block height: {} (highest maturity: {})", 
                 test_block, highest_maturity_block);
        TestMaturityContext { block_height: test_block }
    } else {
        // For regular execution, create a TestMaturityContext with the default block height
        let context = StandaloneBlockContext::new();
        let height = context.get_current_block_height();
        TestMaturityContext { block_height: height }
    };
    
    println!("Fast-forwarded to block height: {}", 
        mature_context.get_current_block_height());
    
    // Check which bonds are mature
    {
        let collection = factory.get_collection(&collection_id).unwrap();
        let mature_bonds = collection.get_mature_bonds(&mature_context);
        println!("Mature bonds: {}", mature_bonds.len());
        
        // Detailed maturity debugging
        if modify_test {
            println!("\n=== Maturity Debug Information ===");
            println!("Current block height: {}", mature_context.get_current_block_height());
            println!("Collection maturity blocks: {}", collection.maturity_blocks);
            
            // Check each bond's maturity individually
            for (i, orbital_id) in orbital_ids.iter().enumerate() {
                if let Some(bond) = collection.get_bond_by_orbital(orbital_id) {
                    println!("Bond {} (orbital {}): Created at block {}, Matures at block {}", 
                        i+1, orbital_id, bond.creation_block, bond.maturity_block);
                    println!("  Is mature: {}", bond.is_mature(&mature_context));
                }
            }
            println!("=================================\n");
        }
    }
    
    // Redeem the first bond
    println!("\n=== Redeeming Orbital Bond ===");
    
    if !orbital_ids.is_empty() {
        let orbital_id = &orbital_ids[0];
        
        // Present the orbital to redeem the bond
        // Since the orbital IS the bond, presenting it proves ownership
        let redemption_result = factory.redeem_bond(
            &collection_id,
            orbital_id,
            &mature_context,
        );
        
        match redemption_result {
            Ok(amount) => {
                println!("Successfully redeemed orbital bond {}", orbital_id);
                println!("Redemption amount: {} (includes principal + interest)", amount);
            },
            Err(e) => {
                println!("Failed to redeem orbital bond: {}", e);
            }
        }
    }
    
    // Final summary
    println!("\n=== Launchpad Summary ===");
    println!("Total collections: {}", factory.total_collections());
    println!("Active collections: {}", factory.active_collections_count());
    println!("Total active bonds value: {}", factory.total_value(&mature_context));
    
    println!("\nExample completed successfully!");
}
