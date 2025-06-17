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

// Helper to create vault setup with specific temporal cap boundary
fn create_temporal_cap_vault_setup(end_reward_block: u128) -> Result<(AlkaneId, AlkaneId, AlkaneId, u128)> {
    clear();
    
    // Deploy contract templates using established working pattern
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
    
    println!("🔍 TRACE: Template deployment at block 0 - Temporal Cap Testing Setup");
    println!("   📍 End reward block configured: {}", end_reward_block);
    println!("   🎯 Testing temporal boundary enforcement");
    
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
                                    0x54454d50,           // name_part1 ("TEMP")
                                    0x43415021,           // name_part2 ("CAP!")
                                    0x544350,             // symbol ("TCP")
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
    
    // Initialize vault with TEMPORAL CAP
    let reward_per_block = 1000u128; // Clear 1000 tokens per block for easy math
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
                                    end_reward_block, // *** TEMPORAL CAP ***
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
    
    println!("✅ TEMPORAL CAP VAULT SETUP COMPLETE:");
    println!("   • Free-mint contract: AlkaneId {{ block: {}, tx: {} }}", free_mint_id.block, free_mint_id.tx);
    println!("   • Deposit token: AlkaneId {{ block: {}, tx: {} }}", deposit_token_id.block, deposit_token_id.tx);
    println!("   • Vault factory: AlkaneId {{ block: {}, tx: 0x{:x} }}", vault_factory_id.block, vault_factory_id.tx);
    println!("   • Reward per block: {} tokens", reward_per_block);
    println!("   • Start block: {}", start_block);
    println!("   • *** END REWARD BLOCK: {} *** (TEMPORAL CAP)", end_reward_block);
    
    Ok((free_mint_id, deposit_token_id, vault_factory_id, reward_per_block))
}

// Helper to create deposit tokens with unique transactions
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
    
    println!("🪙 Created deposit tokens at block {}", block_height);
    Ok(mint_block)
}

// Helper to perform deposit with temporal cap awareness
fn perform_temporal_deposit(mint_block: &Block, deposit_amount: u128, vault_factory_id: AlkaneId, user_name: &str, block_height: u32) -> Result<(Block, ProtoruneRuneId)> {
    let mint_outpoint = OutPoint {
        txid: mint_block.txdata[0].compute_txid(),
        vout: 0,
    };
    
    // Get available tokens
    let mint_sheet = load_sheet(&RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
        .OUTPOINT_TO_RUNES.select(&consensus_encode(&mint_outpoint)?));
    let token_rune_id = ProtoruneRuneId { block: 2, tx: 1 };
    let available_tokens = mint_sheet.get(&token_rune_id);
    
    println!("💳 {} depositing {} tokens at block {} (temporal boundary test)", user_name, deposit_amount, block_height);
    
    if available_tokens < deposit_amount {
        return Err(anyhow::anyhow!("Insufficient tokens: have {}, need {}", available_tokens, deposit_amount));
    }
    
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
                                    deposit_amount // amount parameter - what user wants to deposit
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

    // Trace verification
    let deposit_trace_data = &view::trace(&OutPoint {
        txid: deposit_block.txdata[0].compute_txid(),
        vout: 3,
    })?;
    let deposit_trace_result: Trace = AlkanesTrace::parse_from_bytes(deposit_trace_data)?.into();
    let trace_debug_str = format!("{:?}", deposit_trace_result.0.lock().unwrap());
    
    if trace_debug_str.contains("ReturnContext") {
        println!("✅ {} deposit SUCCESS at block {}", user_name, block_height);
    } else {
        println!("❌ {} deposit FAILED at block {}", user_name, block_height);
        return Err(anyhow::anyhow!("Deposit failed for {}", user_name));
    }

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

    println!("🎫 {} position token: AlkaneId {{ block: {}, tx: {} }}", user_name, position_token_id.block, position_token_id.tx);
    
    Ok((deposit_block, position_token_id))
}

