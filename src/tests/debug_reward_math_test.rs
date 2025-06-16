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

// Simple setup for debugging - exactly like multi_user_rewards_test but with detailed tracing
fn create_debug_vault_setup() -> Result<(Block, AlkaneId, u128)> {
    clear();
    
    println!("🔬 DEBUG REWARD MATH TEST SETUP");
    println!("================================");
    
    // Deploy contract templates using EXACT working pattern
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
    
    // Create free_mint token contract 
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

    // Mint tokens for vault initialization
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
    
    // Get available tokens
    let mint_outpoint = OutPoint { txid: mint_block.txdata[0].compute_txid(), vout: 0 };
    let mint_sheet = load_sheet(&RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
        .OUTPOINT_TO_RUNES.select(&consensus_encode(&mint_outpoint)?));
    let token_rune_id = ProtoruneRuneId { block: 2, tx: 1 };
    let available_tokens = mint_sheet.get(&token_rune_id);

    // Initialize vault factory - NO TEMPORAL CAP for pure math testing
    let deposit_token_id = AlkaneId { block: 2, tx: 1 };
    let reward_per_block = 1000u128; // 1000 tokens per block
    let start_block = 3u128;

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
                                    reward_per_block,
                                    start_block,
                                    999999999u128, // NO TEMPORAL CAP - huge number
                                    AlkaneId { block: 2, tx: 1 }.block,  // free_mint_contract_id 
                                    AlkaneId { block: 2, tx: 1 }.tx,
                                ]).encipher(),
                                protocol_tag: AlkaneMessageContext::protocol_tag() as u128,
                                pointer: Some(0),
                                refund: Some(0),
                                from: None,
                                burn: None,
                                edicts: vec![
                                    ProtostoneEdict {
                                        id: ProtoruneRuneId { block: deposit_token_id.block, tx: deposit_token_id.tx },
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
    index_block(&init_vault_block, 3)?;
    
    println!("✅ DEBUG SETUP COMPLETE");

    let token_id = AlkaneId { block: 2, tx: 1 };
    Ok((init_vault_block, token_id, reward_per_block))
}

// Create fresh tokens for each user with unique sequence
fn create_user_tokens(block_height: u32, user: &str) -> Result<Block> {
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
    
    println!("🪙 Created tokens for {} at block {}", user, block_height);
    Ok(mint_block)
}

// Perform deposit with EXACT amount to debug the math
fn debug_deposit(mint_block: &Block, deposit_amount: u128, user: &str, block_height: u32) -> Result<(Block, ProtoruneRuneId)> {
    let vault_factory_id = AlkaneId { block: 4, tx: 890 };
    
    let mint_outpoint = OutPoint {
        txid: mint_block.txdata[0].compute_txid(),
        vout: 0,
    };
    
    let mint_sheet = load_sheet(&RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
        .OUTPOINT_TO_RUNES.select(&consensus_encode(&mint_outpoint)?));
    let token_rune_id = ProtoruneRuneId { block: 2, tx: 1 };
    let available_tokens = mint_sheet.get(&token_rune_id);
    
    println!("\n💳 {} DEPOSIT DEBUG:", user.to_uppercase());
    println!("   📊 Deposit amount: {} tokens", deposit_amount);
    println!("   📊 Available tokens: {} tokens", available_tokens);
    println!("   📊 Block: {}", block_height);
    
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
                                    1u128           // deposit opcode
                                ]).encipher(),
                                protocol_tag: AlkaneMessageContext::protocol_tag() as u128,
                                pointer: Some(0),
                                refund: Some(0),
                                from: None,
                                burn: None,
                                edicts: vec![
                                    ProtostoneEdict {
                                        id: ProtoruneRuneId { block: 2, tx: 1 },
                                        amount: deposit_amount, // SEND EXACT AMOUNT
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
        .ok_or_else(|| anyhow::anyhow!("No position token found for {}", user))?;
    
    let position_token_id = ProtoruneRuneId {
        block: position_token_info.0.block,
        tx: position_token_info.0.tx,
    };

    println!("   ✅ {} deposited: Position token {:?}", user, position_token_id);
    
    Ok((deposit_block, position_token_id))
}

// Debug withdrawal with detailed tracing
fn debug_withdrawal(deposit_block: &Block, position_token_id: ProtoruneRuneId, user: &str, block_height: u32, deposit_amount: u128) -> Result<u128> {
    let vault_factory_id = AlkaneId { block: 4, tx: 890 };
    
    let position_outpoint = OutPoint {
        txid: deposit_block.txdata[0].compute_txid(),
        vout: 0,
    };

    println!("\n💸 {} WITHDRAWAL DEBUG:", user.to_uppercase());
    println!("   📊 Block: {}", block_height);
    println!("   📊 Original deposit: {} tokens", deposit_amount);

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
    
    println!("   💰 Withdrawal analysis:");
    for (id, amount) in withdrawal_sheet.balances().iter() {
        println!("     • Token {:?}: {} tokens", id, amount);
        if id.block == 2 && id.tx == 1 {
            total_returned += *amount;
        }
    }
    
    let rewards_returned = total_returned.saturating_sub(deposit_amount);
    
    println!("   📊 Total returned: {} tokens", total_returned);
    println!("   📊 Principal: {} tokens", deposit_amount);  
    println!("   📊 Rewards: {} tokens", rewards_returned);
    
    Ok(total_returned)
}

#[wasm_bindgen_test]
fn test_debug_reward_math_simple_2_users() -> Result<()> {
    println!("🔬 DEBUG REWARD MATH: Simple 2-User Test");
    println!("=========================================");
    
    let (_init_vault_block, _token_id, reward_per_block) = create_debug_vault_setup()?;
    
    println!("\n🎯 TEST SCENARIO:");
    println!("   • Alice deposits 1000 tokens at block 10");
    println!("   • Bob deposits 2000 tokens at block 15");  
    println!("   • Both withdraw at block 25");
    println!("   • Reward rate: {} tokens/block", reward_per_block);
    
    // User A: Deposits 1000 at block 10
    let alice_mint = create_user_tokens(4, "Alice")?;
    let (alice_deposit, alice_position) = debug_deposit(&alice_mint, 1000, "Alice", 10)?;
    
    // User B: Deposits 2000 at block 15
    let bob_mint = create_user_tokens(11, "Bob")?;
    let (bob_deposit, bob_position) = debug_deposit(&bob_mint, 2000, "Bob", 15)?;
    
    println!("\n📊 EXPECTED MATH CALCULATION:");
    println!("   • Alice alone (blocks 10-15): 5 blocks × 1000 tokens/block = 5000 rewards");
    println!("   • Alice+Bob (blocks 15-25): 10 blocks × 1000 tokens/block");
    println!("     - Alice share: 1000/(1000+2000) = 33.33% → 3333 rewards");
    println!("     - Bob share: 2000/(1000+2000) = 66.67% → 6667 rewards");
    println!("   • Alice total expected: 5000 + 3333 = 8333 rewards");
    println!("   • Bob total expected: 6667 rewards");
    
    // Both withdraw at block 25
    let alice_total = debug_withdrawal(&alice_deposit, alice_position, "Alice", 25, 1000)?;
    let bob_total = debug_withdrawal(&bob_deposit, bob_position, "Bob", 25, 2000)?;
    
    // Calculate actual rewards (subtract principal)
    let alice_rewards = alice_total.saturating_sub(1000);
    let bob_rewards = bob_total.saturating_sub(2000);
    
    println!("\n📊 RESULTS COMPARISON:");
    println!("   • Alice: Expected 8333, Got {} (diff: {})", alice_rewards, alice_rewards as i128 - 8333);
    println!("   • Bob: Expected 6667, Got {} (diff: {})", bob_rewards, bob_rewards as i128 - 6667);
    
    // Check if the math is approximately correct (within reasonable tolerance)
    let alice_diff = (alice_rewards as i128 - 8333).abs();
    let bob_diff = (bob_rewards as i128 - 6667).abs();
    let tolerance = 1000; // 1000 token tolerance
    
    if alice_diff <= tolerance && bob_diff <= tolerance {
        println!("\n🎉 REWARD MATH VERIFICATION: ✅ PASSED!");
        println!("   🏆 Both users received mathematically correct rewards");
    } else {
        println!("\n❌ REWARD MATH VERIFICATION: FAILED!");
        println!("   • Math errors detected in MasterChef calculations");
        return Err(anyhow::anyhow!("Reward math verification failed"));
    }
    
    Ok(())
}
