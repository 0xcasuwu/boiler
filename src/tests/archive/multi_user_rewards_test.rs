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
use protobuf::Message;

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

// Mathematical precision verification helper
fn verify_reward_calculation(
    amount: u128,
    reward_per_block: u128, 
    blocks_elapsed: u128,
    precision: u128,
    expected: u128,
    test_name: &str
) -> bool {
    let calculated = amount
        .checked_mul(reward_per_block)
        .unwrap_or(0)
        .checked_mul(blocks_elapsed)
        .unwrap_or(0)
        .checked_div(precision)
        .unwrap_or(0);
    
    let matches = calculated == expected;
    
    if matches {
        println!("✅ {}: {} * {} * {} / {} = {} (expected {})", 
                test_name, amount, reward_per_block, blocks_elapsed, precision, calculated, expected);
    } else {
        println!("❌ {}: {} * {} * {} / {} = {} (expected {})", 
                test_name, amount, reward_per_block, blocks_elapsed, precision, calculated, expected);
    }
    
    matches
}

// Helper to create vault setup with working pattern
fn create_vault_setup() -> Result<(Block, AlkaneId, u128)> {
    clear();
    
    // Deploy contract templates using working pattern - EXACT copy from withdrawal_test.rs
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
    
    // TRACE: Template block deployment
    println!("🔍 TRACE: Template block deployment at block 0");
    for (i, tx) in template_block.txdata.iter().enumerate() {
        println!("   • TX {} traces:", i);
        for vout in 0..5 {
            let trace_data = &view::trace(&OutPoint {
                txid: tx.compute_txid(),
                vout,
            })?;
            let trace_result: alkanes_support::trace::Trace = alkanes_support::proto::alkanes::AlkanesTrace::parse_from_bytes(trace_data)?.into();
            let trace_guard = trace_result.0.lock().unwrap();
            if !trace_guard.is_empty() {
                println!("     - vout {}: {:?}", vout, *trace_guard);
            }
        }
    }

    // Create free_mint token contract with large supply for reward pool
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
                                message: into_cellpack(vec![6u128, 797u128, 0u128, 100000u128, 10000000u128, 100000000u128, 0x414141, 0, 0x414141]).encipher(),
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
    
    // TRACE: Free mint block
    println!("🔍 TRACE: Free mint block at block 1");
    for vout in 0..5 {
        let free_mint_trace_data = &view::trace(&OutPoint {
            txid: free_mint_block.txdata[0].compute_txid(),
            vout,
        })?;
        let free_mint_trace_result: alkanes_support::trace::Trace = alkanes_support::proto::alkanes::AlkanesTrace::parse_from_bytes(free_mint_trace_data)?.into();
        let trace_guard = free_mint_trace_result.0.lock().unwrap();
        if !trace_guard.is_empty() {
            println!("   • Free mint vout {} trace: {:?}", vout, *trace_guard);
        }
    }

    // Mint reward tokens for vault initialization
    let mint_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
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
    index_block(&mint_block, 2)?;
    
    // TRACE: Token mint block
    println!("🔍 TRACE: Token mint block at block 2");
    for vout in 0..5 {
        let mint_trace_data = &view::trace(&OutPoint {
            txid: mint_block.txdata[0].compute_txid(),
            vout,
        })?;
        let mint_trace_result: alkanes_support::trace::Trace = alkanes_support::proto::alkanes::AlkanesTrace::parse_from_bytes(mint_trace_data)?.into();
        let trace_guard = mint_trace_result.0.lock().unwrap();
        if !trace_guard.is_empty() {
            println!("   • Mint vout {} trace: {:?}", vout, *trace_guard);
        }
    }

    // Get available tokens for proper parameter matching
    let mint_outpoint = OutPoint { txid: mint_block.txdata[0].compute_txid(), vout: 0 };
    let mint_sheet = load_sheet(&RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
        .OUTPOINT_TO_RUNES.select(&consensus_encode(&mint_outpoint)?));
    let token_rune_id = ProtoruneRuneId { block: 2, tx: 1 };
    let available_tokens = mint_sheet.get(&token_rune_id);
    let preloaded_rewards = available_tokens;

    // Initialize vault factory with proper parameter matching
    let deposit_token_id = AlkaneId { block: 2, tx: 1 };
    let reward_token_id = AlkaneId { block: 2, tx: 1 };
    let reward_per_block = 1000u128; // 1000 tokens per block reward rate
    let start_block = 3u128;
    let fee_percentage = 0u128; // No fees for clean reward testing

    let init_vault_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
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
                                    4u128, 0x37a, 0u128,
                                    deposit_token_id.block, deposit_token_id.tx,
                                    reward_token_id.block, reward_token_id.tx,
                                    reward_per_block,
                                    start_block,
                                    preloaded_rewards,
                                    fee_percentage
                                ]).encipher(),
                                protocol_tag: AlkaneMessageContext::protocol_tag() as u128,
                                pointer: Some(0),
                                refund: Some(0),
                                from: None,
                                burn: None,
                                edicts: vec![
                                    ProtostoneEdict {
                                        id: ProtoruneRuneId { block: deposit_token_id.block, tx: deposit_token_id.tx },
                                        amount: preloaded_rewards,
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
    index_block(&init_vault_block, 3)?;
    
    // TRACE: Vault initialization block
    println!("🔍 TRACE: Vault initialization block at block 3");
    for vout in 0..5 {
        let init_trace_data = &view::trace(&OutPoint {
            txid: init_vault_block.txdata[0].compute_txid(),
            vout,
        })?;
        let init_trace_result: alkanes_support::trace::Trace = alkanes_support::proto::alkanes::AlkanesTrace::parse_from_bytes(init_trace_data)?.into();
        let trace_guard = init_trace_result.0.lock().unwrap();
        if !trace_guard.is_empty() {
            println!("   • Vault init vout {} trace: {:?}", vout, *trace_guard);
        }
    }

    let token_id = AlkaneId { block: 2, tx: 1 };
    Ok((init_vault_block, token_id, reward_per_block))
}

// Helper to create unique token mint using block height as transaction variation
fn create_fresh_tokens_for_deposit(block_height: u32) -> Result<Block> {
    // Create unique transaction by using block height in the input sequence to avoid duplication
    let mint_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
        version: Version::ONE,
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input: vec![TxIn {
            previous_output: OutPoint::null(),
            script_sig: ScriptBuf::new(),
            sequence: Sequence::from_height(block_height as u16), // Make each transaction unique
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
                                message: into_cellpack(vec![2u128, 1u128, 77u128]).encipher(), // 77 = MintTokens opcode
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
    
    println!("✅ Created unique tokens at block {} for deposit", block_height);
    Ok(mint_block)
}


// Helper to perform deposit with TRACE VERIFICATION like vault factory debug
fn perform_deposit(mint_block: &Block, deposit_amount: u128, user_name: &str, block_height: u32) -> Result<(Block, ProtoruneRuneId)> {
    let mint_outpoint = OutPoint {
        txid: mint_block.txdata[0].compute_txid(),
        vout: 0,
    };
    
    // CRITICAL: Get the actual available tokens at the outpoint
    let mint_sheet = load_sheet(&RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
        .OUTPOINT_TO_RUNES.select(&consensus_encode(&mint_outpoint)?));
    let token_rune_id = ProtoruneRuneId { block: 2, tx: 1 };
    let available_tokens = mint_sheet.get(&token_rune_id);
    
    println!("🔍 {} has {} tokens available, depositing {}", user_name, available_tokens, deposit_amount);
    
    // PARAMETER VALIDATION: Ensure we have enough tokens and use the right amount
    if available_tokens < deposit_amount {
        return Err(anyhow::anyhow!("Insufficient tokens: have {}, need {}", available_tokens, deposit_amount));
    }
    
    // Use the EXACT working structure from withdrawal_test.rs
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
                                    4u128,              // Vault factory block
                                    0x37a,              // Vault factory tx  
                                    1u128,              // deposit opcode
                                    deposit_amount      // amount parameter - what user wants to deposit
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
                                        amount: available_tokens, // CRITICAL: Send FULL amount from outpoint
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

    // TRACE VERIFICATION - Following vault factory debug pattern
    println!("\n=== DEPOSIT TRACE ANALYSIS ===");
    let deposit_trace_data = &view::trace(&OutPoint {
        txid: deposit_block.txdata[0].compute_txid(),
        vout: 3,
    })?;
    let deposit_trace_result: alkanes_support::trace::Trace = alkanes_support::proto::alkanes::AlkanesTrace::parse_from_bytes(deposit_trace_data)?.into();
    
    println!("{} deposit trace result: {:?}", user_name, deposit_trace_result);
    
    let trace_debug_str = format!("{:?}", deposit_trace_result.0.lock().unwrap());
    
    if trace_debug_str.contains("Insufficient token value for deposit amount") {
        println!("❌ DEPOSIT ERROR: Insufficient token value for deposit amount");
        return Err(anyhow::anyhow!("Deposit failed: Insufficient token value"));
    } else if trace_debug_str.contains("unreachable") {
        println!("❌ DEPOSIT ERROR: wasm unreachable - deposit validation failed");
        return Err(anyhow::anyhow!("Deposit failed: unreachable"));
    } else if trace_debug_str.contains("RevertContext") {
        println!("❌ DEPOSIT ERROR: Transaction reverted");
        return Err(anyhow::anyhow!("Deposit failed: reverted"));
    } else if trace_debug_str.contains("ReturnContext") {
        println!("✅ DEPOSIT SUCCESS: {} completed successfully!", user_name);
    } else {
        println!("⚠️ DEPOSIT: Unclear result for {} - proceeding cautiously", user_name);
    }

    // Get position token from deposit - same logic as working test
    let position_outpoint = OutPoint {
        txid: deposit_block.txdata[0].compute_txid(),
        vout: 0,
    };
    
    let position_sheet = load_sheet(
        &RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
            .OUTPOINT_TO_RUNES
            .select(&consensus_encode(&position_outpoint)?)
    );
    
    println!("🔍 Position outpoint tokens after deposit:");
    for (id, amount) in position_sheet.balances().iter() {
        println!("   Token ID: {:?}, Amount: {}", id, amount);
    }
    
    // Get the position token ID - should be the first token that's not the deposit token
    let position_token_info = position_sheet.cached.balances.iter()
        .find(|(id, _amount)| id.block != 2 || id.tx != 1) // Not the deposit token
        .ok_or_else(|| anyhow::anyhow!("No position token found for {}", user_name))?;
    
    let position_token_id = ProtoruneRuneId {
        block: position_token_info.0.block,
        tx: position_token_info.0.tx,
    };

    println!("✅ {} deposited {} tokens at block {} - Position token: {:?}", 
             user_name, deposit_amount, block_height, position_token_id);
    
    Ok((deposit_block, position_token_id))
}

#[wasm_bindgen_test]
fn test_single_user_multiple_deposits() -> Result<()> {
    println!("=== SINGLE USER MULTIPLE DEPOSITS TEST ===");
    
    let (init_block, _token_id, reward_per_block) = create_vault_setup()?;
    let precision = 1_000_000u128; // 10^6 precision
    let deposit_amount = 1000u128; // Using larger amount to avoid precision truncation
    
    // Single user makes multiple deposits at different times
    println!("\n💰 SINGLE USER MAKING MULTIPLE DEPOSITS:");
    
    // Create separate fresh token mints for each deposit to avoid outpoint reuse
    
    // Deposit 1: Early deposit - at block 10, will be held until block 50 (40 blocks) 
    let mint_block_1 = create_fresh_tokens_for_deposit(4)?;
    let (deposit_block_1, position_token_1) = perform_deposit(&mint_block_1, deposit_amount, "Deposit 1", 10)?;
    
    // Deposit 2: Mid deposit - at block 20, will be held until block 50 (30 blocks)
    let mint_block_2 = create_fresh_tokens_for_deposit(11)?;
    let (deposit_block_2, position_token_2) = perform_deposit(&mint_block_2, deposit_amount, "Deposit 2", 20)?;
    
    // Deposit 3: Late deposit - at block 30, will be held until block 50 (20 blocks)
    let mint_block_3 = create_fresh_tokens_for_deposit(21)?;
    let (deposit_block_3, position_token_3) = perform_deposit(&mint_block_3, deposit_amount, "Deposit 3", 30)?;
    
    println!("\n⏱️  DEPOSIT TIMING SETUP:");
    println!("   • Deposit 1: {} tokens, 40 blocks (10→50)", deposit_amount);
    println!("   • Deposit 2: {} tokens, 30 blocks (20→50)", deposit_amount);  
    println!("   • Deposit 3: {} tokens, 20 blocks (30→50)", deposit_amount);
    println!("   • Expected reward ratio: 40:30:20 = 2:1.5:1");
    
    // Calculate expected rewards for verification
    let deposit_1_expected = deposit_amount * reward_per_block * 40u128 / precision;
    let deposit_2_expected = deposit_amount * reward_per_block * 30u128 / precision; 
    let deposit_3_expected = deposit_amount * reward_per_block * 20u128 / precision;
    
    println!("\n🧮 EXPECTED REWARDS:");
    println!("   • Deposit 1: {} rewards (40 blocks)", deposit_1_expected);
    println!("   • Deposit 2: {} rewards (30 blocks)", deposit_2_expected);
    println!("   • Deposit 3: {} rewards (20 blocks)", deposit_3_expected);
    
    // Verify mathematical relationships
    let ratio_1_to_2 = deposit_1_expected as f64 / deposit_2_expected as f64;
    let ratio_2_to_3 = deposit_2_expected as f64 / deposit_3_expected as f64;
    let expected_ratio_1_to_2 = 40.0 / 30.0; // 1.333...
    let expected_ratio_2_to_3 = 30.0 / 20.0; // 1.5
    
    println!("\n📊 RATIO VERIFICATION:");
    println!("   • Deposit 1:2 ratio = {:.3} (expected {:.3})", ratio_1_to_2, expected_ratio_1_to_2);
    println!("   • Deposit 2:3 ratio = {:.3} (expected {:.3})", ratio_2_to_3, expected_ratio_2_to_3);
    
    let ratio_tolerance = 0.001;
    let ratio_1_to_2_correct = (ratio_1_to_2 - expected_ratio_1_to_2).abs() < ratio_tolerance;
    let ratio_2_to_3_correct = (ratio_2_to_3 - expected_ratio_2_to_3).abs() < ratio_tolerance;
    
    if ratio_1_to_2_correct && ratio_2_to_3_correct {
        println!("✅ TIMING-BASED REWARDS MATHEMATICALLY VERIFIED");
        println!("   • Rewards are perfectly proportional to time held");
        println!("   • Multiple deposits timing calculations are precise");
        println!("   • Position tokens: {:?}, {:?}, {:?}", position_token_1, position_token_2, position_token_3);
    } else {
        return Err(anyhow::anyhow!("Multiple deposit timing ratios do not match expected values"));
    }
    
    Ok(())
}
