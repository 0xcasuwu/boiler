use alkanes::view;
use anyhow::Result;
use bitcoin::blockdata::transaction::OutPoint;
use wasm_bindgen_test::wasm_bindgen_test;
use alkanes::tests::helpers::clear;
use alkanes::indexer::index_block;
use std::str::FromStr;
use std::fmt::Write;
use alkanes::message::AlkaneMessageContext;
use alkanes_support::cellpack::Cellpack;
use alkanes_support::id::AlkaneId;
use alkanes::tests::helpers as alkane_helpers;
use protorune::{balance_sheet::{load_sheet}, tables::RuneTable, message::MessageContext};
use protorune_support::balance_sheet::BalanceSheetOperations;
use bitcoin::{transaction::Version, ScriptBuf, Sequence};
use bitcoin::{Address, Amount, Block, Transaction, TxIn, TxOut, Witness};
use metashrew_support::{index_pointer::KeyValuePointer, utils::consensus_encode};
use ordinals::Runestone;
use protorune::test_helpers::{get_btc_network, ADDRESS1};
use protorune::{test_helpers as protorune_helpers};
use protorune_support::{balance_sheet::ProtoruneRuneId, protostone::{Protostone, ProtostoneEdict}};
use protorune::protostone::Protostones;
use metashrew_core::{println, stdio::stdout};

use crate::precompiled::free_mint_build;
use crate::tests::std::alk4626_position_token_build;
use crate::tests::std::alk4626_vault_factory_build;

pub fn into_cellpack(v: Vec<u128>) -> Cellpack {
    Cellpack {
        target: AlkaneId {
            block: v[0],
            tx: v[1]
        },
        inputs: v[2..].into()
    }
}

// Pool period for proper MasterChef calculations
#[derive(Debug, Clone)]
struct PoolPeriod {
    start_block: u32,
    end_block: u32,
    total_staked: u128,
    active_users: Vec<(String, u128)>, // (user_name, amount)
}

impl PoolPeriod {
    fn get_user_share(&self, user_amount: u128) -> f64 {
        if self.total_staked == 0 {
            0.0
        } else {
            user_amount as f64 / self.total_staked as f64
        }
    }
    
    fn get_period_rewards(&self, reward_per_block: u128) -> u128 {
        let blocks = (self.end_block - self.start_block) as u128;
        blocks * reward_per_block
    }
}

// Position data
#[derive(Debug, Clone)]
struct Position {
    name: String,
    amount: u128,
    deposit_block: u32,
    withdrawal_block: u32,
    expected_rewards: u128,
}

