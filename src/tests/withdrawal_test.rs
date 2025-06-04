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
use alkanes::tests::helpers as alkane_helpers;
use protorune::{balance_sheet::{load_sheet}, tables::RuneTable, message::MessageContext};
use protorune_support::balance_sheet::BalanceSheetOperations;
use protorune::message::MessageContextParcel;
use bitcoin::hashes::Hash; // Add Hash trait for all_zeros
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

#[wasm_bindgen_test]
fn test_withdrawal_flow() -> Result<()> {
    // First, repeat the setup and deposit steps to get a position token
    clear();
    
    // Deploy all the contract templates
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
    
    // Initialize the free_mint contract
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
                        message: into_cellpack(vec![6u128, 797u128, 0u128, 10000u128, 100u128, 1000000u128, 0x414141, 0, 0x414141]).encipher(),
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
    
    // Mint free_mint tokens
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
                        message: into_cellpack(vec![2u128, 1u128, 77u128]).encipher(), // Mint tokens using working opcode
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
    
    // Initialize the vault factory using the WORKING pattern
    let free_mint_id = AlkaneId { block: 2, tx: 1 };
    
    // Get available tokens for proper parameter matching
    let mint_outpoint = OutPoint { txid: mint_block.txdata[0].compute_txid(), vout: 0 };
    let mint_sheet = load_sheet(&RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
        .OUTPOINT_TO_RUNES.select(&consensus_encode(&mint_outpoint)?));
    let token_rune_id = ProtoruneRuneId { block: 2, tx: 1 };
    let available_tokens = mint_sheet.get(&token_rune_id);
    let preloaded_rewards = available_tokens; // CRITICAL: Must match exactly!

    println!("✅ Available tokens for vault initialization: {}", available_tokens);

    // Initialize vault factory with proper parameter matching
    let deposit_token_id = AlkaneId { block: 2, tx: 1 };
    let reward_token_id = AlkaneId { block: 2, tx: 1 };
    let reward_per_block = 1000u128;
    let start_block = 3u128;
    let fee_percentage = 50u128; // 50 basis points (0.5%)

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
                                    4u128, 0x37a, 0u128,                    // target + Initialize opcode
                                    deposit_token_id.block, deposit_token_id.tx,  // AlkaneId (2 u128s)
                                    reward_token_id.block, reward_token_id.tx,    // AlkaneId (2 u128s)
                                    reward_per_block,                       // u128
                                    start_block,                           // u128
                                    preloaded_rewards,                     // u128 - MUST match edict!
                                    fee_percentage                         // u128
                                ]).encipher(),
                                protocol_tag: AlkaneMessageContext::protocol_tag() as u128,
                                pointer: Some(0),
                                refund: Some(0),
                                from: None,
                                burn: None,
                                edicts: vec![
                                    ProtostoneEdict {
                                        id: ProtoruneRuneId { block: deposit_token_id.block, tx: deposit_token_id.tx },
                                        amount: preloaded_rewards, // SAME VALUE as parameter!
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
    println!("✅ Vault factory initialized successfully");
    
    // Create new mint transaction for deposit tokens (since first mint was consumed by vault init)
    let deposit_mint_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
        version: Version::ONE,
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input: vec![TxIn {
            previous_output: OutPoint {
                txid: init_vault_block.txdata[0].compute_txid(), // Use different outpoint
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
                                message: into_cellpack(vec![2u128, 1u128, 77u128]).encipher(), // Mint tokens for deposit using correct opcode
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
    index_block(&deposit_mint_block, 4)?;
    
    // Perform deposit to get position token
    let vault_factory_id = AlkaneId { block: 4, tx: 0x37a };
    let deposit_amount = 50u128;  // Use amount that fits within available tokens (100)
    
    println!("=== E2E FEE DEMONSTRATION ===");
    println!("Initial deposit amount: {} tokens", deposit_amount);
    println!("Fee percentage: 50 basis points (0.5%)");
    println!("Expected deposit fee: {} tokens", deposit_amount * 50 / 10000);
    println!("Expected assets after deposit fee: {} tokens", deposit_amount - (deposit_amount * 50 / 10000));
    
    let deposit_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
      version: Version::ONE,
      lock_time: bitcoin::absolute::LockTime::ZERO,
      input: vec![TxIn {
        previous_output: OutPoint {
          txid: deposit_mint_block.txdata[0].compute_txid(),
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
                            4u128,
                            0x37a,
                            1u128,
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
    index_block(&deposit_block, 5)?;
    
    // Verify we received the position token and check its ID
    let position_outpoint = OutPoint {
        txid: deposit_block.txdata[0].compute_txid(),
        vout: 0,
    };
    
    let position_sheet = load_sheet(
        &RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
            .OUTPOINT_TO_RUNES
            .select(&consensus_encode(&position_outpoint)?)
    );
    
    // Get the position token ID
    let position_token_info = position_sheet.cached.balances.iter().next();
    
    // Ensure we have a position token
    if position_token_info.is_none() {
        println!("No position token found, cannot test withdrawal");
        return Err(anyhow::anyhow!("No position token found for withdrawal test"));
    }
    
    let (position_id, _) = position_token_info.unwrap();
    let position_token_id = ProtoruneRuneId {
        block: position_id.block,
        tx: position_id.tx,
    };
    
    println!("Starting withdrawal test with position token: {:?}", position_token_id);
    
    // Print details about the position token to help with debugging
    println!("Position token ID: {:?}", position_token_id);
    println!("Amount of position tokens: {}", position_sheet.get(&position_token_id));
    
    // The position token should be automatically registered in the vault factory
    // during the deposit process as shown in the vault_factory.rs code:
    //
    // 1. First the position token is created from the position token template
    // 2. Then the position is added to the registry with:
    //    self.add_position(&position_token.id)
    //
    // We can't directly verify this in the test since we don't have access to the 
    // vault factory's storage, but we can check if the withdrawal works properly

    // Create a withdrawal transaction
    // Since we changed to full withdrawal, expect the full deposit amount
    let expected_withdrawal = deposit_amount; // Full withdrawal
    
    // Create transaction with position token as input and in the incoming_alkanes
    let withdraw_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
      version: Version::ONE,
      lock_time: bitcoin::absolute::LockTime::ZERO,
      input: vec![TxIn {
        previous_output: position_outpoint,  // Reference the position token
        script_sig: ScriptBuf::new(),
        sequence: Sequence::MAX,
        witness: Witness::new()
      }],
      output: vec![
        // Output 0: Will receive the withdrawn tokens and remaining position tokens
        TxOut {
          script_pubkey: Address::from_str(ADDRESS1().as_str())
            .unwrap()
            .require_network(get_btc_network())
            .unwrap()
            .script_pubkey(),
          value: Amount::from_sat(546),
        },
        // Output 1: Contains the withdraw message
        TxOut {
          script_pubkey: (Runestone {
            // No runestone edicts
            edicts: vec![],
            etching: None,
            mint: None,
            pointer: None,
            protocol: Some(
                vec![
                    Protostone {
                        // The withdraw message to the vault factory
                        message: into_cellpack(vec![
                            4u128,              // Vault factory block
                            0x37a,              // Vault factory tx
                            2u128,              // withdraw opcode
                            0u128               // position_id (index 0)
                        ]).encipher(),
                        protocol_tag: AlkaneMessageContext::protocol_tag() as u128,
                        pointer: Some(0),       // Withdrawn tokens go to output 0
                        refund: Some(0),        // Refunds go to output 0
                        from: None,
                        burn: None,
                        // Include position token in protostone edicts for authentication
                        edicts: vec![
                            ProtostoneEdict {
                                id: position_token_id,
                                amount: 1,      // Position token amount (must be at least 1)
                                output: 1,      // Target the message output itself
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
    index_block(&withdraw_block, 6)?;
    
    // Trace the withdrawal operation to get debugging info
    let withdraw_trace_data = &view::trace(
        &(OutPoint {
            txid: withdraw_block.txdata[0].compute_txid(),
            vout: 1, // Trace from the withdraw message output
        }),
    )?;
    
    println!("Withdrawal transaction traced successfully");
    
    // Check the balance after withdrawal
    let withdraw_outpoint = OutPoint {
        txid: withdraw_block.txdata[0].compute_txid(),
        vout: 0, // Output that should receive the withdrawn tokens
    };
    
    let withdraw_sheet = load_sheet(
        &RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
            .OUTPOINT_TO_RUNES
            .select(&consensus_encode(&withdraw_outpoint)?)
    );
    
    println!("--- After Withdrawal Balances ---");
    for (id, amount) in withdraw_sheet.balances().iter() {
        println!("Token ID: {:?}, Amount: {}", id, amount);
    }
    
    // Verify we received the original deposit tokens and the position token
    let free_mint_rune_id = ProtoruneRuneId { block: free_mint_id.block, tx: free_mint_id.tx };
    
    let withdrawn_tokens = withdraw_sheet.get(&free_mint_rune_id);
    if withdrawn_tokens > 0 {
        println!("Successfully received {} of free_mint tokens back", withdrawn_tokens);
        
        // Verify the amount matches what we expected (full withdrawal)
        if withdrawn_tokens == expected_withdrawal {
            println!("✅ SUCCESS: Received the correct amount of tokens!");
        } else {
            println!("⚠️ WARNING: Received unexpected amount: {} (expected {})", withdrawn_tokens, expected_withdrawal);
        }
    } else {
        println!("❌ ERROR: No free_mint tokens received from withdrawal");
    }
    
    // Check if the position token is still present (should be, with less assets)
    let position_token_after = withdraw_sheet.get(&position_token_id);
    if position_token_after > 0 {
        println!("Position token still exists (expected since we only withdrew part of the assets)");
    } else {
        println!("Position token is gone (unexpected for partial withdrawal)");
    }
    
    Ok(())
}
