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

// EXACT copy of the working setup from long_term_determinism_test.rs
fn create_working_vault_setup() -> Result<(AlkaneId, AlkaneId, AlkaneId, OutPoint)> {
    clear();
    
    // Deploy contract templates (EXACT copy)
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
    
    // Create free_mint with authorization system (EXACT copy)
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
    
    // Create deposit token supply (EXACT copy)
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
    
    // Initialize vault (EXACT copy)  
    let end_reward_block = 500u128; // Extended testing period
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
                                    end_reward_block, // end reward block (EXTENDED for testing)
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

// Create deposit tokens for a user (EXACT copy)
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

// Simplified deposit function (EXACT copy)
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

// Simplified withdrawal function (EXACT copy)
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
fn test_fixed_reward_math_2_users() -> Result<()> {
    println!("🔬 FIXED DEBUG: 2-User Reward Math Test");
    println!("======================================");
    
    let (_free_mint_id, _deposit_token_id, vault_factory_id, _vault_auth_outpoint) = create_working_vault_setup()?;
    
    println!("🎯 TEST SCENARIO (Using Working Pattern):");
    println!("   • Alice deposits 1000 tokens at block 10");
    println!("   • Bob deposits 2000 tokens at block 15");  
    println!("   • Both withdraw at block 25");
    println!("   • Reward rate: 10 tokens/block (same as working test)");
    
    // User A: Deposits 1000 at block 10
    let alice_tokens = create_user_tokens(5, 1000)?;
    let (alice_deposit, alice_position) = perform_deposit(&alice_tokens, 1000, vault_factory_id, 10)?;
    
    // User B: Deposits 2000 at block 15
    let bob_tokens = create_user_tokens(11, 2000)?;
    let (bob_deposit, bob_position) = perform_deposit(&bob_tokens, 2000, vault_factory_id, 15)?;
    
    println!("\n📊 EXPECTED MATH CALCULATION:");
    println!("   • Alice alone (blocks 10-15): 5 blocks × 10 tokens/block = 50 rewards");
    println!("   • Alice+Bob (blocks 15-25): 10 blocks × 10 tokens/block");
    println!("     - Alice share: 1000/(1000+2000) = 33.33% → 33 rewards");
    println!("     - Bob share: 2000/(1000+2000) = 66.67% → 67 rewards");
    println!("   • Alice total expected: 50 + 33 = 83 rewards");
    println!("   • Bob total expected: 67 rewards");
    
    // Both withdraw at block 25
    let alice_total = perform_withdrawal(&alice_deposit, alice_position, vault_factory_id, 25)?;
    let bob_total = perform_withdrawal(&bob_deposit, bob_position, vault_factory_id, 25)?;
    
    // Calculate actual rewards (subtract principal)
    let alice_rewards = alice_total.saturating_sub(1000);
    let bob_rewards = bob_total.saturating_sub(2000);
    
    println!("\n📊 RESULTS COMPARISON:");
    println!("   • Alice: Expected 83, Got {} (diff: {})", alice_rewards, alice_rewards as i128 - 83);
    println!("   • Bob: Expected 67, Got {} (diff: {})", bob_rewards, bob_rewards as i128 - 67);
    println!("   • Alice principal: Expected 1000, Got {}", alice_total.saturating_sub(alice_rewards));
    println!("   • Bob principal: Expected 2000, Got {}", bob_total.saturating_sub(bob_rewards));
    
    // Principal safety check
    let alice_principal_returned = alice_total >= 1000;
    let bob_principal_returned = bob_total >= 2000;
    
    println!("\n🛡️ PRINCIPAL SAFETY:");
    println!("   • Alice principal safe: {}", if alice_principal_returned { "✅" } else { "❌ CRITICAL BUG" });
    println!("   • Bob principal safe: {}", if bob_principal_returned { "✅" } else { "❌ CRITICAL BUG" });
    
    // Check if the math is approximately correct (within reasonable tolerance)
    let alice_diff = (alice_rewards as i128 - 83).abs();
    let bob_diff = (bob_rewards as i128 - 67).abs();
    let tolerance = 10; // 10 token tolerance
    
    if alice_diff <= tolerance && bob_diff <= tolerance && alice_principal_returned && bob_principal_returned {
        println!("\n🎉 REWARD MATH VERIFICATION: ✅ PASSED!");
        println!("   🏆 Both users received mathematically correct rewards");
        println!("   🛡️ Principal safety maintained");
    } else {
        println!("\n❌ REWARD MATH VERIFICATION: FAILED!");
        if !alice_principal_returned || !bob_principal_returned {
            println!("   🚨 CRITICAL: Principal not fully returned!");
        }
        return Err(anyhow::anyhow!("Reward math verification failed"));
    }
    
    Ok(())
}
