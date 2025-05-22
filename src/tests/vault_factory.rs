use alkanes::view;
use alkane_factory_support::constants::ALKANE_FACTORY_FREE_MINT_ID;
use anyhow::Result;
use bitcoin::blockdata::transaction::OutPoint;
use wasm_bindgen_test::wasm_bindgen_test;
use alkanes::tests::helpers::clear;
use alkanes::indexer::index_block;
use alkanes::network::set_view_mode;
use metashrew::stdio::stdout;
use std::fmt::Write;
use alkanes::message::AlkaneMessageContext;
use alkanes_support::cellpack::Cellpack;
use alkanes_support::id::AlkaneId;
use alkanes::tests::helpers as alkane_helpers;
use protorune::{balance_sheet::load_sheet, tables::RuneTable, message::MessageContext};
use protorune_support::{utils::consensus_encode, balance_sheet::ProtoruneRuneId};
use protorune::message::MessageContextParcel;
use metashrew_support::index_pointer::KeyValuePointer;

use crate::tests::helper::init_factory;

#[wasm_bindgen_test]
fn test_free_mint_deployment() -> Result<()> {
    clear();
    set_view_mode();
    
    let (block, deployment_ids) = init_factory::init_free_mint_block()?;
    
    let block_height: u32 = 850_000;
    index_block(&block, block_height)?;
    
    // Assert the contract was deployed correctly
    init_factory::assert_free_mint_deployed(&deployment_ids)?;
    
    // Get the last transaction for tracing
    let trace_result = view::trace(
        &(OutPoint {
            txid: block.txdata[block.txdata.len() - 1].compute_txid(),
            vout: 0,
        }),
    )?;
    
    let mut out = stdout();
    writeln!(out, "Trace result: {:?}", trace_result)?;
    
    // Verify the deployment ID matches what we expect
    assert_eq!(
        deployment_ids.free_mint_factory.tx, 
        ALKANE_FACTORY_FREE_MINT_ID,
        "Free mint contract should be deployed with the correct ID"
    );
    
    assert_eq!(
        deployment_ids.free_mint_factory.block, 
        4,
        "Free mint contract should be deployed to block 4"
    );
    
    writeln!(out, "Free mint contract successfully deployed and verified")?;
    
    Ok(())
}

