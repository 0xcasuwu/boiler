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

// QA-specific setup with EXACT QA parameters
fn create_qa_environment_setup() -> Result<(AlkaneId, AlkaneId, u128, OutPoint)> {
    clear();
    
    println!("🚨 QA ENVIRONMENT SETUP - EXACT QA PARAMETERS");
    println!("=============================================");
    println!("   • reward_per_block: 100 (QA value, not 1000)"); 
    println!("   • end_reward_block: 30000 (QA value, not 1000)");
    println!("   • This may trigger precision/rounding edge cases!");
    
    // PHASE 1: Deploy contract templates (same as before)
    println!("\n📦 PHASE 1: Deploying Contract Templates");
    let template_block = alkane_helpers::init_with_multiple_cellpacks_with_tx(
        [
            free_mint_build::get_bytes(),
            alk4626_position_token_build::get_bytes(),
            alk4626_vault_factory_build::get_bytes(),
            crate::precompiled::auth_token_build::get_bytes(),
        ].into(),
        [
            vec![3u128, 797u128, 101u128],
            vec![3u128, 0x385, 10u128],
            vec![3u128, 0x37a, 10u128],
            vec![3u128, 0xffee, 0u128, 1u128],
        ].into_iter().map(|v| into_cellpack(v)).collect::<Vec<Cellpack>>()
    );
    index_block(&template_block, 0)?;
    println!("✅ Contract templates deployed at block 0");
    
    // PHASE 2: Initialize Free-Mint Contract
    println!("\n🪙 PHASE 2: Initializing Free-Mint Contract");
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
                                    6u128, 797u128, 0u128,  // Deploy to block 6, tx 797, opcode 0 (Initialize)
                                    1000000u128,            // token_units (initial supply)
                                    100000u128,             // value_per_mint  
                                    1000000000u128,         // cap (high cap for testing)
                                    0x54455354,             // name_part1 ("TEST")
                                    0x434f494e,             // name_part2 ("COIN")
                                    0x545354,               // symbol ("TST")
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
    
    let free_mint_contract_id = AlkaneId { block: 2, tx: 1 };
    let free_mint_auth_token_id = AlkaneId { block: 2, tx: 2 };
    
    println!("✅ Free-mint contract initialized at {:?}", free_mint_contract_id);
    println!("🔑 Auth token created at {:?}", free_mint_auth_token_id);
    
    // PHASE 3: Initialize Vault Factory with EXACT QA PARAMETERS
    println!("\n🏭 PHASE 3: Initializing Vault Factory with QA Parameters");
    let deposit_token_id = AlkaneId { block: 2, tx: 1 }; // Same as free-mint for simplicity
    let reward_per_block = 100u128; // 🚨 QA VALUE: 100 (not 1000)
    let start_block = 1u128; // 🚨 QA VALUE: 1 (not 3)
    let end_reward_block = 30000u128; // 🚨 QA VALUE: 30000 (not 1000)
    
    println!("🚨 CRITICAL QA PARAMETERS:");
    println!("   • reward_per_block: {} (10x smaller than test)", reward_per_block);
    println!("   • start_block: {}", start_block);
    println!("   • end_reward_block: {} (30x larger than test)", end_reward_block);
    
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
                                    reward_per_block, // 🚨 QA VALUE: 100
                                    start_block, // 🚨 QA VALUE: 1
                                    end_reward_block, // 🚨 QA VALUE: 30000
                                    free_mint_contract_id.block, free_mint_contract_id.tx, // free-mint contract
                                ]).encipher(),
                                protocol_tag: AlkaneMessageContext::protocol_tag() as u128,
                                pointer: Some(0),
                                refund: Some(0),
                                from: None,
                                burn: None,
                                edicts: vec![], // NO preloaded rewards - on-demand minting
                            }
                        ].encipher()?
                    )
                }).encipher(),
                value: Amount::from_sat(546)
            }
        ],
    }]);
    index_block(&init_vault_block, 3)?;
    
    let vault_factory_id = AlkaneId { block: 4, tx: 0x37a };
    
    println!("✅ Vault factory initialized at {:?}", vault_factory_id);
    println!("🔗 Linked to free-mint contract: {:?}", free_mint_contract_id);
    
    // PHASE 4: Factory Authorization (same as before)
    println!("\n🔐 PHASE 4: Factory Authorization");
    let auth_token_outpoint = OutPoint {
        txid: free_mint_block.txdata[0].compute_txid(),
        vout: 0,
    };
    
    let auth_sheet = load_sheet(&RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
        .OUTPOINT_TO_RUNES.select(&consensus_encode(&auth_token_outpoint)?));
    let auth_token_rune_id = ProtoruneRuneId { block: 2, tx: 2 };
    let available_auth_tokens = auth_sheet.get(&auth_token_rune_id);
    
    println!("🔍 Auth token available: {} tokens", available_auth_tokens);
    
    let authorize_factory_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
        version: Version::ONE,
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input: vec![TxIn {
            previous_output: auth_token_outpoint,
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
                                    free_mint_contract_id.block, free_mint_contract_id.tx, 1u128, // Call free-mint, opcode 1 (UpdateFactoryWhitelist)
                                    vault_factory_id.block, // Factory block to authorize
                                    vault_factory_id.tx,    // Factory tx to authorize
                                ]).encipher(),
                                protocol_tag: AlkaneMessageContext::protocol_tag() as u128,
                                pointer: Some(0),
                                refund: Some(0),
                                from: None,
                                burn: None,
                                edicts: vec![
                                    ProtostoneEdict {
                                        id: ProtoruneRuneId {
                                            block: free_mint_auth_token_id.block,
                                            tx: free_mint_auth_token_id.tx,
                                        },
                                        amount: available_auth_tokens,
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
    index_block(&authorize_factory_block, 4)?;
    
    println!("✅ Factory authorized using deployer's auth token");
    
    println!("\n🎉 QA ENVIRONMENT SETUP COMPLETE!");
    println!("=================================");
    println!("✅ Free-mint contract: {:?}", free_mint_contract_id);
    println!("✅ Vault factory: {:?} with QA parameters", vault_factory_id);
    println!("✅ Factory properly authorized");
    println!("✅ Ready for QA precision testing");
    
    let deposit_token_outpoint = OutPoint {
        txid: free_mint_block.txdata[0].compute_txid(),
        vout: 0,
    };
    
    Ok((free_mint_contract_id, vault_factory_id, reward_per_block, deposit_token_outpoint))
}

// Helper to create deposit tokens (same as before)
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
                                message: into_cellpack(vec![2u128, 1u128, 77u128]).encipher(), // MintTokens
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
    
    println!("✅ Created fresh deposit tokens at block {}", block_height);
    Ok(mint_block)
}

