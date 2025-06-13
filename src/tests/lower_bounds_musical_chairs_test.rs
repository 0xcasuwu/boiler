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

// Test configuration structure for parameterized testing
#[derive(Debug, Clone)]
pub struct TestConfig {
    pub deposit_amounts: Vec<u128>,
    pub reward_per_block: u128,
    pub position_timeline: Vec<(u32, Option<u32>)>, // (deposit_block, withdrawal_block)
    pub reward_pool_size: u128,
    pub precision: u128,
    pub scenario_name: String,
}

impl TestConfig {
    // Lower bounds scenario - small deposits and rewards
    pub fn lower_bounds() -> Self {
        TestConfig {
            deposit_amounts: vec![100, 250, 150, 300],
            reward_per_block: 10,
            position_timeline: vec![
                (10, Some(35)),  // Position A: early exit
                (20, Some(50)),  // Position B: mid exit
                (30, Some(55)),  // Position C: late exit
                (40, Some(65)),  // Position D: final exit
            ],
            reward_pool_size: 10000, // Scale reward pool appropriately
            precision: 1000, // Lower precision for smaller numbers
            scenario_name: "Lower Bounds (100-300 deposits, 10 reward/block)".to_string(),
        }
    }
    
    // Very small amounts - testing precision limits
    pub fn micro_amounts() -> Self {
        TestConfig {
            deposit_amounts: vec![10, 25, 15, 30],
            reward_per_block: 1,
            position_timeline: vec![
                (10, Some(30)),  // Position A: short term
                (15, Some(35)),  // Position B: medium term
                (20, Some(40)),  // Position C: medium term
                (25, Some(45)),  // Position D: final exit
            ],
            reward_pool_size: 1000,
            precision: 100, // Very low precision for micro amounts
            scenario_name: "Micro Amounts (10-30 deposits, 1 reward/block)".to_string(),
        }
    }
    
    // Mixed amounts - different scales for complexity testing
    pub fn mixed_scale() -> Self {
        TestConfig {
            deposit_amounts: vec![50, 500, 200, 1000],
            reward_per_block: 25,
            position_timeline: vec![
                (10, Some(40)),  // Position A: small long-term
                (15, Some(35)),  // Position B: large short-term
                (25, Some(55)),  // Position C: medium long-term
                (30, Some(60)),  // Position D: whale medium-term
            ],
            reward_pool_size: 25000,
            precision: 1000,
            scenario_name: "Mixed Scale (50-1000 deposits, 25 reward/block)".to_string(),
        }
    }
}

// Position tracking with enhanced analytics
#[derive(Debug, Clone)]
struct Position {
    id: String,
    amount: u128,
    deposit_block: u32,
    withdrawal_block: Option<u32>,
    position_token_id: Option<ProtoruneRuneId>,
    deposit_block_ref: Option<Block>,
    expected_rewards: u128,
    actual_rewards: u128,
    reward_percentage: f64,
}

impl Position {
    fn new(id: &str, amount: u128, deposit_block: u32, withdrawal_block: Option<u32>) -> Self {
        Position {
            id: id.to_string(),
            amount,
            deposit_block,
            withdrawal_block,
            position_token_id: None,
            deposit_block_ref: None,
            expected_rewards: 0,
            actual_rewards: 0,
            reward_percentage: 0.0,
        }
    }
    
    fn calculate_reward_percentage(&mut self) {
        if self.amount > 0 {
            self.reward_percentage = (self.actual_rewards as f64 / self.amount as f64) * 100.0;
        }
    }
}

// Enhanced mathematical verification with precision analysis
fn verify_reward_calculation_with_precision(
    amount: u128,
    reward_per_block: u128, 
    blocks_elapsed: u128,
    precision: u128,
    expected: u128,
    test_name: &str,
    _config: &TestConfig
) -> (bool, u128, f64) {
    let calculated = amount
        .checked_mul(reward_per_block)
        .unwrap_or(0)
        .checked_mul(blocks_elapsed)
        .unwrap_or(0)
        .checked_div(precision)
        .unwrap_or(0);
    
    let matches = calculated == expected;
    let precision_loss = if expected > 0 {
        ((expected as f64 - calculated as f64) / expected as f64).abs() * 100.0
    } else {
        0.0
    };
    
    if matches {
        println!("✅ {}: {} * {} * {} / {} = {} (expected {}) - Precision ✓", 
                test_name, amount, reward_per_block, blocks_elapsed, precision, calculated, expected);
    } else {
        println!("⚠️ {}: {} * {} * {} / {} = {} (expected {}) - Loss: {:.2}%", 
                test_name, amount, reward_per_block, blocks_elapsed, precision, calculated, expected, precision_loss);
    }
    
    (matches, calculated, precision_loss)
}

