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
use protorune::message::MessageContext;
use bitcoin::{transaction::Version, ScriptBuf, Sequence};
use bitcoin::{Address, Amount, Block, Transaction, TxIn, TxOut, Witness};
use ordinals::Runestone;
use protorune::test_helpers::{get_btc_network, ADDRESS1};
use protorune::{test_helpers as protorune_helpers};
use protorune_support::{balance_sheet::ProtoruneRuneId, protostone::{Protostone, ProtostoneEdict}};
use protorune::protostone::Protostones;
use protobuf::Message;
use alkanes_support::trace::Trace;
use alkanes_support::proto::alkanes::AlkanesTrace;
use metashrew_core::{println, stdio::stdout};
    
use crate::precompiled::free_mint_build;
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

fn setup_test_environment() -> Result<(AlkaneId, AlkaneId, AlkaneId)> {
    clear();
    
    // Deploy template contracts
    let template_block = alkane_helpers::init_with_multiple_cellpacks_with_tx(
        [
            free_mint_build::get_bytes(),
            alk4626_vault_factory_build::get_bytes(),
        ].into(),
        [
            vec![3u128, 797u128, 101u128],
            vec![3u128, 0x37a, 10u128],
        ].into_iter().map(|v| into_cellpack(v)).collect::<Vec<Cellpack>>()
    );
    index_block(&template_block, 0)?;
    
    // Deploy free-mint contract with authorization
    let authorized_vault_factory_id = AlkaneId { block: 4, tx: 890 };
    
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
                                    authorized_vault_factory_id.block, // vault_factory_block (AUTHORIZED)
                                    authorized_vault_factory_id.tx,    // vault_factory_tx (AUTHORIZED)
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
    
    // Deploy deposit token
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
                                message: into_cellpack(vec![
                                    6u128, 797u128, 101u128, 1000u128
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
    index_block(&deposit_token_block, 2)?;
    let deposit_token_id = AlkaneId { block: 2, tx: 1 };
    
    // Deploy authorized vault factory
    let vault_factory_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
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
                                    10u128, // reward per block
                                    3u128, // start_block
                                    1000u128, // end reward block
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
    index_block(&vault_factory_block, 3)?;
    let vault_factory_id = authorized_vault_factory_id;
    
    Ok((free_mint_id, deposit_token_id, vault_factory_id))
}

#[wasm_bindgen_test]
fn test_direct_attack_prevention() -> Result<()> {
    println!("🛡️ SECURITY TEST: Direct Attack Prevention - Enhanced Trace Analysis");
    
    let (free_mint_id, _deposit_token_id, vault_factory_id) = setup_test_environment()?;
    
    // TEST 1: Opcode 77 (MintTokens) - Public Free Mint (SHOULD WORK)
    println!("🔍 TEST 1: Opcode 77 (MintTokens) - Public Free Mint");
    println!("   🎯 Attack Vector: Direct opcode 77 call (public free mint)");
    println!("   ✅ Expected Result: Should SUCCEED (this is intentionally public)");
    
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
                                    free_mint_id.block, free_mint_id.tx, 77u128, // Opcode 77 - Public free mint
                                ]).encipher(),
                                protocol_tag: AlkaneMessageContext::protocol_tag() as u128,
                                pointer: Some(0),
                                refund: Some(0),
                                from: None,
                                burn: None,
                                edicts: vec![], // No auth tokens needed for opcode 77
                            }
                        ].encipher()?
                    )
                }).encipher(),
                value: Amount::from_sat(546)
            }
        ],
    }]);
    index_block(&free_mint_block, 4)?;
    
    let free_mint_trace = &view::trace(&OutPoint {
        txid: free_mint_block.txdata[0].compute_txid(),
        vout: 3,
    })?;
    let free_mint_result: Trace = AlkanesTrace::parse_from_bytes(free_mint_trace)?.into();
    let free_mint_succeeded = format!("{:?}", free_mint_result.0.lock().unwrap()).contains("AlkaneTransfer");
    
    println!("   ✅ Opcode 77 (MintTokens) succeeded: {} (expected: true)", free_mint_succeeded);
    assert!(free_mint_succeeded, "Opcode 77 should succeed - it's public free mint");
    
    // TEST 2: Opcode 78 (FactoryMintTokens) - Unauthorized Attack (SHOULD FAIL)
    println!("🔍 TEST 2: Opcode 78 (FactoryMintTokens) - Unauthorized Attack");
    println!("   🎯 Attack Vector: Direct opcode 78 call without authorization tokens");
    println!("   🔒 Expected Result: Should be BLOCKED by authorization system");
    
    let attack_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
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
                                    free_mint_id.block, free_mint_id.tx, 78u128, // Opcode 78 - Requires authorization!
                                    1000000u128, // Large amount attempt
                                ]).encipher(),
                                protocol_tag: AlkaneMessageContext::protocol_tag() as u128,
                                pointer: Some(0),
                                refund: Some(0),
                                from: None,
                                burn: None,
                                edicts: vec![], // NO AUTHORIZATION TOKENS - THIS IS THE REAL ATTACK
                            }
                        ].encipher()?
                    )
                }).encipher(),
                value: Amount::from_sat(546)
            }
        ],
    }]);
    index_block(&attack_block, 5)?;
    
    println!("🔍 TRACE: Unauthorized opcode 78 attack indexed at block 5");
    
    let attack_trace_data = &view::trace(&OutPoint {
        txid: attack_block.txdata[0].compute_txid(),
        vout: 3,
    })?;
    let attack_trace_result: Trace = AlkanesTrace::parse_from_bytes(attack_trace_data)?.into();
    
    // DETAILED TRACE ANALYSIS
    println!("=== OPCODE 78 SECURITY TRACE ANALYSIS ===");
    let trace_guard = attack_trace_result.0.lock().unwrap();
    println!("Opcode 78 attack trace: {:?}", *trace_guard);
    
    let has_enter_call = format!("{:?}", *trace_guard).contains("EnterCall");
    let has_return_context = format!("{:?}", *trace_guard).contains("ReturnContext");
    let has_revert_context = format!("{:?}", *trace_guard).contains("RevertContext");
    let has_alkane_transfer = format!("{:?}", *trace_guard).contains("AlkaneTransfer");
    let contains_unauthorized = format!("{:?}", *trace_guard).contains("Unauthorized");
    
    println!("📊 DETAILED TRACE BREAKDOWN:");
    println!("   • EnterCall detected: {}", has_enter_call);
    println!("   • ReturnContext detected: {}", has_return_context);
    println!("   • RevertContext detected: {}", has_revert_context);
    println!("   • AlkaneTransfer detected: {}", has_alkane_transfer);
    println!("   • 'Unauthorized' error detected: {}", contains_unauthorized);
    
    // Security Analysis - Opcode 78 should be blocked without authorization
    let attack_was_blocked = has_revert_context || contains_unauthorized || !has_alkane_transfer;
    
    println!("🔒 OPCODE 78 SECURITY VERDICT:");
    if attack_was_blocked {
        println!("   ✅ ATTACK BLOCKED: Authorization system working correctly");
        println!("   🛡️ Opcode 78 properly requires authorization tokens");
    } else {
        println!("   ❌ ATTACK SUCCEEDED: Security breach detected!");
        println!("   🚨 CRITICAL: Unauthorized opcode 78 call was not blocked");
        if has_alkane_transfer {
            println!("   🚨 TOKENS WERE MINTED WITHOUT AUTHORIZATION!");
        }
    }
    
    // Assert that opcode 78 attack should be blocked
    assert!(attack_was_blocked, 
        "❌ SECURITY FAILURE: Opcode 78 unauthorized attack was not blocked! \
         This means the free-mint contract allowed opcode 78 minting without proper authorization tokens. \
         Expected: RevertContext, 'Unauthorized' error, or no successful token minting. \
         Got: Successful token minting without authorization.");
    
    println!("🎉 SECURITY TEST PASSED: Direct attack prevention working correctly!");
    Ok(())
}
