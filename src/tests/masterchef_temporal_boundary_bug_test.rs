use alkanes::view;
use anyhow::Result;
use bitcoin::blockdata::transaction::OutPoint;
use wasm_bindgen_test::wasm_bindgen_test;
use alkanes::tests::helpers::clear;
use alkanes::indexer::index_block;
use std::str::FromStr;
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
use alkanes_support::trace::Trace;
use alkanes_support::proto::alkanes::AlkanesTrace;
use metashrew_core::{println, stdio::stdout};
use protobuf::Message;
use std::fmt::Write;

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

// Helper to create vault setup with temporal cap
fn create_masterchef_bug_setup(end_reward_block: u128) -> Result<(AlkaneId, AlkaneId, AlkaneId, u128)> {
    clear();
    
    // Deploy contract templates
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
                                    0x42554758,           // name_part1 ("BUGX")
                                    0x544553544,          // name_part2 ("TEST")
                                    0x425447,             // symbol ("BTG")
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
                                message: into_cellpack(vec![2u128, 1u128, 77u128]).encipher(),
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
    
    // Initialize vault with temporal cap
    let reward_per_block = 1000u128;
    let start_block = 3u128;
    
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
                                    start_block, // start_block
                                    end_reward_block, // end reward block (temporal cap)
                                    free_mint_id.block, free_mint_id.tx, // free-mint contract
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
    index_block(&init_vault_block, 3)?;
    let vault_factory_id = AlkaneId { block: 4, tx: 890 };
    
    println!("🚨 MASTERCHEF BUG TEST SETUP:");
    println!("   • Free-mint contract: AlkaneId {{ block: {}, tx: {} }}", free_mint_id.block, free_mint_id.tx);
    println!("   • Deposit token: AlkaneId {{ block: {}, tx: {} }}", deposit_token_id.block, deposit_token_id.tx);
    println!("   • Vault factory: AlkaneId {{ block: {}, tx: 0x{:x} }}", vault_factory_id.block, vault_factory_id.tx);
    println!("   • Reward per block: {} tokens", reward_per_block);
    println!("   • Start block: {}", start_block);
    println!("   • End reward block: {} (TEMPORAL CAP)", end_reward_block);
    
    Ok((free_mint_id, deposit_token_id, vault_factory_id, reward_per_block))
}

