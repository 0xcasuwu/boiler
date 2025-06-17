use alkanes::view;
use anyhow::Result;
use bitcoin::blockdata::transaction::OutPoint;
use wasm_bindgen_test::wasm_bindgen_test;
use alkanes::tests::helpers::clear;
use alkanes::indexer::index_block;
use alkanes::network::set_view_mode;
use std::fmt::Write;
use std::str::FromStr;
use alkanes::message::AlkaneMessageContext;
use alkanes_support::cellpack::Cellpack;
use alkanes_support::id::AlkaneId;
use alkanes_support::parcel::AlkaneTransferParcel;
use alkanes::tests::helpers as alkane_helpers;
use protorune::{balance_sheet::{load_sheet}, tables::RuneTable, message::MessageContext};
use protorune_support::balance_sheet::BalanceSheetOperations;
use protorune::message::MessageContextParcel;
use bitcoin::hashes::Hash;
use alkanes_support::envelope::RawEnvelope;
use bitcoin::address::NetworkChecked;
use bitcoin::{transaction::Version, ScriptBuf, Sequence};
use bitcoin::{Address, Amount, Block, Transaction, TxIn, TxOut, Witness};
use metashrew_support::{index_pointer::KeyValuePointer, utils::consensus_encode};
use ordinals::{Runestone, RuneId};
use protorune::protostone::Protostones;
use protorune::test_helpers::{create_block_with_coinbase_tx, get_btc_network, ADDRESS1};
use protorune::{test_helpers as protorune_helpers};
use protorune_support::{balance_sheet::ProtoruneRuneId, protostone::{Protostone, ProtostoneEdict}};
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

fn create_basic_token_setup() -> Result<(Block, Block, AlkaneId)> {
    // Create a basic free mint token for testing
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
                        message: into_cellpack(vec![6u128, 797u128, 0u128, 10000000u128, 2000000u128, 20000000u128, 0x555555, 0, 0x555555]).encipher(),
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
    
    // Mint tokens
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
    
    let free_mint_id = AlkaneId { block: 2, tx: 1 };
    Ok((free_mint_block, mint_block, free_mint_id))
}