// Calculate proper MasterChef pool-sharing rewards
fn calculate_masterchef_rewards(positions: &[Position], reward_per_block: u128) -> Vec<u128> {
    // First, determine all the time periods with different pool compositions
    let mut all_events: Vec<(u32, String, u128, bool)> = Vec::new(); // (block, user, amount, is_deposit)
    
    for pos in positions {
        all_events.push((pos.deposit_block, pos.name.clone(), pos.amount, true));
        all_events.push((pos.withdrawal_block, pos.name.clone(), pos.amount, false));
    }
    
    // Sort by block
    all_events.sort_by_key(|e| e.0);
    
    // Generate periods
    let mut periods: Vec<PoolPeriod> = Vec::new();
    let mut current_stakers: std::collections::HashMap<String, u128> = std::collections::HashMap::new();
    let mut last_block = 0u32;
    
    for (block, user, amount, is_deposit) in all_events {
        // Close previous period if there were active stakers
        if !current_stakers.is_empty() && block > last_block {
            let total_staked: u128 = current_stakers.values().sum();
            let active_users: Vec<(String, u128)> = current_stakers.iter()
                .map(|(k, v)| (k.clone(), *v)).collect();
            
            periods.push(PoolPeriod {
                start_block: last_block,
                end_block: block,
                total_staked,
                active_users,
            });
        }
        
        // Update current stakers
        if is_deposit {
            current_stakers.insert(user, amount);
        } else {
            current_stakers.remove(&user);
        }
        
        last_block = block;
    }
    
    println!("📊 MASTERCHEF POOL PERIODS:");
    for (i, period) in periods.iter().enumerate() {
        println!("   Period {}: blocks {}-{} ({} blocks)", 
                 i + 1, period.start_block, period.end_block, 
                 period.end_block - period.start_block);
        println!("     Total staked: {} tokens", period.total_staked);
        for (user, amount) in &period.active_users {
            let share = period.get_user_share(*amount);
            println!("     • {}: {} tokens ({:.1}% share)", user, amount, share * 100.0);
        }
        let period_rewards = period.get_period_rewards(reward_per_block);
        println!("     Period rewards: {} tokens", period_rewards);
    }
    
    // Calculate rewards for each position
    let mut user_rewards: std::collections::HashMap<String, u128> = std::collections::HashMap::new();
    
    for period in &periods {
        let period_total_rewards = period.get_period_rewards(reward_per_block);
        
        for (user, amount) in &period.active_users {
            let user_share = period.get_user_share(*amount);
            let user_period_rewards = (period_total_rewards as f64 * user_share) as u128;
            
            *user_rewards.entry(user.clone()).or_insert(0) += user_period_rewards;
            
            println!("   {} in period {}-{}: {:.1}% share × {} rewards = {} tokens", 
                     user, period.start_block, period.end_block, 
                     user_share * 100.0, period_total_rewards, user_period_rewards);
        }
    }
    
    // Return rewards in the same order as positions
    let mut result = Vec::new();
    for pos in positions {
        let rewards = user_rewards.get(&pos.name).unwrap_or(&0);
        result.push(*rewards);
    }
    
    result
}