// Helper to perform withdrawal with temporal cap analysis
fn perform_temporal_withdrawal(deposit_block: &Block, position_token_id: ProtoruneRuneId, vault_factory_id: AlkaneId, user_name: &str, block_height: u32, expected_rewards: u128, deposit_amount: u128) -> Result<u128> {
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

    // Trace verification
    let withdrawal_trace_data = &view::trace(&OutPoint {
        txid: withdrawal_block.txdata[0].compute_txid(),
        vout: 3,
    })?; 
    let withdrawal_trace_result: Trace = AlkanesTrace::parse_from_bytes(withdrawal_trace_data)?.into();
    let trace_debug_str = format!("{:?}", withdrawal_trace_result.0.lock().unwrap());
    
    if trace_debug_str.contains("ReturnContext") {
        println!("✅ {} withdrawal SUCCESS at block {}", user_name, block_height);
    } else {
        println!("❌ {} withdrawal FAILED at block {}", user_name, block_height);
        return Err(anyhow::anyhow!("Withdrawal failed for {}", user_name));
    }

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
    println!("   • Temporal cap compliance: {}", if actual_rewards <= expected_rewards { "✅" } else { "❌" });
    
    Ok(total_returned)
}

#[wasm_bindgen_test]
fn test_temporal_cap_boundary_enforcement_corrected() -> Result<()> {
    println!("🚨 TEMPORAL CAP MANIPULATION TEST: Pool Sharing Corrected");
    println!("🎯 Objective: Verify rewards with proper MasterChef pool sharing");
    
    let end_reward_block = 50u128;
    let (_free_mint_id, _deposit_token_id, vault_factory_id, reward_per_block) = create_temporal_cap_vault_setup(end_reward_block)?;
    let deposit_amount = 1000u128;
    
    println!("\n⏰ TEMPORAL BOUNDARY TEST SCENARIO:");
    println!("   • End reward block: {}", end_reward_block);
    println!("   • Reward rate: {} tokens per block", reward_per_block);
    println!("   • Testing exact boundary behavior");
    
    // User A: Deposit early, withdraw before cap
    println!("\n👤 USER A: PRE-CAP WITHDRAWAL");
    let mint_block_a = create_deposit_tokens(4)?;
    let (deposit_block_a, position_token_a) = perform_temporal_deposit(&mint_block_a, deposit_amount, vault_factory_id, "User A", 10)?;
    
    // User B: Deposit later, withdraw exactly at cap boundary  
    println!("\n👤 USER B: EXACT CAP BOUNDARY WITHDRAWAL"); 
    let mint_block_b = create_deposit_tokens(5)?;
    let (deposit_block_b, position_token_b) = perform_temporal_deposit(&mint_block_b, deposit_amount, vault_factory_id, "User B", 20)?;
    
    println!("\n📊 CORRECTED CALCULATION (with pool sharing):");
    println!("   🧮 MASTERCHEF POOL SHARING CALCULATION:");
    println!("   • Blocks 10-20 (10 blocks): User A SOLO → 10,000 rewards to A");
    println!("   • Blocks 20-49 (29 blocks): A+B SHARE → 29,000 ÷ 2 = 14,500 each");
    println!("   • Block 50 (1 block): User B SOLO + TEMPORAL CAP → 0 rewards (post-cap)");
    println!("   📊 CORRECTED EXPECTATIONS:");
    println!("   • User A: 10,000 + 14,500 = 24,500 rewards");
    println!("   • User B: 14,500 + 0 = 14,500 rewards");
    
    // User A withdraws first - corrected expectation with pool sharing
    println!("\n💸 USER A WITHDRAWAL (CORRECTED EXPECTATION):");
    let expected_rewards_a = 10u128 * reward_per_block + (29u128 * reward_per_block) / 2; // 10 solo + 14.5 shared
    let total_a = perform_temporal_withdrawal(&deposit_block_a, position_token_a, vault_factory_id, "User A", 49, expected_rewards_a, deposit_amount)?;
    
    // User B withdraws second - corrected expectation with pool sharing and temporal cap
    println!("\n💸 USER B WITHDRAWAL (CORRECTED EXPECTATION):");
    let expected_rewards_b = (29u128 * reward_per_block) / 2; // 14.5 shared, 0 for block 50 due to temporal cap
    let total_b = perform_temporal_withdrawal(&deposit_block_b, position_token_b, vault_factory_id, "User B", 50, expected_rewards_b, deposit_amount)?;
    
    // VERIFICATION: Corrected temporal cap enforcement
    println!("\n🔍 CORRECTED TEMPORAL CAP VERIFICATION:");
    
    let actual_rewards_a = total_a.saturating_sub(deposit_amount);
    let actual_rewards_b = total_b.saturating_sub(deposit_amount);
    
    println!("   • User A (corrected): {} rewards (expected {})", actual_rewards_a, expected_rewards_a);
    println!("   • User B (corrected): {} rewards (expected {})", actual_rewards_b, expected_rewards_b);
    
    // Verify results are within reasonable range (account for MasterChef precision)
    let tolerance = reward_per_block / 10; // 10% tolerance for precision issues
    let a_correct = (actual_rewards_a as i128 - expected_rewards_a as i128).abs() <= tolerance as i128;
    let b_correct = (actual_rewards_b as i128 - expected_rewards_b as i128).abs() <= tolerance as i128;
    
    println!("\n🚨 CORRECTED TEMPORAL CAP ENFORCEMENT RESULTS:");
    println!("   • User A pool sharing accuracy: {}", if a_correct { "✅ PASS" } else { "❌ FAIL" });
    println!("   • User B pool sharing accuracy: {}", if b_correct { "✅ PASS" } else { "❌ FAIL" });
    
    // Key insight: Users C and D (post-cap) should still get 0 rewards
    println!("\n👤 USER C: POST-CAP WITHDRAWAL");
    let mint_block_c = create_deposit_tokens(6)?;
    let (deposit_block_c, position_token_c) = perform_temporal_deposit(&mint_block_c, deposit_amount, vault_factory_id, "User C", 30)?;
    let expected_rewards_c = 0u128; // No rewards after temporal cap
    let total_c = perform_temporal_withdrawal(&deposit_block_c, position_token_c, vault_factory_id, "User C", 60, expected_rewards_c, deposit_amount)?;
    let actual_rewards_c = total_c.saturating_sub(deposit_amount);
    
    println!("\n👤 USER D: AT-CAP DEPOSIT, POST-CAP WITHDRAWAL");
    let mint_block_d = create_deposit_tokens(7)?;
    let (deposit_block_d, position_token_d) = perform_temporal_deposit(&mint_block_d, deposit_amount, vault_factory_id, "User D", 50)?;
    let expected_rewards_d = 0u128; // No rewards, deposited at temporal cap
    let total_d = perform_temporal_withdrawal(&deposit_block_d, position_token_d, vault_factory_id, "User D", 70, expected_rewards_d, deposit_amount)?;
    let actual_rewards_d = total_d.saturating_sub(deposit_amount);
    
    let temporal_cap_working = actual_rewards_c == 0 && actual_rewards_d == 0;
    
    println!("   • User C (post-cap): {} rewards (expected 0)", actual_rewards_c);
    println!("   • User D (at-cap): {} rewards (expected 0)", actual_rewards_d);
    println!("   • Temporal cap enforcement: {}", if temporal_cap_working { "✅ PASS" } else { "❌ FAIL" });
    
    if a_correct && b_correct && temporal_cap_working {
        println!("\n🎉 TEMPORAL CAP MANIPULATION TEST: ✅ PASSED (CORRECTED)");
        println!("   ✅ MasterChef pool sharing working correctly");
        println!("   ✅ Temporal boundary enforcement working correctly");
        println!("   ✅ Users get proportional rewards based on staking time");
        println!("   ✅ Post-cap users correctly get zero rewards");
        println!("   📊 Mathematical precision verified with MasterChef algorithm");
    } else {
        println!("\n❌ TEMPORAL CAP MANIPULATION TEST: FAILED");
        return Err(anyhow::anyhow!("Corrected temporal cap test verification failed"));
    }
    
    Ok(())
}

