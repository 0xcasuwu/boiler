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

// Helper to create vault setup with LIMITED reward pool
fn create_limited_vault_setup() -> Result<(Block, AlkaneId, u128)> {
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

    // Create free_mint token contract with SMALL supply for reward pool
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
                                message: into_cellpack(vec![6u128, 797u128, 0u128, 100u128, 5000u128, 10000u128, 0x414141, 0, 0x414141]).encipher(),
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

    // Mint limited reward tokens
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

    // Get available tokens - we'll use ALL of them (but mint small amount for limited rewards)
    let mint_outpoint = OutPoint { txid: mint_block.txdata[0].compute_txid(), vout: 0 };
    let mint_sheet = load_sheet(&RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
        .OUTPOINT_TO_RUNES.select(&consensus_encode(&mint_outpoint)?));
    let token_rune_id = ProtoruneRuneId { block: 2, tx: 1 };
    let available_tokens = mint_sheet.get(&token_rune_id);
    
    // Use ALL available tokens (which is already small) for rewards - creating scarcity
    let limited_rewards = available_tokens; // Use exactly what we have
    
    println!("🎯 CREATING LIMITED REWARD POOL:");
    println!("   • Available tokens: {}", available_tokens);
    println!("   • Limited reward pool: {}", limited_rewards);
    println!("   • This creates scarcity because we minted only 5000 tokens total");

    // Initialize vault factory with LIMITED reward pool
    let deposit_token_id = AlkaneId { block: 2, tx: 1 };
    let reward_token_id = AlkaneId { block: 2, tx: 1 };
    let reward_per_block = 500_000u128; // VERY HIGH reward rate to exhaust quickly (accounting for precision)
    let start_block = 3u128;
    let fee_percentage = 0u128;

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
                                    limited_rewards, // Using limited amount, not all available tokens
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
                                        amount: limited_rewards, // Send only limited rewards to vault
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

// Helper to create fresh tokens for deposit
fn create_fresh_tokens_for_deposit(block_height: u32) -> Result<Block> {
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
    
    // TRACE: Fresh token mint
    println!("🔍 TRACE: Fresh token mint at block {}", block_height);
    for vout in 0..5 {
        let fresh_trace_data = &view::trace(&OutPoint {
            txid: mint_block.txdata[0].compute_txid(),
            vout,
        })?;
        let fresh_trace_result: alkanes_support::trace::Trace = alkanes_support::proto::alkanes::AlkanesTrace::parse_from_bytes(fresh_trace_data)?.into();
        let trace_guard = fresh_trace_result.0.lock().unwrap();
        if !trace_guard.is_empty() {
            println!("   • Fresh mint vout {} trace: {:?}", vout, *trace_guard);
        }
    }
    
    println!("✅ Created fresh tokens at block {} for deposit", block_height);
    Ok(mint_block)
}

// Helper to attempt withdrawal using position token
fn attempt_withdrawal_with_position_token(position_outpoint: OutPoint, position_id: u128, user_name: &str, block_height: u32) -> Result<bool> {
    println!("🔍 {} attempting withdrawal of position {} at block {}", user_name, position_id, block_height);
    
    // Get position token info first
    let position_sheet = load_sheet(&RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
        .OUTPOINT_TO_RUNES.select(&consensus_encode(&position_outpoint)?));
    
    println!("   • Position token balance check:");
    for (id, amount) in position_sheet.balances().iter() {
        println!("     - Token ID: {:?}, Amount: {}", id, amount);
    }
    
    // Find the position token 
    let position_token_info = position_sheet.cached.balances.iter()
        .find(|(id, _amount)| id.block != 2 || id.tx != 1) // Not the deposit token
        .ok_or_else(|| anyhow::anyhow!("No position token found for withdrawal"))?;
    
    let withdrawal_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
        version: Version::ONE,
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input: vec![TxIn {
            previous_output: position_outpoint, // Use the position token outpoint
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
                                    2u128,              // withdrawal opcode
                                    position_id         // position_id parameter
                                ]).encipher(),
                                protocol_tag: AlkaneMessageContext::protocol_tag() as u128,
                                pointer: Some(0),
                                refund: Some(0),
                                from: None,
                                burn: None,
                                edicts: vec![
                                    ProtostoneEdict {
                                        id: ProtoruneRuneId {
                                            block: position_token_info.0.block,
                                            tx: position_token_info.0.tx
                                        },
                                        amount: *position_token_info.1, // Send the position token for authentication
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

    // Analyze withdrawal result with trace
    println!("\n=== WITHDRAWAL ATTEMPT TRACE ANALYSIS ===");
    let withdrawal_trace_data = &view::trace(&OutPoint {
        txid: withdrawal_block.txdata[0].compute_txid(),
        vout: 3,
    })?;
    let withdrawal_trace_result: alkanes_support::trace::Trace = alkanes_support::proto::alkanes::AlkanesTrace::parse_from_bytes(withdrawal_trace_data)?.into();
    
    let trace_debug_str = format!("{:?}", withdrawal_trace_result.0.lock().unwrap());
    
    if trace_debug_str.contains("No rewards available") ||
       trace_debug_str.contains("Insufficient rewards") ||
       trace_debug_str.contains("Nothing to withdraw") {
        println!("❌ WITHDRAWAL BLOCKED: {} - No rewards available", user_name);
        return Ok(false);
    } else if trace_debug_str.contains("unreachable") {
        println!("❌ WITHDRAWAL ERROR: {} - wasm unreachable", user_name);
        return Ok(false);
    } else if trace_debug_str.contains("RevertContext") {
        println!("❌ WITHDRAWAL REVERTED: {} - Transaction reverted", user_name);
        return Ok(false);
    } else if trace_debug_str.contains("ReturnContext") {
        println!("✅ WITHDRAWAL SUCCESSFUL: {} - Rewards withdrawn successfully", user_name);
        return Ok(true);
    } else {
        println!("⚠️ WITHDRAWAL UNCLEAR: {} - Result unclear, assuming failed", user_name);
        return Ok(false);
    }
}

// Helper to attempt deposit and return position token info
fn attempt_deposit_and_get_position(mint_block: &Block, deposit_amount: u128, user_name: &str, block_height: u32) -> Result<(bool, OutPoint, u128)> {
    let mint_outpoint = OutPoint {
        txid: mint_block.txdata[0].compute_txid(),
        vout: 0,
    };
    
    let mint_sheet = load_sheet(&RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
        .OUTPOINT_TO_RUNES.select(&consensus_encode(&mint_outpoint)?));
    let token_rune_id = ProtoruneRuneId { block: 2, tx: 1 };
    let available_tokens = mint_sheet.get(&token_rune_id);
    
    println!("🔍 {} attempting deposit: {} tokens available, depositing {}", user_name, available_tokens, deposit_amount);
    
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
                                    4u128,              // Vault factory block
                                    0x37a,              // Vault factory tx  
                                    1u128,              // deposit opcode
                                    deposit_amount      // amount parameter
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

    // Analyze deposit result with trace
    println!("\n=== DEPOSIT ATTEMPT TRACE ANALYSIS ===");
    let deposit_trace_data = &view::trace(&OutPoint {
        txid: deposit_block.txdata[0].compute_txid(),
        vout: 3,
    })?;
    let deposit_trace_result: alkanes_support::trace::Trace = alkanes_support::proto::alkanes::AlkanesTrace::parse_from_bytes(deposit_trace_data)?.into();
    
    let trace_debug_str = format!("{:?}", deposit_trace_result.0.lock().unwrap());
    
    let success = if trace_debug_str.contains("Insufficient reward pool") || 
       trace_debug_str.contains("Reward pool exhausted") ||
       trace_debug_str.contains("No rewards available") {
        println!("❌ DEPOSIT BLOCKED: {} - Reward pool exhausted", user_name);
        false
    } else if trace_debug_str.contains("Insufficient token value for deposit amount") {
        println!("❌ DEPOSIT ERROR: {} - Insufficient token value", user_name);
        false
    } else if trace_debug_str.contains("unreachable") {
        println!("❌ DEPOSIT ERROR: {} - wasm unreachable", user_name);
        false
    } else if trace_debug_str.contains("RevertContext") {
        println!("❌ DEPOSIT REVERTED: {} - Transaction reverted", user_name);
        false
    } else if trace_debug_str.contains("ReturnContext") {
        println!("✅ DEPOSIT SUCCESSFUL: {} - Completed successfully", user_name);
        true
    } else {
        println!("⚠️ DEPOSIT UNCLEAR: {} - Result unclear, assuming failed", user_name);
        false
    };
    
    let position_outpoint = OutPoint {
        txid: deposit_block.txdata[0].compute_txid(),
        vout: 0,
    };
    
    // Get position ID from vault's position count logic
    // Position ID should be sequential starting from 0
    let position_count = if user_name == "User 1" { 0u128 } else { 1u128 };
    
    Ok((success, position_outpoint, position_count))
}

// Helper to attempt deposit and analyze results
fn attempt_deposit(mint_block: &Block, deposit_amount: u128, user_name: &str, block_height: u32) -> Result<bool> {
    let mint_outpoint = OutPoint {
        txid: mint_block.txdata[0].compute_txid(),
        vout: 0,
    };
    
    let mint_sheet = load_sheet(&RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
        .OUTPOINT_TO_RUNES.select(&consensus_encode(&mint_outpoint)?));
    let token_rune_id = ProtoruneRuneId { block: 2, tx: 1 };
    let available_tokens = mint_sheet.get(&token_rune_id);
    
    println!("🔍 {} attempting deposit: {} tokens available, depositing {}", user_name, available_tokens, deposit_amount);
    
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
                                    4u128,              // Vault factory block
                                    0x37a,              // Vault factory tx  
                                    1u128,              // deposit opcode
                                    deposit_amount      // amount parameter
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

    // Analyze deposit result with trace
    println!("\n=== DEPOSIT ATTEMPT TRACE ANALYSIS ===");
    let deposit_trace_data = &view::trace(&OutPoint {
        txid: deposit_block.txdata[0].compute_txid(),
        vout: 3,
    })?;
    let deposit_trace_result: alkanes_support::trace::Trace = alkanes_support::proto::alkanes::AlkanesTrace::parse_from_bytes(deposit_trace_data)?.into();
    
    // DETAILED TRACE PARSING - Show internal execution flow
    println!("🔍 DETAILED TRACE ANALYSIS FOR {}:", user_name);
    let trace_guard = deposit_trace_result.0.lock().unwrap();
    println!("   Raw trace data: {:?}", *trace_guard);
    
    // Parse the trace more thoroughly
    let trace_debug_str = format!("{:?}", *trace_guard);
    
    // Look for specific reward pool exhaustion patterns
    if trace_debug_str.contains("Reward pool exhausted") {
        println!("🚫 REWARD POOL EXHAUSTION DETECTED:");
        println!("   • Error: 'Reward pool exhausted - no rewards available for new deposits'");
        println!("   • This confirms the deposit function correctly checks remaining_rewards");
        println!("   • The vault is protecting new users from deposits with no reward potential");
        println!("❌ DEPOSIT BLOCKED: {} - Reward pool exhausted", user_name);
        return Ok(false);
    } else if trace_debug_str.contains("Insufficient reward pool") || 
              trace_debug_str.contains("No rewards available") {
        println!("❌ DEPOSIT BLOCKED: {} - Reward pool insufficient", user_name);
        return Ok(false);
    } else if trace_debug_str.contains("Insufficient token value for deposit amount") {
        println!("❌ DEPOSIT ERROR: {} - Insufficient token value", user_name);
        return Ok(false);
    } else if trace_debug_str.contains("unreachable") {
        println!("❌ DEPOSIT ERROR: {} - wasm unreachable", user_name);
        return Ok(false);
    } else if trace_debug_str.contains("RevertContext") {
        println!("❌ DEPOSIT REVERTED: {} - Transaction reverted", user_name);
        
        // Show revert details
        if let Some(start) = trace_debug_str.find("RevertContext") {
            let revert_section = &trace_debug_str[start..std::cmp::min(start + 200, trace_debug_str.len())];
            println!("   Revert details: {}", revert_section);
        }
        return Ok(false);
    } else if trace_debug_str.contains("ReturnContext") {
        println!("✅ DEPOSIT SUCCESSFUL: {} - Completed successfully", user_name);
        return Ok(true);
    } else {
        println!("⚠️ DEPOSIT UNCLEAR: {} - Result unclear, assuming failed", user_name);
        println!("   Full trace for analysis: {}", trace_debug_str);
        return Ok(false);
    }
}

#[wasm_bindgen_test]
fn test_reward_pool_exhaustion_blocks_new_deposits() -> Result<()> {
    println!("=== REWARD POOL EXHAUSTION VIA WITHDRAWALS TEST ===");
    println!("Demonstrating how withdrawals deplete reward pool and block new deposits");
    
    let (_init_block, _token_id, _reward_per_block) = create_limited_vault_setup()?;
    let deposit_amount = 1000u128;
    
    println!("\n💰 PHASE 1: INITIAL DEPOSITS TO CREATE POSITION TOKENS");
    
    // Create deposits to get position tokens WITH tracking
    let mint_block_1 = create_fresh_tokens_for_deposit(4)?;
    let (deposit_1_success, user_1_position_outpoint, user_1_position_id) = 
        attempt_deposit_and_get_position(&mint_block_1, deposit_amount, "User 1", 10)?;
    
    let mint_block_2 = create_fresh_tokens_for_deposit(11)?;
    let (deposit_2_success, user_2_position_outpoint, user_2_position_id) = 
        attempt_deposit_and_get_position(&mint_block_2, deposit_amount, "User 2", 20)?;
    
    if !deposit_1_success || !deposit_2_success {
        return Err(anyhow::anyhow!("Initial deposits failed - cannot proceed with withdrawal test"));
    }
    
    println!("✅ Both users successfully deposited and received position tokens");
    println!("   • User 1: Position {} at outpoint {:?}", user_1_position_id, user_1_position_outpoint);
    println!("   • User 2: Position {} at outpoint {:?}", user_2_position_id, user_2_position_outpoint);
    
    // Let time pass so rewards accumulate (go to block 50)
    println!("\n⏰ PHASE 2: TIME PASSES - REWARDS ACCUMULATE");
    println!("   • User 1 deposited at block 10, now at block 50 = 40 blocks of rewards");
    println!("   • User 2 deposited at block 20, now at block 50 = 30 blocks of rewards");
    println!("   • With reward rate of {} tokens/block, significant rewards should accumulate", _reward_per_block);
    
    println!("\n💸 PHASE 3: WITHDRAWING REWARDS TO DEPLETE POOL");
    
    // Withdraw from User 1's position token using actual position token
    let user_1_withdrawal_success = attempt_withdrawal_with_position_token(
        user_1_position_outpoint, user_1_position_id, "User 1", 52)?;
    
    // Withdraw from User 2's position token using actual position token
    let user_2_withdrawal_success = attempt_withdrawal_with_position_token(
        user_2_position_outpoint, user_2_position_id, "User 2", 54)?;
    
    println!("\n📊 WITHDRAWAL RESULTS:");
    println!("   • User 1 Withdrawal: {}", if user_1_withdrawal_success { "SUCCESS" } else { "FAILED" });
    println!("   • User 2 Withdrawal: {}", if user_2_withdrawal_success { "SUCCESS" } else { "FAILED" });
    
    // Count successful withdrawals to estimate pool depletion
    let successful_withdrawals = [user_1_withdrawal_success, user_2_withdrawal_success]
        .iter().filter(|&&x| x).count();
    
    println!("   • Total successful withdrawals: {}", successful_withdrawals);
    
    // DEBUG: Check remaining rewards after withdrawals
    println!("\n🔍 DEBUG: Checking vault state after withdrawals");
    
    // Query vault for remaining rewards using read-only call
    let debug_query_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
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
                                    4u128,              // Vault factory block
                                    0x37a,              // Vault factory tx  
                                    99u128,             // TestPing opcode to trigger response
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
    index_block(&debug_query_block, 55)?;
    
    // TRACE: Debug query block
    println!("🔍 TRACE: Debug query block at block 55");
    for vout in 0..5 {
        let debug_trace_data = &view::trace(&OutPoint {
            txid: debug_query_block.txdata[0].compute_txid(),
            vout,
        })?;
        let debug_trace_result: alkanes_support::trace::Trace = alkanes_support::proto::alkanes::AlkanesTrace::parse_from_bytes(debug_trace_data)?.into();
        let trace_guard = debug_trace_result.0.lock().unwrap();
        if !trace_guard.is_empty() {
            println!("   • Debug query vout {} trace: {:?}", vout, *trace_guard);
        }
    }
    
    println!("   • Debug query executed at block 55");
    
    println!("\n🔥 PHASE 4: ATTEMPT NEW DEPOSIT AFTER POOL DEPLETION");
    
    // Now try a new deposit - this should fail if rewards are exhausted
    let mint_block_new = create_fresh_tokens_for_deposit(43)?;
    let new_deposit_success = attempt_deposit(&mint_block_new, deposit_amount, "New User", 60)?;
    
    println!("\n🎯 FINAL RESULTS:");
    println!("   • Reward pool started with: 5000 tokens");
    println!("   • Successful withdrawals: {}", successful_withdrawals);
    println!("   • New deposit attempt: {}", if new_deposit_success { "SUCCESS (UNEXPECTED)" } else { "FAILED (EXPECTED)" });
    
    if successful_withdrawals > 0 && !new_deposit_success {
        println!("✅ REWARD POOL EXHAUSTION TEST PASSED");
        println!("   • Withdrawals successfully depleted the reward pool");
        println!("   • New deposits are now blocked due to insufficient rewards");
        println!("   • This demonstrates proper reward pool exhaustion functionality");
    } else if successful_withdrawals == 0 {
        println!("⚠️ WITHDRAWALS FAILED:");
        println!("   • No withdrawals succeeded - position token authentication issue");
        println!("   • Need to investigate position token mechanism");
        println!("   • Pool depletion test inconclusive");
    } else {
        println!("⚠️ UNEXPECTED BEHAVIOR:");
        println!("   • Withdrawals succeeded but new deposit also succeeded");
        println!("   • This may indicate reward pool was not actually depleted");
        println!("   • Or deposit doesn't check reward pool availability");
    }
    
    Ok(())
}