// Setup the working vault system (copied from long_term_determinism_test.rs)
fn create_working_masterchef_setup() -> Result<(AlkaneId, AlkaneId, AlkaneId, OutPoint)> {
    clear();
    
    // Deploy contract templates using working pattern
    let template_block = alkane_helpers::init_with_multiple_cellpacks_with_tx(
        [
            free_mint_build::get_bytes(),
            alk4626_position_token_build::get_bytes(),
            alk4626_vault_factory_build::get_bytes(),
        ].into(),
        [
            vec![3u128, 797u128, 101u128],
            vec![3u128, 0x379, 10u128],
            vec![3u128, 0x37a, 10u128],
        ].into_iter().map(|v| into_cellpack(v)).collect::<Vec<Cellpack>>()
    );
    index_block(&template_block, 0)?;
    
    // Create free_mint with authorization system
    let free_mint_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
        version: Version::ONE,
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input: vec![TxIn {
            previous_output: OutPoint::null(),
            script_sig: ScriptBuf::new(),
            sequence: Sequence::MAX,
            witness: Witness::new()
        }],
        output: vec![
            TxOut {
                script_pubkey: Address::from_str(ADDRESS1().as_str())
                    .unwrap()
                    .require_network(get_btc_network())
                    .unwrap()
                    .script_pubkey(),
                value: Amount::from_sat(546),
            },
            TxOut {
                script_pubkey: (Runestone {
                    edicts: vec![],
                    etching: None,
                    mint: None,
                    pointer: None,
                    protocol: Some(
                        vec![
                            Protostone {
                                message: into_cellpack(vec![
                                    6u128, 797u128, 0u128, 
                                    100000u128,           // auth tokens  
                                    1000u128,             // value per mint
                                    100000u128,           // total supply
                                    0x46524545,           // name_part1 ("FREE")
                                    0x4d494e54,           // name_part2 ("MINT")
                                    0x46524d,             // symbol ("FRM")
                                    4u128,                // vault_factory_block
                                    0x37a,                // vault_factory_tx
                                ]).encipher(),
                                protocol_tag: AlkaneMessageContext::protocol_tag() as u128,
                                pointer: Some(0),
                                refund: Some(0),
                                from: None,
                                burn: None,
                                edicts: vec![],
                            }
                        ].encipher()?
                    )
                }).encipher(),
                value: Amount::from_sat(546)
            }
        ],
    }]);
    index_block(&free_mint_block, 1)?;
    let free_mint_id = AlkaneId { block: 2, tx: 1 };
    
    // Create deposit token supply
    let deposit_token_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
        version: Version::ONE,
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input: vec![TxIn {
            previous_output: OutPoint::null(),
            script_sig: ScriptBuf::new(),
            sequence: Sequence::MAX,
            witness: Witness::new()
        }],
        output: vec![
            TxOut {
                script_pubkey: Address::from_str(ADDRESS1().as_str())
                    .unwrap()
                    .require_network(get_btc_network())
                    .unwrap()
                    .script_pubkey(),
                value: Amount::from_sat(546),
            },
            TxOut {
                script_pubkey: (Runestone {
                    edicts: vec![],
                    etching: None,
                    mint: None,
                    pointer: None,
                    protocol: Some(
                        vec![
                            Protostone {
                                message: into_cellpack(vec![2u128, 1u128, 77u128, 1000u128]).encipher(),
                                protocol_tag: AlkaneMessageContext::protocol_tag() as u128,
                                pointer: Some(0),
                                refund: Some(0),
                                from: None,
                                burn: None,
                                edicts: vec![],
                            }
                        ].encipher()?
                    )
                }).encipher(),
                value: Amount::from_sat(546)
            }
        ],
    }]);
    index_block(&deposit_token_block, 2)?;
    let deposit_token_id = AlkaneId { block: 2, tx: 1 };
    
    // Initialize vault  
    let end_reward_block = 500u128;
    let reward_per_block = 10u128;
    
    let init_vault_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
        version: Version::ONE,
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input: vec![TxIn {
            previous_output: OutPoint::null(),
            script_sig: ScriptBuf::new(),
            sequence: Sequence::MAX,
            witness: Witness::new()
        }],
        output: vec![
            TxOut {
                script_pubkey: Address::from_str(ADDRESS1().as_str())
                    .unwrap()
                    .require_network(get_btc_network())
                    .unwrap()
                    .script_pubkey(),
                value: Amount::from_sat(546),
            },
            TxOut {
                script_pubkey: (Runestone {
                    edicts: vec![],
                    etching: None,
                    mint: None,
                    pointer: None,
                    protocol: Some(
                        vec![
                            Protostone {
                                message: into_cellpack(vec![
                                    4u128, 0x37a, 0u128, // Initialize vault factory
                                    deposit_token_id.block, deposit_token_id.tx, // deposit token
                                    reward_per_block, // reward per block
                                    3u128, // start_block
                                    end_reward_block, // end reward block
                                    free_mint_id.block, free_mint_id.tx, // free-mint contract
                                ]).encipher(),
                                protocol_tag: AlkaneMessageContext::protocol_tag() as u128,
                                pointer: Some(0),
                                refund: Some(0),
                                from: None,
                                burn: None,
                                edicts: vec![], // ZERO PRELOADED REWARDS
                            }
                        ].encipher()?
                    )
                }).encipher(),
                value: Amount::from_sat(546)
            }
        ],
    }]);
    index_block(&init_vault_block, 3)?;
    let vault_factory_id = AlkaneId { block: 4, tx: 890 };
    
    let vault_auth_outpoint = OutPoint {
        txid: init_vault_block.txdata[0].compute_txid(),
        vout: 0,
    };
    
    Ok((free_mint_id, deposit_token_id, vault_factory_id, vault_auth_outpoint))
}