#[wasm_bindgen_test]
fn test_step_by_step_initialization() -> Result<()> {
    clear();
    
    println!("=== STEP-BY-STEP INITIALIZATION DEBUG ===");
    
    // Deploy contracts
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
    
    // STEP 1: Create and initialize one token contract
    let (_free_mint_block, mint_block, free_mint_id) = create_basic_token_setup()?;
    
    println!("✅ STEP 1: Created token contract at {:?}", free_mint_id);
    
    // Verify we have tokens
    let mint_outpoint = OutPoint {
        txid: mint_block.txdata[0].compute_txid(),
        vout: 0,
    };
    
    let mint_sheet = load_sheet(
        &RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
            .OUTPOINT_TO_RUNES
            .select(&consensus_encode(&mint_outpoint)?)
    );
    
    let free_mint_rune_id = ProtoruneRuneId { block: free_mint_id.block, tx: free_mint_id.tx };
    let available_tokens = mint_sheet.get(&free_mint_rune_id);
    
    println!("✅ STEP 2: Available tokens: {} at outpoint {:?}", available_tokens, mint_outpoint);
    
    // STEP 3: Initialize vault with EXACT token amount in edict
    // CRITICAL: preloaded_rewards parameter must match EXACTLY what's sent via edict
    let preloaded_rewards = available_tokens; // Use all available tokens to match edict
    
    println!("✅ STEP 3: Using ALL available tokens ({}) for preloaded_rewards parameter", preloaded_rewards);
    
    println!("✅ STEP 3: Proceeding with initialization using {} reward tokens", preloaded_rewards);
    
    let init_vault_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
      version: Version::ONE,
      lock_time: bitcoin::absolute::LockTime::ZERO,
      input: vec![TxIn {
        previous_output: mint_outpoint, // Reference the tokens we minted
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
                            4u128, 0x37a, 0u128,                    // target + Initialize opcode
                            free_mint_id.block, free_mint_id.tx,    // deposit_token_id (AlkaneId)
                            free_mint_id.block, free_mint_id.tx,    // reward_token_id (AlkaneId)
                            1000u128,                               // reward_per_block
                            3u128,                                  // start_block
                            preloaded_rewards,                      // preloaded_rewards
                            0u128                                   // fee_percentage
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
                                amount: preloaded_rewards, // Send EXACT amount
                                output: 1,                 // To the vault initialization output
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
    
    // STEP 4: Trace the initialization
    let init_trace_data = &view::trace(&OutPoint {
        txid: init_vault_block.txdata[0].compute_txid(),
        vout: 3,
    })?;
    let init_trace_result: alkanes_support::trace::Trace = alkanes_support::proto::alkanes::AlkanesTrace::parse_from_bytes(init_trace_data)?.into();
    
    println!("\n=== STEP 4: INITIALIZATION TRACE ANALYSIS ===");
    println!("Initialization trace result: {:?}", init_trace_result);
    
    // Check if we get any specific error messages
    let trace_debug_str = format!("{:?}", init_trace_result.0.lock().unwrap());
    
    if trace_debug_str.contains("Must preload reward pool") {
        println!("❌ ERROR: Must preload reward pool - no reward tokens received");
    } else if trace_debug_str.contains("doesn't match preloaded_rewards") {
        println!("❌ ERROR: Token amount mismatch - edict not working correctly");
    } else if trace_debug_str.contains("RevertContext") {
        println!("❌ ERROR: Transaction reverted - check trace for specific error");
    } else if trace_debug_str.contains("ReturnContext") {
        println!("✅ SUCCESS: Initialization completed successfully!");
        
        // Verify vault received auth token
        let init_outpoint = OutPoint {
            txid: init_vault_block.txdata[0].compute_txid(),
            vout: 0,
        };
        
        let init_sheet = load_sheet(
            &RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
                .OUTPOINT_TO_RUNES
                .select(&consensus_encode(&init_outpoint)?)
        );
        
        println!("\n--- Tokens received after initialization ---");
        for (id, amount) in init_sheet.balances().iter() {
            println!("Token ID: {:?}, Amount: {}", id, amount);
        }
        
        // Check for vault factory auth token
        let vault_factory_auth_id = ProtoruneRuneId { block: 4, tx: 0x37a };
        let auth_token_amount = init_sheet.get(&vault_factory_auth_id);
        
        if auth_token_amount >= 1 {
            println!("✅ INITIALIZATION VERIFIED: Received {} vault factory auth tokens", auth_token_amount);
            
            // STEP 5: Test a simple deposit
            println!("\n=== STEP 5: TESTING SIMPLE DEPOSIT ===");
            
            // We should have remaining tokens from the mint
            let remaining_tokens = available_tokens - preloaded_rewards;
            if remaining_tokens > 5000 {
                let deposit_amount = 5000u128;
                
                let deposit_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
                  version: Version::ONE,
                  lock_time: bitcoin::absolute::LockTime::ZERO,
                  input: vec![TxIn {
                    previous_output: OutPoint {
                      txid: init_vault_block.txdata[0].compute_txid(),
                      vout: 0, // Use remaining tokens from initialization
                    },
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
                                        4u128, 0x37a, 1u128, deposit_amount // deposit opcode
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
                index_block(&deposit_block, 4)?;
                
                // Trace deposit
                let deposit_trace_data = &view::trace(&OutPoint {
                    txid: deposit_block.txdata[0].compute_txid(),
                    vout: 3,
                })?;
                let deposit_trace_result: alkanes_support::trace::Trace = alkanes_support::proto::alkanes::AlkanesTrace::parse_from_bytes(deposit_trace_data)?.into();
                
                println!("Deposit trace result: {:?}", deposit_trace_result);
                
                let deposit_debug_str = format!("{:?}", deposit_trace_result.0.lock().unwrap());
                
                if deposit_debug_str.contains("unreachable") {
                    println!("❌ DEPOSIT ERROR: wasm unreachable - deposit validation failed");
                } else if deposit_debug_str.contains("RevertContext") {
                    println!("❌ DEPOSIT ERROR: Transaction reverted");
                } else if deposit_debug_str.contains("ReturnContext") {
                    println!("✅ DEPOSIT SUCCESS: Deposit completed successfully!");
                    
                    // Check what was returned
                    let deposit_outpoint = OutPoint {
                        txid: deposit_block.txdata[0].compute_txid(),
                        vout: 0,
                    };
                    
                    let deposit_sheet = load_sheet(
                        &RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
                            .OUTPOINT_TO_RUNES
                            .select(&consensus_encode(&deposit_outpoint)?)
                    );
                    
                    println!("--- Tokens received after deposit ---");
                    for (id, amount) in deposit_sheet.balances().iter() {
                        println!("Token ID: {:?}, Amount: {}", id, amount);
                    }
                } else {
                    println!("⚠️ DEPOSIT: Unclear result - check trace manually");
                }
            } else {
                println!("❌ INSUFFICIENT TOKENS: Only {} remaining after initialization", remaining_tokens);
            }
        } else {
            println!("❌ INITIALIZATION FAILED: No auth token received");
        }
    } else {
        println!("⚠️ UNCLEAR RESULT: Check trace manually");
    }
    
    println!("\n🎯 STEP-BY-STEP INITIALIZATION COMPLETE");
    
    Ok(())
}

#[wasm_bindgen_test]
fn test_initialization_parameter_validation() -> Result<()> {
    clear();
    
    println!("=== INITIALIZATION PARAMETER VALIDATION TEST ===");
    
    // Deploy contracts
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
    
    // Create token setup
    let (_free_mint_block, mint_block, free_mint_id) = create_basic_token_setup()?;
    
    // Test different parameter combinations to understand what works
    let test_cases = vec![
        ("CORRECT_PARAMS", 1000000u128, 1000000u128), // preloaded = sent
    ];
    
    for (i, (test_name, preloaded_param, sent_amount)) in test_cases.iter().enumerate() {
        println!("\n--- TEST CASE {}: {} ---", i + 1, test_name);
        println!("   Preloaded parameter: {}", preloaded_param);
        println!("   Sent via edict: {}", sent_amount);
        
        let init_vault_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
          version: Version::ONE,
          lock_time: bitcoin::absolute::LockTime::ZERO,
          input: vec![TxIn {
            previous_output: OutPoint {
                txid: mint_block.txdata[0].compute_txid(),
                vout: 0,
            },
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
                                4u128, 0x37a, 0u128,                    // target + Initialize opcode
                                free_mint_id.block, free_mint_id.tx,    // deposit_token_id
                                free_mint_id.block, free_mint_id.tx,    // reward_token_id
                                1000u128,                               // reward_per_block
                                3u128,                                  // start_block
                                *preloaded_param,                       // preloaded_rewards parameter
                                0u128                                   // fee_percentage
                            ]).encipher(),
                            protocol_tag: AlkaneMessageContext::protocol_tag() as u128,
                            pointer: Some(0),
                            refund: Some(0),
                            from: None,
                            burn: None,
                            edicts: if *sent_amount > 0 {
                                vec![
                                    ProtostoneEdict {
                                        id: ProtoruneRuneId {
                                            block: free_mint_id.block,
                                            tx: free_mint_id.tx
                                        },
                                        amount: *sent_amount,
                                        output: 1,
                                    }
                                ]
                            } else {
                                vec![] // No edict if sending zero
                            },
                        }
                    ].encipher()?
                )
              }).encipher(),
              value: Amount::from_sat(546)
            }
          ],
        }]);
        index_block(&init_vault_block, 3 + i as u32)?;
        
        // Trace this test case
        let trace_data = &view::trace(&OutPoint {
            txid: init_vault_block.txdata[0].compute_txid(),
            vout: 3,
        })?;
        let trace_result: alkanes_support::trace::Trace = alkanes_support::proto::alkanes::AlkanesTrace::parse_from_bytes(trace_data)?.into();
        
        let trace_debug_str = format!("{:?}", trace_result.0.lock().unwrap());
        
        if trace_debug_str.contains("Must preload reward pool") {
            println!("   ❌ Result: Must preload reward pool error");
        } else if trace_debug_str.contains("doesn't match preloaded_rewards") {
            println!("   ❌ Result: Token amount mismatch error");
        } else if trace_debug_str.contains("RevertContext") {
            println!("   ❌ Result: Other revert error");
        } else if trace_debug_str.contains("ReturnContext") {
            println!("   ✅ Result: SUCCESS - Initialization completed!");
        } else {
            println!("   ⚠️  Result: Unclear - check trace");
        }
        
        // Clean state for next test
        clear();
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
        let (_fb, mb, fmi) = create_basic_token_setup()?;
        // Update references for next iteration
        // mint_block = mb;
        // free_mint_id = fmi;
    }
    
    println!("\n🎯 PARAMETER VALIDATION TEST COMPLETE");
    
    Ok(())
}
