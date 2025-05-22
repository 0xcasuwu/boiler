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
use bitcoin::hashes::Hash; // Add Hash trait for all_zeros

use crate::tests::helper::init_factory;

use crate::precompiled::free_mint_build;
use crate::tests::std::alk4626_position_token_build;
use crate::tests::std::alk4626_vault_factory_build;

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
    let token_amount = mint_sheet.cached.balances.get(&expected_token_id);
    
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

#[wasm_bindgen_test]
fn test_vault_factory_interaction() -> Result<()> {
    clear();
    set_view_mode();

    let block_height: u32 = 850_000;

    let position_token_code = crate::tests::std::alk4626_position_token_build();
    let vault_factory_code = crate::tests::std::alk4626_vault_factory_build();
    let free_mint_code = crate::precompiled::free_mint_build();

    let contracts = vec![
        position_token_code,
        vault_factory_code,
        free_mint_code,
    ];

    let cellpacks: Vec<Cellpack> = vec![
        Cellpack { target: AlkaneId { block: 3, tx: 889 }, inputs: vec![10] }, // Position token deployment
        Cellpack { target: AlkaneId { block: 3, tx: 890 }, inputs: vec![10] }, // Vault factory deployment
        Cellpack { target: AlkaneId { block: 3, tx: 797 }, inputs: vec![101] }, // Free mint deployment
        Cellpack { target: AlkaneId { block: 6, tx: 797 }, inputs: vec![0, 100000000000, 100000000, 10, 4276545, 0, 4276545] }, // Free mint alkane initialization
        Cellpack { target: AlkaneId { block: 6, tx: 890 }, inputs: vec![0, 10000000000, 0, 1000000000] }, // Vault factory initialization
    ];

    let witnesses: Vec<Vec<Vec<u8>>> = vec![
        vec![], // Position token deployment
        vec![], // Vault factory deployment
        vec![], // Free mint deployment
        vec![], // Free mint alkane initialization
        vec![], // Vault factory initialization
    ];

    let (block, _) = alkane_helpers::init_tx_with_cellpacks_and_witness(
        contracts,
        cellpacks,
        witnesses,
    );

    index_block(&block, block_height)?;

    // Track output that created assets land on and find the free mint outpoint
    let tx = &block.txdata[0];
    let txid = tx.compute_txid();
    let mut out = stdout();

    let mut free_mint_outpoint: Option<OutPoint> = None;

    for vout in 0..tx.output.len() {
        let outpoint = OutPoint { txid, vout: vout as u32 };
        // Use alkanes_by_outpoint to find the free mint alkane
        let alkane_balances = view::alkanes_by_outpoint(&consensus_encode(&outpoint)?)?;
        writeln!(out, "Alkane balances for output {}: {:?}", vout, alkane_balances)?;

        // Check if this output contains the free mint alkane [2, 1]
        if alkane_balances.iter().any(|(id, _)| id == &AlkaneId { block: 2, tx: 1 }) {
            free_mint_outpoint = Some(outpoint);
            writeln!(out, "Found free mint alkane at outpoint: {:?}", outpoint)?;
            break; // Found the outpoint, no need to check other outputs
        }
    }

    let free_mint_outpoint = free_mint_outpoint.ok_or_else(|| anyhow::anyhow!("Free mint alkane outpoint not found"))?;

    // Construct the second block with the edict and message
    let mut txdata = Vec::new();

    // Create the transaction input spending the free mint alkane
    let tx_in = bitcoin::TxIn {
        previous_output: free_mint_outpoint,
        script_sig: bitcoin::Script::new().into(), // Placeholder script_sig
        sequence: bitcoin::Sequence::MAX,
        witness: bitcoin::Witness::new(), // Placeholder witness
    };

    // Create the transaction output that will receive the edicted alkane and the message
    let recipient_script = bitcoin::ScriptBuf::new_op_return(&[]); // Placeholder script_pubkey
    let tx_out = bitcoin::TxOut {
        value: bitcoin::Amount::from_sat(1000), // Placeholder value
        script_pubkey: recipient_script,
    };

    // Construct the Protostone with the edict
    let edict_protostone = protorune::message::Protostone {
        edict: Some(protorune::message::Edict {
            id: ProtoruneRuneId { block: 2, tx: 1 }, // Free mint alkane ID
            amount: 100000000, // Amount to edict
            output: 0, // Target the first output (index 0)
        }),
        message: None,
    };

    // Construct the Protostone with the message
    let message_cellpack = Cellpack {
        target: AlkaneId { block: 2, tx: 2 }, // Target AlkaneId for the message
        inputs: vec![100000000], // Inputs for the message cellpack
    };
    let message_protostone = protorune::message::Protostone {
        edict: None,
        message: Some(protorune::message::Message {
            cellpack: message_cellpack.encipher(), // Encipher the cellpack
        }),
    };

    // Construct the Runestone
    let runestone = protorune::message::Runestone {
        protostones: vec![edict_protostone, message_protostone],
        // Other Runestone fields as needed, potentially from the first block's Runestone
        // For simplicity, using default or minimal values for now.
        etching: None,
        mint: None,
        pointer: None,
        cenotaph: false,
        default_output: None,
        divisibility: None,
        premine: None,
        spacers: None,
        symbol: None,
        terms: None,
    };

    // Encode the Runestone into the transaction output script
    // This part requires knowing how Runestones are embedded in transaction outputs.
    // Based on ordinals/src/lib.rs or similar, it's likely embedded in an OP_RETURN script.
    // Need to confirm the exact encoding method.
    // Placeholder for now:
    let runestone_script = bitcoin::ScriptBuf::new_op_return(&consensus_encode(&runestone)?);
    let runestone_tx_out = bitcoin::TxOut {
        value: bitcoin::Amount::from_sat(0), // OP_RETURN outputs typically have 0 value
        script_pubkey: runestone_script,
    };


    txdata.push(bitcoin::Transaction {
        version: bitcoin::transaction::Version(2), // Placeholder version
        lock_time: bitcoin::blockdata::locktime::absolute::LockTime::ZERO, // Placeholder lock_time
        input: vec![tx_in],
        output: vec![tx_out, runestone_tx_out], // Include both the recipient output and the runestone output
    });

    let second_block = bitcoin::Block {
        header: bitcoin::blockdata::block::BlockHeader {
            version: bitcoin::blockdata::block::BlockVersion::from_consensus(0), // Placeholder version
            prev_blockhash: block.block_hash(), // Link to the previous block
            merkle_root: bitcoin::hash_types::TxMerkleNode::from_raw_hash(bitcoin::hash_types::sha256d::Hash::all_zeros()), // Placeholder merkle_root
            time: block.header.time + 600, // Increment time
            bits: block.header.bits, // Keep the same bits
            nonce: 0, // Placeholder nonce
        },
        txdata,
    };

    index_block(&second_block, block_height + 1)?;

    // Use alkanes::view::protorunes_by_outpoint and assert result
    // The new alkane should be at the first output of the second transaction (index 0)
    let new_alkane_outpoint = OutPoint {
        txid: second_block.txdata[0].compute_txid(),
        vout: 0,
    };

    let protorunes = view::protorunes_by_outpoint(&consensus_encode(&new_alkane_outpoint)?)?;
    writeln!(out, "Protorunes at new alkane outpoint: {:?}", protorunes)?;

    // Assert the expected protorune [2, 3] with the correct quantity
    // The quantity should be 100000000 as edicted
    let expected_protorune = (ProtoruneRuneId { block: 2, tx: 3 }, 100000000);
    assert!(protorunes.balances.entries.contains(&expected_protorune), "Expected protorune {:?} not found at outpoint {:?}", expected_protorune, new_alkane_outpoint);

    Ok(())
}