// Perform deposit with QA parameters (simplified from withdrawal_verification_test.rs)
fn perform_qa_deposit(
    mint_block: &Block, 
    vault_factory_id: &AlkaneId, 
    deposit_amount: u128, 
    block_height: u32
) -> Result<(Block, ProtoruneRuneId)> {
    let mint_outpoint = OutPoint {
        txid: mint_block.txdata[0].compute_txid(),
        vout: 0,
    };
    
    let mint_sheet = load_sheet(&RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
        .OUTPOINT_TO_RUNES.select(&consensus_encode(&mint_outpoint)?));
    let token_rune_id = ProtoruneRuneId { block: 2, tx: 1 };
    let available_tokens = mint_sheet.get(&token_rune_id);
    
    println!("\n💰 QA DEPOSIT OPERATION");
    println!("======================");
    println!("🔍 Available tokens: {}", available_tokens);
    println!("🎯 Deposit amount: {}", deposit_amount);
    
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
                                ]).encipher(),
                                protocol_tag: AlkaneMessageContext::protocol_tag() as u128,
                                pointer: Some(0),
                                refund: Some(0),
                                from: None,
                                burn: None,
                                edicts: vec![
                                    ProtostoneEdict {
                                        id: ProtoruneRuneId {
                                            block: 2,
                                            tx: 1,
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
    
    // Extract position token from deposit response
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
        .find(|(id, _amount)| id.block != 2 || id.tx != 1) // Not the deposit token
        .ok_or_else(|| anyhow::anyhow!("No position token found"))?;
    
    let position_token_id = ProtoruneRuneId {
        block: position_token_info.0.block,
        tx: position_token_info.0.tx,
    };
    
    println!("✅ QA deposit successful at block {}", block_height);
    println!("🎫 Position token: {:?}", position_token_id);
    
    Ok((deposit_block, position_token_id))
}

// Perform withdrawal with QA parameters
fn perform_qa_withdrawal(
    deposit_block: &Block,
    position_token_id: &ProtoruneRuneId,
    vault_factory_id: &AlkaneId,
    block_height: u32
) -> Result<Block> {
    let position_outpoint = OutPoint {
        txid: deposit_block.txdata[0].compute_txid(),
        vout: 0,
    };
    
    let position_sheet = load_sheet(&RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
        .OUTPOINT_TO_RUNES.select(&consensus_encode(&position_outpoint)?));
    let available_position_tokens = position_sheet.get(position_token_id);
    
    println!("\n💸 QA WITHDRAWAL OPERATION");
    println!("=========================");
    println!("🎫 Position tokens available: {}", available_position_tokens);
    
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
                                        id: position_token_id.clone(),
                                        amount: available_position_tokens,
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
    
    // Comprehensive trace analysis
    println!("\n🔍 QA WITHDRAWAL TRACE ANALYSIS");
    println!("==============================");
    
    for vout in 0..5 {
        let trace_data = &view::trace(&OutPoint {
            txid: withdrawal_block.txdata[0].compute_txid(),
            vout,
        })?;
        let trace_result: alkanes_support::trace::Trace = alkanes_support::proto::alkanes::AlkanesTrace::parse_from_bytes(trace_data)?.into();
        let trace_guard = trace_result.0.lock().unwrap();
        if !trace_guard.is_empty() {
            println!("   • QA withdrawal vout {} trace: {:?}", vout, *trace_guard);
        }
    }
    
    // Analyze withdrawal results
    let withdrawal_outpoint = OutPoint {
        txid: withdrawal_block.txdata[0].compute_txid(),
        vout: 0,
    };
    
    let withdrawal_sheet = load_sheet(
        &RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
            .OUTPOINT_TO_RUNES
            .select(&consensus_encode(&withdrawal_outpoint)?)
    );
    
    println!("\n💰 QA WITHDRAWAL RESULTS ANALYSIS");
    println!("================================");
    let mut total_received = 0u128;
    for (id, amount) in withdrawal_sheet.balances().iter() {
        println!("   • Received Token ID: {:?}, Amount: {}", id, amount);
        total_received += amount;
    }
    
    println!("✅ QA withdrawal completed at block {}", block_height);
    println!("🏆 Total tokens received: {}", total_received);
    
    Ok(withdrawal_block)
}

#[wasm_bindgen_test]
fn test_qa_precision_edge_case() -> Result<()> {
    println!("\n🚨 QA PRECISION EDGE CASE TEST");
    println!("=============================");
    println!("🎯 Testing EXACT QA parameters to replicate zero rewards issue");
    println!("   • reward_per_block: 100 (QA value, 10x smaller than normal test)");
    println!("   • end_reward_block: 30000 (QA value, 30x larger than normal test)");
    println!("   • Deposit: ~block 3000, Withdrawal: ~block 3004 (QA timing)");
    
    // PHASE 1: QA Environment Setup
    let (_free_mint_id, vault_factory_id, qa_reward_per_block, _deposit_outpoint) = 
        create_qa_environment_setup()?;
    
    println!("\n📊 QA TEST PARAMETERS CONFIRMED:");
    println!("   • QA reward per block: {} tokens", qa_reward_per_block);
    println!("   • Expected small rewards due to low rate");
    println!("   • Testing precision/rounding edge cases");
    
    // PHASE 2: QA Deposit at ~block 3000 (like QA environment)
    println!("\n💰 PHASE 2: QA Deposit Operation");
    println!("===============================");
    
    let mint_block = create_deposit_tokens(2999)?;
    
    let (deposit_block, position_token_id) = perform_qa_deposit(
        &mint_block,
        &vault_factory_id,
        100000000u128, // 100M tokens like QA
        3000 // Deposit at block 3000 (like QA ~3000)
    )?;
    
    // PHASE 3: Short time gap (like QA: 3000 → 3004)
    println!("\n⏰ PHASE 3: QA Time Gap");
    println!("=======================");
    println!("   • Deposit at block 3000");
    println!("   • Withdrawal at block 3004 (QA timing)");
    println!("   • Only 4 blocks elapsed (very short period)");
    println!("   • Expected rewards: 4 blocks × 100 tokens = 400 tokens");
    println!("   • But with precision issues, might round to 0!");
    
    // PHASE 4: QA Withdrawal at block 3004
    println!("\n💸 PHASE 4: QA Withdrawal Operation");
    println!("===================================");
    
    let withdrawal_block = perform_qa_withdrawal(
        &deposit_block,
        &position_token_id,
        &vault_factory_id,
        3004 // Withdrawal at block 3004 (like QA timing)
    )?;
    
    // PHASE 5: Detailed mathematical analysis
    println!("\n🧮 PHASE 5: QA PRECISION ANALYSIS");
    println!("=================================");
    
    let deposit_amount = 100000000u128;
    let blocks_elapsed = 3004 - 3000; // 4 blocks
    let precision = 100_000_000u128; // 1e8 precision
    
    println!("📊 QA MATHEMATICAL BREAKDOWN:");
    println!("   • Deposit amount: {} tokens", deposit_amount);
    println!("   • Blocks elapsed: {} blocks", blocks_elapsed);
    println!("   • QA reward per block: {} tokens", qa_reward_per_block);
    println!("   • Precision constant: {} (1e8)", precision);
    
    // MasterChef calculation simulation
    let theoretical_period_rewards = (blocks_elapsed as u128) * qa_reward_per_block;
    let reward_increment = theoretical_period_rewards
        .checked_mul(precision)
        .and_then(|x| x.checked_div(deposit_amount))
        .unwrap_or(0);
    
    let expected_user_rewards = deposit_amount
        .checked_mul(reward_increment)
        .and_then(|x| x.checked_div(precision))
        .unwrap_or(0);
    
    println!("\n🔍 STEP-BY-STEP CALCULATION:");
    println!("   1. Theoretical period rewards: {} blocks × {} = {}", 
             blocks_elapsed, qa_reward_per_block, theoretical_period_rewards);
    println!("   2. Reward increment: {} × {} ÷ {} = {}", 
             theoretical_period_rewards, precision, deposit_amount, reward_increment);
    println!("   3. Expected user rewards: {} × {} ÷ {} = {}", 
             deposit_amount, reward_increment, precision, expected_user_rewards);
    
    // Extract actual rewards from withdrawal results
    let withdrawal_outpoint = OutPoint {
        txid: withdrawal_block.txdata[0].compute_txid(),
        vout: 0,
    };
    
    let withdrawal_sheet = load_sheet(
        &RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
            .OUTPOINT_TO_RUNES
            .select(&consensus_encode(&withdrawal_outpoint)?)
    );
    
    let mut actual_rewards = 0u128;
    let mut principal_returned = 0u128;
    let mut total_received = 0u128;
    
    for (id, amount) in withdrawal_sheet.balances().iter() {
        total_received += amount;
        if id.block == 2 && id.tx == 1 {
            if *amount >= deposit_amount {
                principal_returned = deposit_amount;
                actual_rewards += *amount - deposit_amount;
            } else {
                actual_rewards += *amount;
            }
        } else {
            actual_rewards += *amount;
        }
    }
    
    println!("\n📊 FINAL QA RESULTS COMPARISON:");
    println!("==============================");
    println!("   • Expected rewards: {} tokens", expected_user_rewards);
    println!("   • Actual rewards: {} tokens", actual_rewards);
    println!("   • Principal returned: {} tokens", principal_returned);
    println!("   • Total received: {} tokens", total_received);
    
    // Check if this matches the QA issue
    let rewards_match = expected_user_rewards == actual_rewards;
    let zero_rewards = actual_rewards == 0;
    let principal_match = principal_returned == deposit_amount;
    
    println!("\n🔍 QA ISSUE ANALYSIS:");
    println!("====================");
    
    if zero_rewards {
        println!("🚨 QA ISSUE REPLICATED: ZERO REWARDS!");
        println!("   • User received 0 rewards despite {} blocks staked", blocks_elapsed);
        println!("   • Expected {} rewards but got 0", expected_user_rewards);
        println!("   • This matches the QA problem exactly");
        
        // Analyze the precision issue
        println!("\n🔍 PRECISION ISSUE ROOT CAUSE:");
        println!("   • reward_increment = {} (may be 0 due to rounding)", reward_increment);
        println!("   • theoretical_period_rewards = {}", theoretical_period_rewards);
        println!("   • When {} × {} ÷ {} rounds to 0", theoretical_period_rewards, precision, deposit_amount);
        println!("   • This causes acc_reward_per_share to not accumulate");
        
    } else if !rewards_match {
        println!("⚠️ PARTIAL QA ISSUE: Incorrect rewards but not zero");
        println!("   • Expected: {} tokens", expected_user_rewards);
        println!("   • Actual: {} tokens", actual_rewards);
        println!("   • Gap: {} tokens", expected_user_rewards.saturating_sub(actual_rewards));
        
    } else {
        println!("✅ NO QA ISSUE: Rewards calculated correctly");
        println!("   • Expected rewards: {} tokens", expected_user_rewards);
        println!("   • Actual rewards: {} tokens", actual_rewards);
    }
    
    if !principal_match {
        println!("❌ PRINCIPAL ISSUE: {} expected, {} received", deposit_amount, principal_returned);
    } else {
        println!("✅ PRINCIPAL RETURNED CORRECTLY: {} tokens", principal_returned);
    }
    
    // PHASE 6: Test edge case scenarios
    println!("\n🧪 PHASE 6: Edge Case Analysis");
    println!("==============================");
    
    // Test with even smaller time gaps
    println!("🔬 Testing 1-block staking period with QA parameters:");
    
    let one_block_theoretical = 1u128 * qa_reward_per_block; // 100 tokens
    let one_block_increment = one_block_theoretical
        .checked_mul(precision)
        .and_then(|x| x.checked_div(deposit_amount))
        .unwrap_or(0);
    let one_block_rewards = deposit_amount
        .checked_mul(one_block_increment)
        .and_then(|x| x.checked_div(precision))
        .unwrap_or(0);
    
    println!("   • 1 block theoretical rewards: {}", one_block_theoretical);
    println!("   • 1 block increment: {}", one_block_increment);
    println!("   • 1 block user rewards: {}", one_block_rewards);
    
    // Compare with normal test parameters
    println!("\n📊 COMPARISON WITH NORMAL TEST PARAMETERS:");
    println!("==========================================");
    
    let normal_reward_per_block = 1000u128;
    let normal_theoretical = (blocks_elapsed as u128) * normal_reward_per_block;
    let normal_increment = normal_theoretical
        .checked_mul(precision)
        .and_then(|x| x.checked_div(deposit_amount))
        .unwrap_or(0);
    let normal_rewards = deposit_amount
        .checked_mul(normal_increment)
        .and_then(|x| x.checked_div(precision))
        .unwrap_or(0);
    
    println!("   • Normal (1000/block) theoretical: {}", normal_theoretical);
    println!("   • Normal increment: {}", normal_increment);  
    println!("   • Normal user rewards: {}", normal_rewards);
    println!("   • QA vs Normal ratio: {:.3}", if normal_rewards > 0 { actual_rewards as f64 / normal_rewards as f64 } else { 0.0 });
    
    // PHASE 7: Identify the exact precision threshold
    println!("\n🎯 PHASE 7: Precision Threshold Analysis"); 
    println!("========================================");
    
    // Find minimum blocks needed for non-zero rewards with QA parameters
    let mut min_blocks_for_rewards = 0u32;
    for test_blocks in 1..=100 {
        let test_theoretical = (test_blocks as u128) * qa_reward_per_block;
        let test_increment = test_theoretical
            .checked_mul(precision)
            .and_then(|x| x.checked_div(deposit_amount))
            .unwrap_or(0);
        let test_rewards = deposit_amount
            .checked_mul(test_increment)
            .and_then(|x| x.checked_div(precision))
            .unwrap_or(0);
            
        if test_rewards > 0 {
            min_blocks_for_rewards = test_blocks;
            println!("   • Minimum blocks for non-zero rewards: {}", min_blocks_for_rewards);
            println!("   • At {} blocks: increment={}, rewards={}", test_blocks, test_increment, test_rewards);
            break;
        }
    }
    
    if min_blocks_for_rewards == 0 {
        println!("   • ⚠️ No rewards possible with current parameters (within 100 blocks)");
        println!("   • QA parameters create permanent zero-reward condition");
    } else if min_blocks_for_rewards > blocks_elapsed {
        println!("   • 🚨 QA staking period ({} blocks) below threshold ({} blocks)", 
                 blocks_elapsed, min_blocks_for_rewards);
        println!("   • This explains the zero rewards in QA!");
    }
    
    println!("\n🎊 QA PRECISION EDGE CASE TEST SUMMARY");
    println!("======================================");
    
    if zero_rewards && min_blocks_for_rewards > blocks_elapsed {
        println!("🎯 QA ISSUE ROOT CAUSE IDENTIFIED!");
        println!("   ✅ QA parameters create precision rounding issue");
        println!("   ✅ Short staking periods (4 blocks) round to zero rewards");
        println!("   ✅ Minimum {} blocks needed for non-zero rewards", min_blocks_for_rewards);
        println!("   ✅ This perfectly explains the QA zero rewards problem");
        
        println!("\n💡 RECOMMENDED SOLUTIONS:");
        println!("   1. Increase reward_per_block in QA environment");
        println!("   2. Use higher precision constant (1e12 instead of 1e8)");
        println!("   3. Implement minimum reward logic for short periods");
        println!("   4. Add precision boost for small deposit amounts");
        
    } else if zero_rewards {
        println!("🚨 QA ISSUE PARTIALLY REPLICATED");
        println!("   ⚠️ Zero rewards detected but threshold analysis inconclusive");
        println!("   ⚠️ May need deeper precision analysis");
        
    } else {
        println!("✅ QA PRECISION TEST COMPLETED");
        println!("   ✅ Rewards calculated correctly with QA parameters");
        println!("   ✅ No precision issues detected");
        println!("   ⚠️ QA issue may be due to other factors");
    }
    
    println!("\n🔍 KEY FINDINGS:");
    println!("   • QA uses 10x smaller reward rate (100 vs 1000)");
    println!("   • Short staking periods amplify precision issues");
    println!("   • MasterChef reward calculations vulnerable to rounding");
    println!("   • Precision threshold analysis identifies minimum viable periods");
    println!("   • This test framework can verify any parameter combination");
    
    Ok(())
}