#[wasm_bindgen_test]
fn test_exact_boundary_edge_cases() -> Result<()> {
    println!("🎯 EXACT BOUNDARY EDGE CASES: Precise Temporal Testing");
    println!("🔬 Objective: Test exact block boundary behavior");
    
    let end_reward_block = 25u128; // Smaller boundary for precise testing
    let (_free_mint_id, _deposit_token_id, vault_factory_id, reward_per_block) = create_temporal_cap_vault_setup(end_reward_block)?;
    let deposit_amount = 1000u128;
    
    println!("\n🔬 PRECISE BOUNDARY TEST CASES:");
    println!("   • End reward block: {}", end_reward_block);
    println!("   • Testing exact block transitions");
    
    // Edge Case 1: Deposit at boundary-1, withdraw at boundary
    println!("\n🎯 EDGE CASE 1: Boundary-1 Deposit, Boundary Withdrawal");
    let mint_block_edge1 = create_deposit_tokens(8)?;
    let (deposit_block_edge1, position_token_edge1) = perform_temporal_deposit(&mint_block_edge1, deposit_amount, vault_factory_id, "Edge1", 24)?;
    
    // Expected: 1 block of rewards (block 24-25, but cap at 25, so only gets block 24)
    let expected_rewards_edge1 = 1u128 * reward_per_block;
    let total_edge1 = perform_temporal_withdrawal(&deposit_block_edge1, position_token_edge1, vault_factory_id, "Edge1", 25, expected_rewards_edge1, deposit_amount)?;
    let actual_rewards_edge1 = total_edge1.saturating_sub(deposit_amount);
    
    // Edge Case 2: Deposit at boundary, withdraw post-boundary
    println!("\n🎯 EDGE CASE 2: Boundary Deposit, Post-Boundary Withdrawal");
    let mint_block_edge2 = create_deposit_tokens(9)?;
    let (deposit_block_edge2, position_token_edge2) = perform_temporal_deposit(&mint_block_edge2, deposit_amount, vault_factory_id, "Edge2", 25)?;
    
    // Expected: 0 rewards (deposited exactly at temporal cap)
    let expected_rewards_edge2 = 0u128;
    let total_edge2 = perform_temporal_withdrawal(&deposit_block_edge2, position_token_edge2, vault_factory_id, "Edge2", 30, expected_rewards_edge2, deposit_amount)?;
    let actual_rewards_edge2 = total_edge2.saturating_sub(deposit_amount);
    
    println!("\n🔍 EDGE CASE VERIFICATION:");
    println!("   • Edge1 (boundary-1): {} rewards (expected {})", actual_rewards_edge1, expected_rewards_edge1);
    println!("   • Edge2 (boundary): {} rewards (expected {})", actual_rewards_edge2, expected_rewards_edge2);
    
    let edge_cases_correct = actual_rewards_edge1 == expected_rewards_edge1 && actual_rewards_edge2 == expected_rewards_edge2;
    
    if edge_cases_correct {
        println!("\n🎉 EXACT BOUNDARY EDGE CASES: ✅ PASSED");
        println!("   ✅ Boundary-1 deposits get correct rewards");
        println!("   ✅ Boundary deposits get zero rewards");
        println!("   ✅ Temporal cap precision verified");
    } else {
        println!("\n❌ EXACT BOUNDARY EDGE CASES: FAILED");
        return Err(anyhow::anyhow!("Edge case verification failed"));
    }
    
    Ok(())
}
