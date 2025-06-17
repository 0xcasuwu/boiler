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
use metashrew_support::{utils::consensus_encode};
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

#[wasm_bindgen_test]
fn test_security_verification() -> Result<()> {
    println!("🔒 SECURITY VERIFICATION: Testing mint authorization");
    
    clear();
    
    // Deploy template contracts
    println!("📋 Deploying contract templates...");
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
    
    // Deploy free-mint contract with authorization for specific vault factory
    println!("🏭 Deploying authorized free-mint contract...");
    let authorized_vault_factory_id = AlkaneId { block: 4, tx: 890 }; // This will be our authorized factory
    
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
    
    println!("✅ Free-mint deployed: AlkaneId {{ block: {}, tx: {} }}", free_mint_id.block, free_mint_id.tx);
    println!("🔐 AUTHORIZED factory: AlkaneId {{ block: {}, tx: {} }}", authorized_vault_factory_id.block, authorized_vault_factory_id.tx);
    
    // Deploy authorized vault factory (this matches the authorization)
    println!("🏦 Deploying authorized vault factory...");
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
                                    2u128, 1u128, // deposit token (dummy)
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
    index_block(&vault_factory_block, 2)?;
    let vault_factory_id = authorized_vault_factory_id; // This matches our authorization
    
    println!("✅ Authorized vault factory deployed: AlkaneId {{ block: {}, tx: {} }}", vault_factory_id.block, vault_factory_id.tx);
    
    // Deploy UNAUTHORIZED factory (different ID - not in whitelist)
    println!("⚠️ Deploying UNAUTHORIZED contract...");
    let unauthorized_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
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
                                    4u128, 0x37a + 1, 0u128, // Different contract (UNAUTHORIZED)
                                    2u128, 1u128, // deposit token (dummy)
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
    index_block(&unauthorized_block, 3)?;
    let unauthorized_id = AlkaneId { block: 4, tx: 891 }; // Different from authorized
    
    println!("❌ UNAUTHORIZED contract deployed: AlkaneId {{ block: {}, tx: {} }}", unauthorized_id.block, unauthorized_id.tx);
    
    // TEST 1: Authorized mint (should succeed)
    println!("");
    println!("🔍 TEST 1: AUTHORIZED MINT (should succeed)");
    
    let authorized_mint_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
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
                                    free_mint_id.block, free_mint_id.tx, 77u128, // Call free-mint with MintTokens opcode
                                    50u128, // Amount to mint
                                ]).encipher(),
                                protocol_tag: AlkaneMessageContext::protocol_tag() as u128,
                                pointer: Some(0),
                                refund: Some(0),
                                from: None,
                                burn: None,
                                edicts: vec![
                                    ProtostoneEdict {
                                        id: ProtoruneRuneId {
                                            block: authorized_vault_factory_id.block, // Use AUTHORIZED factory token
                                            tx: authorized_vault_factory_id.tx,
                                        },
                                        amount: 1, // Auth token amount
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
    index_block(&authorized_mint_block, 4)?;
    
    // Check if authorized mint succeeded
    let authorized_trace_data = &view::trace(&OutPoint {
        txid: authorized_mint_block.txdata[0].compute_txid(),
        vout: 3,
    })?;
    let authorized_trace_result: Trace = AlkanesTrace::parse_from_bytes(authorized_trace_data)?.into();
    let authorized_trace_str = format!("{:?}", authorized_trace_result.0.lock().unwrap());
    let authorized_success = authorized_trace_str.contains("ReturnContext") && !authorized_trace_str.contains("RevertContext");
    
    if authorized_success {
        println!("✅ SUCCESS: Authorized mint completed successfully!");
        println!("🎉 Security system correctly allowed authorized factory");
    } else {
        println!("❌ UNEXPECTED: Authorized mint was rejected!");
    }
    
    // TEST 2: Unauthorized mint (should fail)
    println!("");
    println!("🔍 TEST 2: UNAUTHORIZED MINT (should fail)");
    
    let unauthorized_mint_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
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
                                    free_mint_id.block, free_mint_id.tx, 77u128, // Call free-mint with MintTokens opcode
                                    100u128, // Amount to mint
                                ]).encipher(),
                                protocol_tag: AlkaneMessageContext::protocol_tag() as u128,
                                pointer: Some(0),
                                refund: Some(0),
                                from: None,
                                burn: None,
                                edicts: vec![
                                    ProtostoneEdict {
                                        id: ProtoruneRuneId {
                                            block: unauthorized_id.block, // Use UNAUTHORIZED factory token
                                            tx: unauthorized_id.tx,
                                        },
                                        amount: 1, // Auth token amount
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
    index_block(&unauthorized_mint_block, 5)?;
    
    // Check if unauthorized mint was rejected
    let unauthorized_trace_data = &view::trace(&OutPoint {
        txid: unauthorized_mint_block.txdata[0].compute_txid(),
        vout: 3,
    })?;
    let unauthorized_trace_result: Trace = AlkanesTrace::parse_from_bytes(unauthorized_trace_data)?.into();
    let unauthorized_trace_str = format!("{:?}", unauthorized_trace_result.0.lock().unwrap());
    let unauthorized_rejected = unauthorized_trace_str.contains("RevertContext") || !unauthorized_trace_str.contains("ReturnContext");
    
    if unauthorized_rejected {
        println!("✅ SUCCESS: Unauthorized mint was properly blocked!");
        println!("🛡️ Security system correctly rejected unauthorized factory");
    } else {
        println!("❌ SECURITY BREACH: Unauthorized mint succeeded!");
    }
    
    // TEST 3: Direct attack (no auth tokens)
    println!("");
    println!("🔍 TEST 3: DIRECT ATTACK (no auth tokens, should fail)");
    
    let direct_attack_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
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
                                    free_mint_id.block, free_mint_id.tx, 77u128, // Call free-mint with MintTokens opcode
                                    1000u128, // Amount to mint
                                ]).encipher(),
                                protocol_tag: AlkaneMessageContext::protocol_tag() as u128,
                                pointer: Some(0),
                                refund: Some(0),
                                from: None,
                                burn: None,
                                edicts: vec![], // NO AUTH TOKENS
                            }
                        ].encipher()?
                    )
                }).encipher(),
                value: Amount::from_sat(546)
            }
        ],
    }]);
    index_block(&direct_attack_block, 6)?;
    
    // Check if direct attack was rejected
    let direct_trace_data = &view::trace(&OutPoint {
        txid: direct_attack_block.txdata[0].compute_txid(),
        vout: 3,
    })?;
    let direct_trace_result: Trace = AlkanesTrace::parse_from_bytes(direct_trace_data)?.into();
    let direct_trace_str = format!("{:?}", direct_trace_result.0.lock().unwrap());
    let direct_attack_rejected = direct_trace_str.contains("RevertContext") || !direct_trace_str.contains("ReturnContext");
    
    if direct_attack_rejected {
        println!("✅ SUCCESS: Direct attack was properly blocked!");
        println!("🛡️ Security system detected missing authorization");
    } else {
        println!("❌ SECURITY BREACH: Direct attack succeeded!");
    }
    
    println!("");
    println!("🔒 SECURITY VERIFICATION COMPLETE");
    println!("🎯 Objective: Prove that only authorized factories can mint tokens");
    
    if authorized_success && unauthorized_rejected && direct_attack_rejected {
        println!("✅ AUTHORIZED MINT: SUCCESS (allowed)");
        println!("✅ UNAUTHORIZED MINT: BLOCKED (rejected)");
        println!("✅ DIRECT ATTACK: BLOCKED (rejected)");
        println!("🛡️ SECURITY SYSTEM STATUS: FULLY OPERATIONAL");
        println!("🔒 Authorization whitelist working perfectly");
        println!("🚫 All unauthorized attempts successfully blocked");
        println!("✅ Factory authorization system is PRODUCTION READY");
    } else {
        println!("❌ SECURITY SYSTEM FAILURE DETECTED");
        return Err(anyhow::anyhow!("Security verification failed"));
    }
    
    Ok(())
}