// Create user tokens (exact amount)
fn create_user_tokens(block_height: u32, amount: u128) -> Result<Block> {
    let mint_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
        version: Version::ONE,
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input: vec![TxIn {
            previous_output: OutPoint::null(),
            script_sig: ScriptBuf::new(),
            sequence: Sequence::from_height(block_height as u16),
            witness: Witness::new()
        }],
        output: vec![
            TxOut {
                script_pubkey: Address::from_str(ADDRESS1().as_str())
                    .unwrap()
                    .require_network(get_btc_network())
                    .unwrap()
                    .script_pubkey(),
                value: Amount::from_sat(546),
            },
            TxOut {
                script_pubkey: (Runestone {
                    edicts: vec![],
                    etching: None,
                    mint: None,
                    pointer: None,
                    protocol: Some(
                        vec![
                            Protostone {
                                message: into_cellpack(vec![2u128, 1u128, 77u128, amount]).encipher(),
                                protocol_tag: AlkaneMessageContext::protocol_tag() as u128,
                                pointer: Some(0),
                                refund: Some(0),
                                from: None,
                                burn: None,
                                edicts: vec![],
                            }
                        ].encipher()?
                    )
                }).encipher(),
                value: Amount::from_sat(546)
            }
        ],
    }]);
    index_block(&mint_block, block_height)?;
    Ok(mint_block)
}

// Perform deposit using working pattern
fn perform_deposit(
    mint_block: &Block, 
    deposit_amount: u128, 
    vault_factory_id: AlkaneId,
    block_height: u32
) -> Result<(Block, ProtoruneRuneId)> {
    let mint_outpoint = OutPoint {
        txid: mint_block.txdata[0].compute_txid(),
        vout: 0,
    };
    
    let free_mint_id = AlkaneId { block: 2, tx: 1 };
    
    let deposit_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
        version: Version::ONE,
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input: vec![TxIn {
            previous_output: mint_outpoint,
            script_sig: ScriptBuf::new(),
            sequence: Sequence::MAX,
            witness: Witness::new()
        }],
        output: vec![
            TxOut {
                script_pubkey: Address::from_str(ADDRESS1().as_str())
                    .unwrap()
                    .require_network(get_btc_network())
                    .unwrap()
                    .script_pubkey(),
                value: Amount::from_sat(546),
            },
            TxOut {
                script_pubkey: (Runestone {
                    edicts: vec![],
                    etching: None,
                    mint: None,
                    pointer: None,
                    protocol: Some(
                        vec![
                            Protostone {
                                message: into_cellpack(vec![
                                    vault_factory_id.block,
                                    vault_factory_id.tx,
                                    1u128, // deposit opcode
                                    deposit_amount
                                ]).encipher(),
                                protocol_tag: AlkaneMessageContext::protocol_tag() as u128,
                                pointer: Some(0),
                                refund: Some(0),
                                from: None,
                                burn: None,
                                edicts: vec![
                                    ProtostoneEdict {
                                        id: ProtoruneRuneId {
                                            block: free_mint_id.block,
                                            tx: free_mint_id.tx
                                        },
                                        amount: deposit_amount,
                                        output: 1,
                                    }
                                ],
                            }
                        ].encipher()?
                    )
                }).encipher(),
                value: Amount::from_sat(546)
            }
        ],
    }]);
    index_block(&deposit_block, block_height)?;
    
    // Get position token
    let position_outpoint = OutPoint {
        txid: deposit_block.txdata[0].compute_txid(),
        vout: 0,
    };
    
    let position_sheet = load_sheet(
        &RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
            .OUTPOINT_TO_RUNES
            .select(&consensus_encode(&position_outpoint)?)
    );
    
    let position_token_info = position_sheet.cached.balances.iter()
        .find(|(id, _amount)| id.block != 2 || id.tx != 1)
        .ok_or_else(|| anyhow::anyhow!("No position token found"))?;
    
    let position_token_id = ProtoruneRuneId {
        block: position_token_info.0.block,
        tx: position_token_info.0.tx,
    };
    
    Ok((deposit_block, position_token_id))
}

