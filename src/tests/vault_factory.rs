use alkanes::view;
use anyhow::Result;
use bitcoin::blockdata::transaction::OutPoint;
use wasm_bindgen_test::wasm_bindgen_test;
use alkanes::tests::helpers::clear;
use alkanes::indexer::index_block;
use alkanes::network::set_view_mode;
use std::fmt::Write;
use alkanes::message::AlkaneMessageContext;
use alkanes_support::cellpack::Cellpack;
use alkanes_support::id::AlkaneId;
use alkanes::tests::helpers as alkane_helpers;
use protorune::{balance_sheet::load_sheet, tables::RuneTable, message::MessageContext};
use protorune_support::{utils::consensus_encode, balance_sheet::ProtoruneRuneId};
use protorune::message::MessageContextParcel;
use metashrew_support::index_pointer::KeyValuePointer;
use bitcoin::hashes::Hash; // Add Hash trait for all_zeros
use protorune_support::protostone::Protostone;

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
    let mut test_block = alkane_helpers::init_with_multiple_cellpacks_with_tx(
        [
          free_mint_build::get_bytes(),
          alk4626_position_token_build::get_bytes(),
          alk4626_vault_factory_build::get_bytes(),
          [].into(),
          [].into()
        ].into(),
        [
          vec![3u128, 797u128, 101u128],
          vec![3u128, 0x379, 10u128],
          vec![3u128, 0x37a, 10u128],
          vec![6u128, 797u128, 0u128, 100000000000000u128, 100000000u128, 10u128, 0x414141, 0, 0x414141],
          vec![6u128, 0x379, 5000000, 0, 2, 1, 10_000_000_000_000]
        ].into_iter().map(|v| into_cellpack(v)).collect::<Vec<Cellpack>>()
    );
    index_block(&test_block, 0)?;
    let trace_result = view::trace(
        &(OutPoint {
            txid: test_block.txdata[test_block.txdata.len() - 1].compute_txid(),
            vout: 0,
        }),
    )?;
    println!("Trace result: {:?}", trace_result);
    Ok(())
}
