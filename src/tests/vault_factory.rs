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
fn test_deployment() -> Result<()> {
    clear();
    // Deploy all the contract templates
    let mut template_block = alkane_helpers::init_with_multiple_cellpacks_with_tx(
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
    let trace_result: alkanes_support::trace::Trace = alkanes_support::proto::alkanes::AlkanesTrace::parse_from_bytes(&view::trace(
        &(OutPoint {
            txid: template_block.txdata[template_block.txdata.len() - 3].compute_txid(),
            vout: 3,
        }),
    )?)?.into();
    println!("Trace result: {:?}", trace_result);
    
    // Initialize the free_mint contract with parameters
    let mut free_mint_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
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
                        from: None,
                        burn: None,
                        pointer: Some(0),
                        refund: Some(0),
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
    
    // create a trace
    let init_trace_result: alkanes_support::trace::Trace = alkanes_support::proto::alkanes::AlkanesTrace::parse_from_bytes(&view::trace(
        &(OutPoint {
            txid: free_mint_block.txdata[0].compute_txid(),
            vout: 3,
        }),
    )?)?.into();

    println!("Free mint initialization trace result: {:?}", init_trace_result);
    
    // Step: Mint free_mint tokens that will be used for deposit
    let mut mint_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
      version: Version::ONE,
      lock_time: bitcoin::absolute::LockTime::ZERO,
      input: vec![TxIn {
        previous_output: OutPoint::null(),
        script_sig: ScriptBuf::new(),
        sequence: Sequence::MAX,
        witness: Witness::new()
      }],
      output: vec![
        TxOut { // Output to receive the minted tokens
          script_pubkey: Address::from_str(ADDRESS1().as_str())
            .unwrap()
            .require_network(get_btc_network())
            .unwrap()
            .script_pubkey(),
          value: Amount::from_sat(546),
        },
        TxOut { // Contains the mint call
          script_pubkey: (Runestone {
            edicts: vec![],
            etching: None,
            mint: None,
            pointer: None,
            protocol: Some(
                vec![
                    Protostone {
                        message: into_cellpack(vec![2u128, 1u128, 77u128]).encipher(), // 77 = MintTokens with correct contract ID
                        protocol_tag: AlkaneMessageContext::protocol_tag() as u128,
                        from: None,
                        burn: None,
                        pointer: Some(0), // Minted tokens go to output 0
                        refund: Some(0),
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
    
    // Trace the mint operation
    let mint_trace_result: alkanes_support::trace::Trace = alkanes_support::proto::alkanes::AlkanesTrace::parse_from_bytes(&view::trace(
        &(OutPoint {
            txid: mint_block.txdata[0].compute_txid(),
            vout: 1, // Trace from the mint message output
        }),
    )?)?.into();
    println!("Mint trace result: {:?}", mint_trace_result);
    
    // Check the balance after minting to verify we have tokens
    let mint_outpoint = OutPoint {
        txid: mint_block.txdata[0].compute_txid(),
        vout: 0, // The output where tokens were sent
    };
    
    let mint_sheet = load_sheet(
        &RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
            .OUTPOINT_TO_RUNES
            .select(&consensus_encode(&mint_outpoint)?)
    );
    
    println!("--- After Mint Balances ---");
    for (id, amount) in mint_sheet.balances().iter() {
        println!("Token ID: {:?}, Amount: {}", id, amount);
    }
    
    // Set the free_mint_id to the actual ID that was assigned (from the trace)
    let free_mint_id = AlkaneId { block: 2, tx: 1 };
    let mut test_block = alkane_helpers::init_with_multiple_cellpacks_with_tx(
        Vec::new(),
        [
          // Initialize vault factory with reward info
          vec![4u128, 0x37a, 0u128, 1000u128, 1u128, 4u128, free_mint_id.block, free_mint_id.tx], // Initialize vault factory with correct ID
        ].into_iter().map(|v| into_cellpack(v)).collect::<Vec<Cellpack>>()
    );
    index_block(&test_block, 3)?;

    // Get the correct vault factory ID from the result of initialization
    let vault_factory_id = AlkaneId { block: 4, tx: 0x37a };

    // Create deposit transaction with a single Protostone that includes:
    // 1. Using Runestone to transfer tokens
    // 2. Pointing to the vault_factory for the deposit operation
    let deposit_amount = 100u128; // Amount to deposit
    
    // Try a direct approach with a single Protostone containing both message and token transfer
    println!("Creating deposit transaction with deposit amount: {}", deposit_amount);
    
    // Create the deposit transaction
    let mut deposit_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
      version: Version::ONE,
      lock_time: bitcoin::absolute::LockTime::ZERO,
      input: vec![TxIn {
        previous_output: OutPoint {
          // Reference the block where tokens were minted
          txid: mint_block.txdata[0].compute_txid(),
          vout: 0, // Output containing our minted tokens
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
        // Output 1: Contains the deposit message
        TxOut {
          script_pubkey: (Runestone {
            edicts: vec![],
            etching: None,
            mint: None,
            pointer: None,
            protocol: Some(
                vec![
                    Protostone {
                        // The deposit message to the vault factory (with correct IDs)
                        message: into_cellpack(vec![
                            4u128,           // Make sure we use the confirmed block ID from trace
                            0x37a,           // Use the vault factory tx ID
                            1u128,           // deposit opcode
                            deposit_amount   // amount to deposit
                        ]).encipher(),
                        protocol_tag: AlkaneMessageContext::protocol_tag() as u128,
                        pointer: Some(0),    // Position token goes to output 0
                        refund: Some(0),     // Refunds go to output 0
                        from: None,
                        burn: None,
                        // IMPORTANT: Self-transfer the tokens to the contract for deposit
                        edicts: vec![
                            ProtostoneEdict {
                                id: ProtoruneRuneId {
                                    block: free_mint_id.block,
                                    tx: free_mint_id.tx
                                },
                                amount: deposit_amount,
                                output: 1,    // Target the message output itself
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

    // Trace the deposit operation
    let deposit_trace_result: alkanes_support::trace::Trace = alkanes_support::proto::alkanes::AlkanesTrace::parse_from_bytes(&view::trace(
        &(OutPoint {
            txid: deposit_block.txdata[0].compute_txid(),
            vout: 3, // The output with the deposit message
        }),
    )?)?.into();
    println!("Deposit trace result: {:?}", deposit_trace_result);

    // Check the balance after deposit using the output that should receive position tokens
    let position_outpoint = OutPoint {
        txid: deposit_block.txdata[0].compute_txid(),
        vout: 0, // Output that should receive position token
    };
    
    // Use the balance sheet to verify token movements
    let position_sheet = load_sheet(
        &RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
            .OUTPOINT_TO_RUNES
            .select(&consensus_encode(&position_outpoint)?)
    );
    
    println!("--- After Deposit Balances (Position Token Output) ---");
    for (id, amount) in position_sheet.balances().iter() {
        println!("Token ID: {:?}, Amount: {}", id, amount);
    }
    
    // Check if we have a position token returned
    let position_token = position_sheet.cached.balances.iter().next();
    if let Some((position_id, position_amount)) = position_token {
        println!("Position token ID: {:?}, Amount: {}", position_id, position_amount);
        
        // Verify this is indeed a position token by checking it's not the free_mint token
        let free_mint_rune_id = ProtoruneRuneId { block: free_mint_id.block, tx: free_mint_id.tx };
        if position_id != &free_mint_rune_id {
            println!("Successfully received position token!");
        } else {
            println!("WARNING: Received token is the free_mint token, not a position token");
        }
    } else {
        println!("No position token returned from deposit");
    }
    
    Ok(())
}