// Perform withdrawal
fn perform_withdrawal(
    deposit_block: &Block,
    position_token_id: ProtoruneRuneId,
    vault_factory_id: AlkaneId,
    withdrawal_block: u32
) -> Result<u128> {
    let position_outpoint = OutPoint {
        txid: deposit_block.txdata[0].compute_txid(),
        vout: 0,
    };
    
    let withdrawal_block_tx: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
        version: Version::ONE,
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input: vec![
            TxIn {
                previous_output: position_outpoint,
                script_sig: ScriptBuf::new(),
                sequence: Sequence::MAX,
                witness: Witness::new()
            }
        ],
        output: vec![
            TxOut {
                script_pubkey: Address::from_str(ADDRESS1().as_str())
                    .unwrap()
                    .require_network(get_btc_network())
                    .unwrap()
                    .script_pubkey(),
                value: Amount::from_sat(546),
            },
            TxOut {
                script_pubkey: (Runestone {
                    edicts: vec![],
                    etching: None,
                    mint: None,
                    pointer: None,
                    protocol: Some(
                        vec![
                            Protostone {
                                message: into_cellpack(vec![
                                    vault_factory_id.block,
                                    vault_factory_id.tx,
                                    2u128, // withdraw opcode
                                ]).encipher(),
                                protocol_tag: AlkaneMessageContext::protocol_tag() as u128,
                                pointer: Some(0),
                                refund: Some(0),
                                from: None,
                                burn: None,
                                edicts: vec![
                                    ProtostoneEdict {
                                        id: position_token_id,
                                        amount: 1,
                                        output: 1,
                                    }
                                ],
                            }
                        ].encipher()?
                    )
                }).encipher(),
                value: Amount::from_sat(546)
            }
        ],
    }]);
    index_block(&withdrawal_block_tx, withdrawal_block)?;
    
    // Check withdrawal results
    let withdrawal_outpoint = OutPoint {
        txid: withdrawal_block_tx.txdata[0].compute_txid(),
        vout: 0,
    };
    
    let withdrawal_sheet = load_sheet(
        &RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
            .OUTPOINT_TO_RUNES
            .select(&consensus_encode(&withdrawal_outpoint)?)
    );
    
    let mut total_received = 0u128;
    for (_id, amount) in withdrawal_sheet.balances().iter() {
        total_received += amount;
    }
    
    Ok(total_received)
}

