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
use ordinals::Runestone;
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
                        message: into_cellpack(vec![6u128, 797u128, 0u128, 100000000000000u128, 100000000u128, 10u128, 0x414141, 0, 0x414141]).encipher(),
                        protocol_tag: 1,
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
    let mint_result: alkanes_support::trace::Trace = alkanes_support::proto::alkanes::AlkanesTrace::parse_from_bytes(&view::trace(
        &(OutPoint {
            txid: free_mint_block.txdata[0].compute_txid(),
            vout: 3,
        }),
    )?)?.into();

    println!("Trace result: {:?}", mint_result);
    
    // Complete the test_block with vault factory initialization
    let mut test_block = alkane_helpers::init_with_multiple_cellpacks_with_tx(
        Vec::new(),
        [
          // Initialize free_mint with parameters (already done above)
          // Initialize vault factory with free_mint as reward token
          vec![6u128, 0x37a, 0u128, 1000u128, 1u128, 4u128, 797u128, 100u128], // Initialize vault factory
        ].into_iter().map(|v| into_cellpack(v)).collect::<Vec<Cellpack>>()
    );
    index_block(&test_block, 2)?;

    // Create a transaction with a protostone that has both the message and an edict
    // transferring free_mint tokens to the vault factory for deposit
    let free_mint_id = AlkaneId { block: 4, tx: 797 };
    let vault_factory_id = AlkaneId { block: 4, tx: 0x37a };

    let mut deposit_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
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
                    // Edict transferring free_mint tokens to be used as deposit
                    Protostone {
                        edicts: vec![
                            protorune_support::protostone::ProtostoneEdict {
                                id: ProtoruneRuneId { 
                                    block: free_mint_id.block, 
                                    tx: free_mint_id.tx 
                                },
                                amount: 10000,
                                output: 0,
                            }
                        ],
                        protocol_tag: 1,
                        burn: None,
                        from: None,
                        pointer: Some(0),
                        refund: Some(0),
                        message: vec![],  // Empty Vec<u8> instead of None
                    },
                    // The deposit message to the vault factory
                    Protostone {
                        message: into_cellpack(vec![6u128, 0x37a, 1u128, 100u128]).encipher(),
                        protocol_tag: 1,
                        burn: None,
                        from: None,
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
    index_block(&deposit_block, 3)?;

    // Trace the deposit operation with the correct vout (3 for the Protostone)
    let deposit_trace_result: alkanes_support::trace::Trace = alkanes_support::proto::alkanes::AlkanesTrace::parse_from_bytes(&view::trace(
        &(OutPoint {
            txid: deposit_block.txdata[0].compute_txid(),
            vout: 3, // Use vout 3 for the Protostone with the deposit message
        }),
    )?)?.into();
    println!("Deposit trace result: {:?}", deposit_trace_result);

    // Check the balance after deposit using the correct output
    let deposit_outpoint = OutPoint {
        txid: deposit_block.txdata[0].compute_txid(),
        vout: 3, // Use vout 3 for the protostone output
    };
    
    // Use the balance sheet to verify token movements
    let deposit_sheet = load_sheet(
        &RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
            .OUTPOINT_TO_RUNES
            .select(&consensus_encode(&deposit_outpoint)?)
    );
    
    println!("--- After Deposit Balances ---");
    for (id, amount) in deposit_sheet.balances().iter() {
        println!("Token ID: {:?}, Amount: {}", id, amount);
    }
    
    // Check if we have a position token returned
    let position_balance = deposit_sheet.cached.balances.values().next();
    if let Some(position_amount) = position_balance {
        println!("Position token amount: {}", position_amount);
    } else {
        println!("No position token returned from deposit");
    }
    
    Ok(())
}