#[wasm_bindgen_test]
fn test_free_mint_token_creation() -> Result<()> {
    clear();
    set_view_mode();
    
    let block_height: u32 = 850_000;
    
    let (contract_block, deployment_ids) = init_factory::init_free_mint_block()?;
    index_block(&contract_block, block_height)?;
    
    init_factory::assert_free_mint_deployed(&deployment_ids)?;
    
    // Now create a new token 
    // Parameters for the new token:
    // - opcode 0 (initialize)
    // - token_units: 1000 (amount per mint)
    // - value_per_mint: 1000 (value per mint)
    // - cap: 100 (max supply)
    // - name: 0x414243 (ASCII: "ABC")
    // - symbol: 0x58595A (ASCII: "XYZ")
    let token_cellpacks: Vec<Cellpack> = [
        Cellpack {
            target: AlkaneId {
                block: deployment_ids.free_mint_factory.block,
                tx: ALKANE_FACTORY_FREE_MINT_ID,
            },
            inputs: vec![0, 1000, 1000, 100, 0x414243, 0x58595A],
        },
    ]
    .into();
    
    // Create a new block with the token creation transaction
    let token_block = alkane_helpers::init_with_multiple_cellpacks_with_tx(
        [vec![]].into(), // No new contract code
        token_cellpacks,
    );
    
    index_block(&token_block, block_height + 1)?;
    
    // Get the token creation transaction outpoint
    let token_len = token_block.txdata.len();
    let token_outpoint = OutPoint {
        txid: token_block.txdata[token_len - 1].compute_txid(),
        vout: 0,
    };
    
    // Check the balance sheet after token creation
    let token_ptr = RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
        .OUTPOINT_TO_RUNES
        .select(&consensus_encode(&token_outpoint)?);
    let token_sheet = load_sheet(&token_ptr);
    
    let mut out = stdout();
    writeln!(out, "Token balances: {:?}", token_sheet)?;
    
    // Query the contract for the token cap
    let mut cap_parcel = MessageContextParcel::default();
    cap_parcel.height = u64::from(block_height) + 2;
    cap_parcel.calldata = (Cellpack {
        target: AlkaneId { 
            block: deployment_ids.free_mint_factory.block, 
            tx: ALKANE_FACTORY_FREE_MINT_ID 
        },
        inputs: vec![102], // 102 is the opcode to get the cap
    })
    .encipher();
    
    // Simulate the call and get the response
    let cap_response = view::simulate_parcel(&cap_parcel, u64::MAX)?;
    let cap_bytes = cap_response.0.data;
    
    // Convert the bytes to a u128 (cap value)
    let cap = u128::from_le_bytes(cap_bytes.try_into().unwrap_or([0; 16]));
    writeln!(out, "Token cap: {}", cap)?;
    
    // Verify the cap is set correctly
    assert_eq!(cap, 100, "Cap should be 100");
    
    // Query the contract for the token name
    let mut name_parcel = MessageContextParcel::default();
    name_parcel.height = u64::from(block_height) + 2;
    name_parcel.calldata = (Cellpack {
        target: AlkaneId { 
            block: deployment_ids.free_mint_factory.block, 
            tx: ALKANE_FACTORY_FREE_MINT_ID 
        },
        inputs: vec![99], // 99 is the opcode to get the name
    })
    .encipher();
    
    // Simulate the call and get the response
    let name_response = view::simulate_parcel(&name_parcel, u64::MAX)?;
    let name_bytes = name_response.0.data;
    
    // Convert the bytes to a string
    let name = String::from_utf8(name_bytes)?;
    writeln!(out, "Token name: {}", name)?;
    
    // Verify the name is set correctly
    assert_eq!(name, "CBA", "Name should be CBA");
    
    // Query the contract for the token symbol
    let mut symbol_parcel = MessageContextParcel::default();
    symbol_parcel.height = u64::from(block_height) + 2;
    symbol_parcel.calldata = (Cellpack {
        target: AlkaneId { 
            block: deployment_ids.free_mint_factory.block, 
            tx: ALKANE_FACTORY_FREE_MINT_ID 
        },
        inputs: vec![100], // 100 is the opcode to get the symbol
    })
    .encipher();
    
    // Simulate the call and get the response
    let symbol_response = view::simulate_parcel(&symbol_parcel, u64::MAX)?;
    let symbol_bytes = symbol_response.0.data;
    
    // Convert the bytes to a string
    let symbol = String::from_utf8(symbol_bytes)?;
    writeln!(out, "Token symbol: {}", symbol)?;
    
    // Verify the symbol is set correctly
    assert_eq!(symbol, "ZYX", "Symbol should be ZYX");
    
    writeln!(out, "Token successfully created and verified")?;
    
    Ok(())
}