#[wasm_bindgen_test]
fn test_fixed_masterchef_pool_sharing() -> Result<()> {
    println!("🔧 FIXED MASTERCHEF: Pool Sharing Verification");
    println!("==============================================");
    
    let (_free_mint_id, _deposit_token_id, vault_factory_id, _vault_auth_outpoint) = create_working_masterchef_setup()?;
    
    // Define overlapping positions (same as broken test, but with correct expected values)
    let positions = vec![
        Position { name: "Alice".to_string(), amount: 100, deposit_block: 10, withdrawal_block: 35, expected_rewards: 0 }, // Will calculate
        Position { name: "Bob".to_string(), amount: 250, deposit_block: 20, withdrawal_block: 50, expected_rewards: 0 },   // Will calculate
        Position { name: "Charlie".to_string(), amount: 150, deposit_block: 30, withdrawal_block: 55, expected_rewards: 0 }, // Will calculate
        Position { name: "Diana".to_string(), amount: 300, deposit_block: 40, withdrawal_block: 65, expected_rewards: 0 },   // Will calculate
    ];
    
    println!("🎯 POSITION TIMELINE:");
    for pos in &positions {
        let blocks_held = pos.withdrawal_block - pos.deposit_block;
        println!("   • {}: {} tokens, blocks {}-{} ({} blocks)", 
                 pos.name, pos.amount, pos.deposit_block, pos.withdrawal_block, blocks_held);
    }
    
    // Calculate the CORRECT MasterChef rewards with pool sharing
    let reward_per_block = 10u128;
    let correct_rewards = calculate_masterchef_rewards(&positions, reward_per_block);
    
    println!("\n🏆 CORRECT MASTERCHEF POOL-SHARING REWARDS:");
    for (i, (pos, &correct_reward)) in positions.iter().zip(correct_rewards.iter()).enumerate() {
        println!("   • {}: {} tokens → {} rewards (CORRECT)", pos.name, pos.amount, correct_reward);
    }
    
    // Now test the actual implementation
    println!("\n🧪 TESTING ACTUAL VAULT IMPLEMENTATION:");
    
    // Create tokens and deposits for each user
    let alice_tokens = create_user_tokens(5, 100)?;
    let (alice_deposit, alice_position) = perform_deposit(&alice_tokens, 100, vault_factory_id, 10)?;
    
    let bob_tokens = create_user_tokens(15, 250)?;
    let (bob_deposit, bob_position) = perform_deposit(&bob_tokens, 250, vault_factory_id, 20)?;
    
    let charlie_tokens = create_user_tokens(25, 150)?;
    let (charlie_deposit, charlie_position) = perform_deposit(&charlie_tokens, 150, vault_factory_id, 30)?;
    
    let diana_tokens = create_user_tokens(35, 300)?;
    let (diana_deposit, diana_position) = perform_deposit(&diana_tokens, 300, vault_factory_id, 40)?;
    
    // Perform withdrawals
    let alice_total = perform_withdrawal(&alice_deposit, alice_position, vault_factory_id, 35)?;
    let bob_total = perform_withdrawal(&bob_deposit, bob_position, vault_factory_id, 50)?;
    let charlie_total = perform_withdrawal(&charlie_deposit, charlie_position, vault_factory_id, 55)?;
    let diana_total = perform_withdrawal(&diana_deposit, diana_position, vault_factory_id, 65)?;
    
    // Calculate actual rewards (subtract principal)
    let alice_rewards = alice_total.saturating_sub(100);
    let bob_rewards = bob_total.saturating_sub(250);
    let charlie_rewards = charlie_total.saturating_sub(150);
    let diana_rewards = diana_total.saturating_sub(300);
    
    let actual_rewards = vec![alice_rewards, bob_rewards, charlie_rewards, diana_rewards];
    
    println!("\n📊 RESULTS COMPARISON:");
    let mut all_correct = true;
    let tolerance = 5; // Allow small rounding differences
    
    for (i, pos) in positions.iter().enumerate() {
        let expected = correct_rewards[i];
        let actual = actual_rewards[i];
        let diff = (expected as i128 - actual as i128).abs();
        
        let is_correct = diff <= tolerance;
        if !is_correct {
            all_correct = false;
        }
        
        println!("   • {}: Expected {} → Got {} (diff: {}) {}", 
                 pos.name, expected, actual, 
                 expected as i128 - actual as i128,
                 if is_correct { "✅" } else { "❌" });
        
        // Principal safety check
        let expected_principal = pos.amount;
        let actual_principal = match i {
            0 => alice_total.saturating_sub(alice_rewards),
            1 => bob_total.saturating_sub(bob_rewards),
            2 => charlie_total.saturating_sub(charlie_rewards),
            3 => diana_total.saturating_sub(diana_rewards),
            _ => 0,
        };
        
        let principal_safe = actual_principal >= expected_principal;
        println!("     Principal: Expected {} → Got {} {}", 
                 expected_principal, actual_principal,
                 if principal_safe { "✅" } else { "❌ CRITICAL" });
        
        if !principal_safe {
            all_correct = false;
        }
    }
    
    if all_correct {
        println!("\n🎉 MASTERCHEF POOL SHARING: ✅ WORKING CORRECTLY!");
        println!("   🏆 All users received proportionally correct rewards");
        println!("   🛡️ Principal safety maintained for all users");
    } else {
        println!("\n❌ MASTERCHEF POOL SHARING: STILL BROKEN!");
        println!("   🚨 Pool sharing logic needs further fixes");
        return Err(anyhow::anyhow!("MasterChef pool sharing verification failed"));
    }
    
    Ok(())
}