// Helper to create deposit tokens
fn create_deposit_tokens(block_height: u32) -> Result<Block> {
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
                                message: into_cellpack(vec![2u128, 1u128, 77u128]).encipher(),
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

// Helper to perform deposit
fn perform_deposit_with_debug(mint_block: &Block, deposit_amount: u128, vault_factory_id: AlkaneId, user_name: &str, block_height: u32) -> Result<(Block, ProtoruneRuneId)> {
    let mint_outpoint = OutPoint {
        txid: mint_block.txdata[0].compute_txid(),
        vout: 0,
    };
    
    let mint_sheet = load_sheet(&RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
        .OUTPOINT_TO_RUNES.select(&consensus_encode(&mint_outpoint)?));
    let token_rune_id = ProtoruneRuneId { block: 2, tx: 1 };
    let available_tokens = mint_sheet.get(&token_rune_id);
    
    println!("💳 {} depositing {} tokens at block {} (available: {})", user_name, deposit_amount, block_height, available_tokens);
    
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
                                        amount: available_tokens,
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
        .ok_or_else(|| anyhow::anyhow!("No position token found for {}", user_name))?;
    
    let position_token_id = ProtoruneRuneId {
        block: position_token_info.0.block,
        tx: position_token_info.0.tx,
    };

    println!("✅ {} deposited successfully - Position token: AlkaneId {{ block: {}, tx: {} }}", user_name, position_token_id.block, position_token_id.tx);
    
    Ok((deposit_block, position_token_id))
}

// Helper to perform withdrawal with detailed analysis
fn perform_withdrawal_with_debug(deposit_block: &Block, position_token_id: ProtoruneRuneId, vault_factory_id: AlkaneId, user_name: &str, block_height: u32, expected_rewards: u128, deposit_amount: u128) -> Result<(u128, u128)> {
    let position_outpoint = OutPoint {
        txid: deposit_block.txdata[0].compute_txid(),
        vout: 0,
    };

    println!("💸 {} withdrawing at block {} (expecting {} rewards)", user_name, block_height, expected_rewards);

    let withdrawal_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
        version: Version::ONE,
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input: vec![TxIn {
            previous_output: position_outpoint,
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
    index_block(&withdrawal_block, block_height)?;

    // Analyze returned tokens
    let withdrawal_outpoint = OutPoint {
        txid: withdrawal_block.txdata[0].compute_txid(),
        vout: 0,
    };
    
    let withdrawal_sheet = load_sheet(
        &RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
            .OUTPOINT_TO_RUNES
            .select(&consensus_encode(&withdrawal_outpoint)?)
    );
    
    let mut total_returned = 0u128;
    for (id, amount) in withdrawal_sheet.balances().iter() {
        if id.block == 2 && id.tx == 1 {
            total_returned += *amount;
        }
    }

    let actual_rewards = total_returned.saturating_sub(deposit_amount);
    
    println!("💰 {} WITHDRAWAL RESULT:", user_name);
    println!("   • Total returned: {} tokens", total_returned);
    println!("   • Principal: {} tokens", deposit_amount);
    println!("   • Actual rewards: {} tokens", actual_rewards);
    println!("   • Expected rewards: {} tokens", expected_rewards);
    
    if actual_rewards == expected_rewards {
        println!("   • ✅ CORRECT REWARDS");
    } else {
        let percentage = if expected_rewards > 0 { (actual_rewards * 100) / expected_rewards } else { 0 };
        println!("   • ❌ REWARD BUG: Got {}% of expected rewards", percentage);
    }
    
    Ok((total_returned, actual_rewards))
}

#[wasm_bindgen_test]
fn test_masterchef_global_state_bug() -> Result<()> {
    println!("🚨 MASTERCHEF GLOBAL STATE BUG DEMONSTRATION");
    println!("============================================");
    
    let end_reward_block = 50u128;
    let (_free_mint_id, _deposit_token_id, vault_factory_id, reward_per_block) = create_masterchef_bug_setup(end_reward_block)?;
    let deposit_amount = 1000u128;
    
    println!("\n🔬 BUG SCENARIO SETUP:");
    println!("   • End reward block: {}", end_reward_block);
    println!("   • Reward rate: {} tokens per block", reward_per_block);
    println!("   • Both users deposit {} tokens", deposit_amount);
    
    // Two users with similar deposit periods but different withdrawal timing
    println!("\n👤 USER A: Early Withdrawer (triggers the bug)");
    let mint_block_a = create_deposit_tokens(4)?;
    let (deposit_block_a, position_token_a) = perform_deposit_with_debug(
        &mint_block_a, 
        deposit_amount, 
        vault_factory_id, 
        "User A", 
        10
    )?;
    
    println!("\n👤 USER B: Boundary Withdrawer (affected by the bug)");
    let mint_block_b = create_deposit_tokens(5)?;
    let (deposit_block_b, position_token_b) = perform_deposit_with_debug(
        &mint_block_b, 
        deposit_amount, 
        vault_factory_id, 
        "User B", 
        20
    )?;
    
    println!("\n📊 EXPECTED CALCULATION (without bug):");
    println!("   • User A: Blocks 10-49 = 39 blocks × 1000 = 39,000 rewards");
    println!("   • User B: Blocks 20-50 = 30 blocks × 1000 = 30,000 rewards");
    println!("   • User B should get 30,000 rewards despite User A's early withdrawal");
    
    // User A withdraws first (this sets global last_reward_block = 49)
    println!("\n💸 USER A WITHDRAWAL (TRIGGER EVENT):");
    let expected_rewards_a = 39u128 * reward_per_block; // 39 blocks
    let (_total_a, actual_rewards_a) = perform_withdrawal_with_debug(
        &deposit_block_a, 
        position_token_a, 
        vault_factory_id, 
        "User A", 
        49, 
        expected_rewards_a, 
        deposit_amount
    )?;
    
    println!("🔍 GLOBAL STATE AFTER USER A WITHDRAWAL:");
    println!("   • last_reward_block is now set to: 49");
    println!("   • This will affect User B's reward calculation!");
    
    // User B withdraws second (this should get 30 blocks but will only get 1 due to bug)
    println!("\n💸 USER B WITHDRAWAL (BUG MANIFESTATION):");
    let expected_rewards_b = 30u128 * reward_per_block; // 30 blocks
    let (_total_b, actual_rewards_b) = perform_withdrawal_with_debug(
        &deposit_block_b, 
        position_token_b, 
        vault_factory_id, 
        "User B", 
        50, 
        expected_rewards_b, 
        deposit_amount
    )?;
    
    println!("\n🐛 BUG ANALYSIS:");
    println!("================");
    println!("   • User A expected: {}, actual: {} ({}% correct)", 
             expected_rewards_a, actual_rewards_a, 
             if expected_rewards_a > 0 { (actual_rewards_a * 100) / expected_rewards_a } else { 0 });
    println!("   • User B expected: {}, actual: {} ({}% correct)", 
             expected_rewards_b, actual_rewards_b, 
             if expected_rewards_b > 0 { (actual_rewards_b * 100) / expected_rewards_b } else { 0 });
    
    // The bug: User B only gets rewards for 1 block (49→50) instead of 30 blocks (20→50)
    if actual_rewards_b < expected_rewards_b / 10 {
        println!("\n🚨 BUG CONFIRMED: MasterChef Global State Bug");
        println!("   🔴 Problem: last_reward_block is shared between all users");
        println!("   🔴 Impact: User B only got rewards for ~1 block instead of 30 blocks");
        println!("   🔴 Root Cause: update_rewards() uses global last_reward_block");
        println!("\n🔧 REQUIRED FIX:");
        println!("   • Change update_rewards() to process full reward history per user");
        println!("   • Or maintain separate reward tracking per position");
        println!("   • Or recalculate acc_reward_per_share based on position creation time");
    } else {
        println!("\n✅ No bug detected - both users got expected rewards");
    }
    
    println!("\n📋 BUG REPRODUCTION SUMMARY:");
    println!("===========================");
    println!("1. User A deposits at block 10");
    println!("2. User B deposits at block 20");  
    println!("3. User A withdraws at block 49 → sets last_reward_block = 49");
    println!("4. User B withdraws at block 50 → only processes blocks 49-50 (1 block)");
    println!("5. User B loses 29 blocks worth of rewards due to global state interference");
    
    Ok(())
}

#[wasm_bindgen_test]
fn test_masterchef_isolation_verification() -> Result<()> {
    println!("🔬 MASTERCHEF ISOLATION VERIFICATION");
    println!("====================================");
    
    let end_reward_block = 100u128;
    let (_free_mint_id, _deposit_token_id, vault_factory_id, reward_per_block) = create_masterchef_bug_setup(end_reward_block)?;
    let deposit_amount = 1000u128;
    
    println!("\n🧪 ISOLATION TEST: Reversing Withdrawal Order");
    println!("   • Same users, same deposit times, REVERSED withdrawal order");
    println!("   • If isolated correctly, results should be identical");
    
    // Same setup but reverse withdrawal order
    println!("\n👤 USER C: Will withdraw SECOND this time");
    let mint_block_c = create_deposit_tokens(6)?;
    let (deposit_block_c, position_token_c) = perform_deposit_with_debug(
        &mint_block_c, 
        deposit_amount, 
        vault_factory_id, 
        "User C", 
        10
    )?;
    
    println!("\n👤 USER D: Will withdraw FIRST this time");
    let mint_block_d = create_deposit_tokens(7)?;
    let (deposit_block_d, position_token_d) = perform_deposit_with_debug(
        &mint_block_d, 
        deposit_amount, 
        vault_factory_id, 
        "User D", 
        20
    )?;
    
    // User D withdraws first this time (20→50 = 30 blocks)
    let expected_rewards_d = 30u128 * reward_per_block; 
    let (_total_d, actual_rewards_d) = perform_withdrawal_with_debug(
        &deposit_block_d, 
        position_token_d, 
        vault_factory_id, 
        "User D", 
        50, 
        expected_rewards_d, 
        deposit_amount
    )?;
    
    // User C withdraws second this time (10→49 = 39 blocks)
    let expected_rewards_c = 39u128 * reward_per_block;
    let (_total_c, actual_rewards_c) = perform_withdrawal_with_debug(
        &deposit_block_c, 
        position_token_c, 
        vault_factory_id, 
        "User C", 
        49, 
        expected_rewards_c, 
        deposit_amount
    )?;
    
    println!("\n🔍 ISOLATION VERIFICATION:");
    println!("=========================");
    println!("   • User C (10→49): {} rewards (expected {})", actual_rewards_c, expected_rewards_c);
    println!("   • User D (20→50): {} rewards (expected {})", actual_rewards_d, expected_rewards_d);
    
    // If the MasterChef algorithm is working correctly, withdrawal order shouldn't matter
    if actual_rewards_c != expected_rewards_c || actual_rewards_d != expected_rewards_d {
        println!("\n🚨 ISOLATION BUG CONFIRMED:");
        println!("   • Withdrawal order affects reward calculation");
        println!("   • This proves the global state interference bug");
    } else {
        println!("\n✅ ISOLATION VERIFIED:");
        println!("   • Withdrawal order doesn't affect rewards");
        println!("   • MasterChef algorithm is working correctly");
    }
    
    Ok(())
}