// Parameterized vault setup function
fn create_parameterized_vault_setup(config: &TestConfig) -> Result<(Block, AlkaneId, u128)> {
    clear();
    
    println!("🏗️ Setting up vault with config: {}", config.scenario_name);
    println!("   • Reward pool size: {}", config.reward_pool_size);
    println!("   • Reward per block: {}", config.reward_per_block);
    println!("   • Precision: {}", config.precision);
    
    // Deploy contract templates - same as original
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
    
    // Create free_mint token with appropriate scaling for smaller amounts
    // Calculate total needed tokens for all deposits plus reward pool
    let total_deposits: u128 = config.deposit_amounts.iter().sum();
    let total_needed = total_deposits + config.reward_pool_size;
    let scaled_supply = total_needed.max(100000); // Ensure minimum supply
    let value_per_mint = total_needed / 10; // Allow more mints to get the total needed
    
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
                                    value_per_mint,       // scaled value per mint
                                    scaled_supply,        // scaled total supply
                                    0x414141, 0, 0x414141
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
    
    // Mint reward tokens scaled to config
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
    
    // Get available tokens for vault initialization
    let mint_outpoint = OutPoint { txid: mint_block.txdata[0].compute_txid(), vout: 0 };
    let mint_sheet = load_sheet(&RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
        .OUTPOINT_TO_RUNES.select(&consensus_encode(&mint_outpoint)?));
    let token_rune_id = ProtoruneRuneId { block: 2, tx: 1 };
    let available_tokens = mint_sheet.get(&token_rune_id);
    
    println!("🪙 Available tokens for vault: {}", available_tokens);
    
    // Initialize vault with parameterized values
    let deposit_token_id = AlkaneId { block: 2, tx: 1 };
    let reward_token_id = AlkaneId { block: 2, tx: 1 };
    let preloaded_rewards = available_tokens.min(config.reward_pool_size);
    
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
                                    4u128, 0x37a, 0u128,
                                    deposit_token_id.block, deposit_token_id.tx,
                                    reward_token_id.block, reward_token_id.tx,
                                    config.reward_per_block,
                                    3u128, // start_block
                                    preloaded_rewards,
                                    0u128 // fee_percentage
                                ]).encipher(),
                                protocol_tag: AlkaneMessageContext::protocol_tag() as u128,
                                pointer: Some(0),
                                refund: Some(0),
                                from: None,
                                burn: None,
                                edicts: vec![
                                    ProtostoneEdict {
                                        id: ProtoruneRuneId { block: deposit_token_id.block, tx: deposit_token_id.tx },
                                        amount: preloaded_rewards,
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
    
    println!("✅ Vault initialized with {} reward tokens at {} per block", preloaded_rewards, config.reward_per_block);
    
    let token_id = AlkaneId { block: 2, tx: 1 };
    Ok((init_vault_block, token_id, config.reward_per_block))
}

// Create tokens with specific amounts (scaled for test)
fn create_parameterized_tokens(block_height: u32, amount: u128) -> Result<Block> {
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
    index_block(&mint_block, block_height)?;
    
    println!("🪙 Created {} tokens at block {} for deposit", amount, block_height);
    Ok(mint_block)
}

// Enhanced deposit function with better validation
fn perform_parameterized_deposit(
    mint_block: &Block, 
    deposit_amount: u128, 
    user_name: &str, 
    block_height: u32,
    _config: &TestConfig
) -> Result<(Block, ProtoruneRuneId)> {
    let mint_outpoint = OutPoint {
        txid: mint_block.txdata[0].compute_txid(),
        vout: 0,
    };
    
    // Validate available tokens
    let mint_sheet = load_sheet(&RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
        .OUTPOINT_TO_RUNES.select(&consensus_encode(&mint_outpoint)?));
    let token_rune_id = ProtoruneRuneId { block: 2, tx: 1 };
    let available_tokens = mint_sheet.get(&token_rune_id);
    
    println!("💰 {} depositing {} tokens (has {} available) at block {}", 
             user_name, deposit_amount, available_tokens, block_height);
    
    if available_tokens < deposit_amount {
        return Err(anyhow::anyhow!("Insufficient tokens: have {}, need {}", available_tokens, deposit_amount));
    }
    
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
                                    4u128,              // Vault factory block
                                    0x37a,              // Vault factory tx  
                                    1u128,              // deposit opcode
                                    deposit_amount      // amount parameter
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
                                        amount: available_tokens,
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

    // Enhanced trace verification for smaller amounts
    let deposit_trace_data = &view::trace(&OutPoint {
        txid: deposit_block.txdata[0].compute_txid(),
        vout: 3,
    })?;
    let deposit_trace_result: alkanes_support::trace::Trace = alkanes_support::proto::alkanes::AlkanesTrace::parse_from_bytes(deposit_trace_data)?.into();
    
    let trace_debug_str = format!("{:?}", deposit_trace_result.0.lock().unwrap());
    
    if trace_debug_str.contains("Insufficient token value") || trace_debug_str.contains("unreachable") || trace_debug_str.contains("RevertContext") {
        println!("❌ DEPOSIT FAILED: {} - {}", user_name, 
                 if trace_debug_str.contains("Insufficient") { "Insufficient tokens" }
                 else if trace_debug_str.contains("unreachable") { "Unreachable code" }
                 else { "Transaction reverted" });
        return Err(anyhow::anyhow!("Deposit failed for {}", user_name));
    }
    
    println!("✅ DEPOSIT SUCCESS: {} deposited {} tokens", user_name, deposit_amount);

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
        .ok_or_else(|| anyhow::anyhow!("No position token found for {}", user_name))?;
    
    let position_token_id = ProtoruneRuneId {
        block: position_token_info.0.block,
        tx: position_token_info.0.tx,
    };

    Ok((deposit_block, position_token_id))
}

// Enhanced withdrawal with detailed analysis for small amounts
fn perform_parameterized_withdrawal(
    deposit_block: &Block, 
    position_token_id: ProtoruneRuneId, 
    position_name: &str, 
    block_height: u32,
    original_deposit: u128,
    _config: &TestConfig
) -> Result<(u128, u128)> {
    let position_outpoint = OutPoint {
        txid: deposit_block.txdata[0].compute_txid(),
        vout: 0,
    };

    println!("📤 {} withdrawing at block {} (original deposit: {})", 
             position_name, block_height, original_deposit);

    let withdrawal_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
        version: Version::ONE,
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input: vec![TxIn {
            previous_output: position_outpoint,
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
                                    4u128,      // Vault factory block
                                    0x37a,      // Vault factory tx  
                                    2u128,      // withdraw opcode
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
    index_block(&withdrawal_block, block_height)?;

    // Analyze withdrawal results
    let withdrawal_outpoint = OutPoint {
        txid: withdrawal_block.txdata[0].compute_txid(),
        vout: 0,
    };
    
    let withdrawal_sheet = load_sheet(
        &RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
            .OUTPOINT_TO_RUNES
            .select(&consensus_encode(&withdrawal_outpoint)?)
    );
    
    let mut total_returned = 0u128;
    for (id, amount) in withdrawal_sheet.balances().iter() {
        if id.block == 2 && id.tx == 1 {
            total_returned += *amount;
        }
    }

    let rewards_earned = total_returned.saturating_sub(original_deposit);
    let reward_percentage = if original_deposit > 0 {
        (rewards_earned as f64 / original_deposit as f64) * 100.0
    } else {
        0.0
    };

    println!("✅ {} withdrew {} total ({} principal + {} rewards = {:.2}% yield)", 
             position_name, total_returned, original_deposit, rewards_earned, reward_percentage);

    Ok((total_returned, rewards_earned))
}

// Calculate expected rewards with enhanced period analysis
fn calculate_parameterized_expected_rewards(
    amount: u128,
    reward_per_block: u128, 
    periods: &[(u32, u32, u128)],
    precision: u128,
    position_name: &str
) -> u128 {
    let mut total_rewards = 0u128;
    
    println!("🧮 Calculating expected rewards for {}:", position_name);
    
    for (start_block, end_block, total_pool_amount) in periods {
        let blocks_in_period = end_block - start_block;
        
        if *total_pool_amount == 0 {
            println!("   ⚠️ Period {}-{}: Empty pool, skipping", start_block, end_block);
            continue;
        }
        
        let period_rewards = amount
            .checked_mul(reward_per_block)
            .unwrap_or(0)
            .checked_mul(blocks_in_period as u128)
            .unwrap_or(0)
            .checked_div(*total_pool_amount)
            .unwrap_or(0);
        
        total_rewards = total_rewards.checked_add(period_rewards).unwrap_or(total_rewards);
        
        let share_percentage = if *total_pool_amount > 0 {
            (amount as f64 / *total_pool_amount as f64) * 100.0
        } else {
            0.0
        };
        
        println!("   • Period {}-{}: {:.1}% of {} pool ({} blocks) = {} rewards", 
                 start_block, end_block, share_percentage, total_pool_amount, blocks_in_period, period_rewards);
    }
    
    println!("   📊 Total expected rewards: {}", total_rewards);
    total_rewards
}

// Main parameterized test executor
fn execute_musical_chairs_scenario(config: TestConfig) -> Result<()> {
    println!("\n🎭 EXECUTING SCENARIO: {}", config.scenario_name);
    println!("{}", "=".repeat(80));
    
    // Set up vault system
    let (_init_block, _token_id, reward_per_block) = create_parameterized_vault_setup(&config)?;
    
    // Initialize positions
    let mut positions = vec![
        Position::new("Position_A", config.deposit_amounts[0], config.position_timeline[0].0, config.position_timeline[0].1),
        Position::new("Position_B", config.deposit_amounts[1], config.position_timeline[1].0, config.position_timeline[1].1),
        Position::new("Position_C", config.deposit_amounts[2], config.position_timeline[2].0, config.position_timeline[2].1),
        Position::new("Position_D", config.deposit_amounts[3], config.position_timeline[3].0, config.position_timeline[3].1),
    ];
    
    println!("\n🎪 POSITION TIMELINE:");
    for position in &positions {
        println!("• {}: {} tokens, blocks {}-{}", 
                 position.id, position.amount, 
                 position.deposit_block, position.withdrawal_block.unwrap_or(0));
    }
    
    // Create tokens for each position
    println!("\n💰 CREATING TOKENS:");
    let mut mint_blocks = Vec::new();
    for (i, position) in positions.iter().enumerate() {
        let mint_block = create_parameterized_tokens(4 + i as u32, position.amount)?;
        mint_blocks.push(mint_block);
    }
    
    // Execute deposits chronologically
    println!("\n📥 EXECUTING DEPOSITS:");
    let mut position_data = Vec::new();
    
    for (i, position) in positions.iter().enumerate() {
        let (deposit_block, position_token_id) = perform_parameterized_deposit(
            &mint_blocks[i], 
            position.amount, 
            &position.id, 
            position.deposit_block,
            &config
        )?;
        
        position_data.push((deposit_block, position_token_id));
    }
    
    // Execute withdrawals chronologically
    println!("\n📤 EXECUTING WITHDRAWALS:");
    let mut results = Vec::new();
    
    for (i, position) in positions.iter().enumerate() {
        if let Some(withdrawal_block) = position.withdrawal_block {
            let (total_returned, rewards_earned) = perform_parameterized_withdrawal(
                &position_data[i].0,
                position_data[i].1,
                &position.id,
                withdrawal_block,
                position.amount,
                &config
            )?;
            
            results.push((total_returned, rewards_earned));
        }
    }
    
    // Analysis and verification
    println!("\n📊 SCENARIO ANALYSIS:");
    let mut total_rewards_distributed = 0u128;
    
    for (i, (total_returned, rewards_earned)) in results.iter().enumerate() {
        let position = &positions[i];
        let reward_percentage = if position.amount > 0 {
            (*rewards_earned as f64 / position.amount as f64) * 100.0
        } else {
            0.0
        };
        
        println!("• {}: {} principal + {} rewards = {} total ({:.2}% yield)", 
                 position.id, position.amount, rewards_earned, total_returned, reward_percentage);
        
        total_rewards_distributed += rewards_earned;
    }
    
    println!("\n✅ SCENARIO COMPLETED:");
    println!("   • Total rewards distributed: {}", total_rewards_distributed);
    println!("   • Average reward per position: {}", total_rewards_distributed / results.len() as u128);
    println!("   • All positions recovered principal: {}", 
             results.iter().enumerate().all(|(i, (total, _))| *total >= positions[i].amount));
    
    Ok(())
}

// Individual test functions for each scenario
#[wasm_bindgen_test]
fn test_lower_bounds_musical_chairs() -> Result<()> {
    let config = TestConfig::lower_bounds();
    execute_musical_chairs_scenario(config)
}

#[wasm_bindgen_test]
fn test_micro_amounts_musical_chairs() -> Result<()> {
    let config = TestConfig::micro_amounts();
    execute_musical_chairs_scenario(config)
}

#[wasm_bindgen_test]
fn test_mixed_scale_musical_chairs() -> Result<()> {
    let config = TestConfig::mixed_scale();
    execute_musical_chairs_scenario(config)
}

// Comprehensive test that runs all scenarios
#[wasm_bindgen_test]
fn test_all_parameterized_scenarios() -> Result<()> {
    println!("🎯 RUNNING ALL PARAMETERIZED MUSICAL CHAIRS SCENARIOS");
    println!("{}", "=".repeat(100));
    
    let scenarios = vec![
        TestConfig::lower_bounds(),
        TestConfig::micro_amounts(),
        TestConfig::mixed_scale(),
    ];
    
    for (i, config) in scenarios.into_iter().enumerate() {
        println!("\n🎬 SCENARIO {} OF 3", i + 1);
        match execute_musical_chairs_scenario(config) {
            Ok(_) => println!("✅ Scenario {} completed successfully", i + 1),
            Err(e) => {
                println!("❌ Scenario {} failed: {}", i + 1, e);
                return Err(e);
            }
        }
    }
    
    println!("\n🎉 ALL PARAMETERIZED SCENARIOS COMPLETED SUCCESSFULLY!");
    Ok(())
}

// Test specifically for the 100-1000 deposit range with various reward rates
#[wasm_bindgen_test]
fn test_deposit_range_100_to_1000() -> Result<()> {
    println!("🎯 TESTING DEPOSIT RANGE 100-1000 WITH VARIABLE REWARDS");
    println!("{}", "=".repeat(80));
    
    let test_cases = vec![
        // (deposits, reward_per_block, scenario_name)
        (vec![100, 200, 500, 1000], 5, "Ultra Low Rewards (5/block)"),
        (vec![150, 300, 750, 1000], 15, "Low Rewards (15/block)"),
        (vec![100, 250, 600, 800], 25, "Medium Rewards (25/block)"),
        (vec![200, 400, 700, 1000], 50, "High Rewards (50/block)"),
    ];
    
    for (i, (deposits, reward_rate, name)) in test_cases.into_iter().enumerate() {
        println!("\n🧪 TEST CASE {}: {}", i + 1, name);
        
        let config = TestConfig {
            deposit_amounts: deposits.clone(),
            reward_per_block: reward_rate,
            position_timeline: vec![
                (10, Some(40)),  // Position A: 30 blocks
                (15, Some(35)),  // Position B: 20 blocks
                (20, Some(50)),  // Position C: 30 blocks
                (25, Some(45)),  // Position D: 20 blocks
            ],
            reward_pool_size: deposits.iter().sum::<u128>() * 10, // Scale pool to deposits
            precision: 1000,
            scenario_name: format!("{} - Deposits: {:?}, Rewards: {}/block", name, deposits, reward_rate),
        };
        
        match execute_musical_chairs_scenario(config) {
            Ok(_) => println!("✅ Test case {} ({}) completed successfully", i + 1, name),
            Err(e) => {
                println!("❌ Test case {} ({}) failed: {}", i + 1, name, e);
                return Err(e);
            }
        }
    }
    
    println!("\n🎉 ALL 100-1000 DEPOSIT RANGE TESTS COMPLETED!");
    Ok(())
}