#[wasm_bindgen_test]
fn test_free_mint_token_minting() -> Result<()> {
    clear();
    set_view_mode();
    let block_height: u32 = 850_000;
    
    let (contract_block, deployment_ids) = init_factory::init_free_mint_block()?;
    index_block(&contract_block, block_height)?;
    
    init_factory::assert_free_mint_deployed(&deployment_ids)?;
    
    // Create a new token
    let token_cellpacks: Vec<Cellpack> = [
        Cellpack {
            target: AlkaneId {
                block: deployment_ids.free_mint_factory.block,
                tx: ALKANE_FACTORY_FREE_MINT_ID,
            },
            inputs: vec![0, 1000, 1000, 100, 0x414243, 0x58595A],
        },
    ]
    .into();
    
    // Create a new block with the token creation transaction
    let token_block = alkane_helpers::init_with_multiple_cellpacks_with_tx(
        [vec![]].into(), 
        token_cellpacks,
    );
    
    index_block(&token_block, block_height + 1)?;
    
    // Get the token creation transaction outpoint
    let token_len = token_block.txdata.len();
    let token_outpoint = OutPoint {
        txid: token_block.txdata[token_len - 1].compute_txid(),
        vout: 0,
    };
    
    // Check the balance sheet after token creation
    let token_ptr = RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
        .OUTPOINT_TO_RUNES
        .select(&consensus_encode(&token_outpoint)?);
    let token_sheet = load_sheet(&token_ptr);
    
    let mut out = stdout();
    writeln!(out, "Initial token balances: {:?}", token_sheet)?;
    
    // Mint a token
    let mint_cellpacks: Vec<Cellpack> = [
        Cellpack {
            target: AlkaneId {
                block: deployment_ids.free_mint_factory.block,
                tx: ALKANE_FACTORY_FREE_MINT_ID,
            },
            inputs: vec![77],
        },
    ]
    .into();
    
    // Create a new block with the mint transaction
    let mint_block = alkane_helpers::init_with_multiple_cellpacks_with_tx(
        [vec![]].into(), 
        mint_cellpacks,
    );
    
    index_block(&mint_block, block_height + 2)?;
    
    // Get the mint transaction outpoint
    let mint_len = mint_block.txdata.len();
    let mint_outpoint = OutPoint {
        txid: mint_block.txdata[mint_len - 1].compute_txid(),
        vout: 0,
    };
    
    // Check the balance sheet after minting
    let mint_ptr = RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
        .OUTPOINT_TO_RUNES
        .select(&consensus_encode(&mint_outpoint)?);
    let mint_sheet = load_sheet(&mint_ptr);
    
    writeln!(out, "Balances after mint: {:?}", mint_sheet)?;
    
    // Create the expected token ID
    let expected_token_id = ProtoruneRuneId {
        block: deployment_ids.free_mint_factory.block,
        tx: ALKANE_FACTORY_FREE_MINT_ID,
    };
    
    // Check if the mint_sheet contains the token with the expected amount
    let token_amount = mint_sheet.balances.get(&expected_token_id);
    
    writeln!(out, "Token amount: {:?}", token_amount)?;
    
    // Verify the token was minted with the correct amount
    assert!(token_amount.is_some(), "Token should be present in the balance sheet");
    assert_eq!(*token_amount.unwrap(), 1000, "Token amount should be 1000");
    
    // Query the contract for minted count
    let mut minted_parcel = MessageContextParcel::default();
    minted_parcel.height = u64::from(block_height) + 3;
    minted_parcel.calldata = (Cellpack {
        target: AlkaneId { 
            block: deployment_ids.free_mint_factory.block, 
            tx: ALKANE_FACTORY_FREE_MINT_ID 
        },
        inputs: vec![103], // 103 is the opcode to get minted count
    })
    .encipher();
    
    // Simulate the call and get the response
    let minted_response = view::simulate_parcel(&minted_parcel, u64::MAX)?;
    let minted_bytes = minted_response.0.data;
    
    // Convert the bytes to a u128 (minted count)
    let minted = u128::from_le_bytes(minted_bytes.try_into().unwrap_or([0; 16]));
    writeln!(out, "Minted count: {}", minted)?;
    
    // Verify one token was minted
    assert_eq!(minted, 1, "Minted count should be 1");
    
    // Query the contract for total supply
    let mut supply_parcel = MessageContextParcel::default();
    supply_parcel.height = u64::from(block_height) + 3;
    supply_parcel.calldata = (Cellpack {
        target: AlkaneId { 
            block: deployment_ids.free_mint_factory.block, 
            tx: ALKANE_FACTORY_FREE_MINT_ID 
        },
        inputs: vec![101], // 101 is the opcode to get total supply
    })
    .encipher();
    
    // Simulate the call and get the response
    let supply_response = view::simulate_parcel(&supply_parcel, u64::MAX)?;
    let supply_bytes = supply_response.0.data;
    
    // Convert the bytes to a u128 (total supply)
    let total_supply = u128::from_le_bytes(supply_bytes.try_into().unwrap_or([0; 16]));
    writeln!(out, "Total supply: {}", total_supply)?;
    
    // Verify the total supply is correct (initial 1000 + minted 1000)
    assert_eq!(total_supply, 2000, "Total supply should be 2000");
    
    writeln!(out, "Token successfully minted and verified")?;
    
    Ok(())
} 
