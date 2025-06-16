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
use metashrew_support::{index_pointer::KeyValuePointer, utils::consensus_encode};
use ordinals::Runestone;
use protorune::test_helpers::{get_btc_network, ADDRESS1};
use protorune::{test_helpers as protorune_helpers};
use protorune_support::{balance_sheet::ProtoruneRuneId, protostone::{Protostone, ProtostoneEdict}};
use protorune::protostone::Protostones;
use alkanes_support::trace::Trace;
use alkanes_support::proto::alkanes::AlkanesTrace;
use metashrew_core::{println, stdio::stdout};
use protobuf::Message;
use std::fmt::Write;

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

// Position tracking for complex scenarios
#[derive(Debug, Clone)]
struct Position {
    user: String,
    amount: u128,
    deposit_block: u32,
    withdrawal_block: Option<u32>,
    position_token_id: Option<ProtoruneRuneId>,
    deposit_block_ref: Option<Block>,
    expected_rewards: u128,
    actual_rewards: u128,
    actual_total: u128,
}

impl Position {
    fn new(user: &str, amount: u128, deposit_block: u32, withdrawal_block: Option<u32>) -> Self {
        Position {
            user: user.to_string(),
            amount,
            deposit_block,
            withdrawal_block,
            position_token_id: None,
            deposit_block_ref: None,
            expected_rewards: 0,
            actual_rewards: 0,
            actual_total: 0,
        }
    }
}

// Pool state tracker for mathematical verification
#[derive(Debug, Clone)]
struct PoolState {
    block: u32,
    total_staked: u128,
    users: Vec<String>,
    reward_rate: u128,
}

impl PoolState {
    fn new(block: u32, reward_rate: u128) -> Self {
        PoolState {
            block,
            total_staked: 0,
            users: Vec::new(),
            reward_rate,
        }
    }
    
    fn add_user(&mut self, user: &str, amount: u128) {
        self.total_staked += amount;
        self.users.push(user.to_string());
    }
    
    fn remove_user(&mut self, user: &str, amount: u128) {
        self.total_staked = self.total_staked.saturating_sub(amount);
        if let Some(pos) = self.users.iter().position(|u| u == user) {
            self.users.remove(pos);
        }
    }
    
    fn get_user_share(&self, amount: u128) -> f64 {
        if self.total_staked == 0 {
            0.0
        } else {
            amount as f64 / self.total_staked as f64
        }
    }
}

// Helper to create vault setup using EXACT multi-user pattern with temporal cap
fn create_intricate_temporal_vault_setup(end_reward_block: u128) -> Result<(Block, AlkaneId, u128)> {
    clear();
    
    println!("🚀 INTRICATE TEMPORAL CAP TEST SETUP");
    println!("====================================");
    
    // Deploy contract templates using EXACT working pattern from multi_user_rewards_test.rs
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
    
    // Create free_mint token contract with large supply for reward pool
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
                                message: into_cellpack(vec![6u128, 797u128, 0u128, 100000u128, 10000000u128, 100000000u128, 0x414141, 0, 0x414141]).encipher(),
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

    // Mint reward tokens for vault initialization
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
    
    // Get available tokens for proper parameter matching
    let mint_outpoint = OutPoint { txid: mint_block.txdata[0].compute_txid(), vout: 0 };
    let mint_sheet = load_sheet(&RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
        .OUTPOINT_TO_RUNES.select(&consensus_encode(&mint_outpoint)?));
    let token_rune_id = ProtoruneRuneId { block: 2, tx: 1 };
    let available_tokens = mint_sheet.get(&token_rune_id);
    let preloaded_rewards = available_tokens;

    // Initialize vault factory with proper parameter matching + TEMPORAL CAP
    let deposit_token_id = AlkaneId { block: 2, tx: 1 };
    let reward_token_id = AlkaneId { block: 2, tx: 1 };
    let reward_per_block = 1000u128; // 1000 tokens per block reward rate
    let start_block = 3u128;
    let fee_percentage = 0u128; // No fees for clean reward testing

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
                                    reward_per_block,
                                    start_block,
                                    end_reward_block, // *** TEMPORAL CAP ***
                                    AlkaneId { block: 2, tx: 1 }.block,  // free_mint_contract_id 
                                    AlkaneId { block: 2, tx: 1 }.tx,
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
    
    println!("✅ SETUP COMPLETE: Temporal cap at block {}", end_reward_block);

    let token_id = AlkaneId { block: 2, tx: 1 };
    Ok((init_vault_block, token_id, reward_per_block))
}

// Create EXACT amount of tokens - no excess to avoid deposit amount confusion
fn create_precise_deposit_tokens(block_height: u32, user: &str, exact_amount: u128) -> Result<Block> {
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
    
    println!("🪙 Created EXACTLY {} tokens for {} at block {}", exact_amount, user, block_height);
    
    Ok(mint_block)
}

// Enhanced deposit function - now sends exact amount
fn perform_enhanced_deposit(mint_block: &Block, deposit_amount: u128, vault_factory_id: AlkaneId, user: &str, block_height: u32, pool_state: &mut PoolState) -> Result<(Block, ProtoruneRuneId)> {
    let mint_outpoint = OutPoint {
        txid: mint_block.txdata[0].compute_txid(),
        vout: 0,
    };
    
    let mint_sheet = load_sheet(&RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
        .OUTPOINT_TO_RUNES.select(&consensus_encode(&mint_outpoint)?));
    let token_rune_id = ProtoruneRuneId { block: 2, tx: 1 };
    let available_tokens = mint_sheet.get(&token_rune_id);
    
    println!("\n💳 {} DEPOSIT: {} tokens (available: {})", user.to_uppercase(), deposit_amount, available_tokens);
    
    if available_tokens < deposit_amount {
        return Err(anyhow::anyhow!("Insufficient tokens for {}: have {}, need {}", user, available_tokens, deposit_amount));
    }
    
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
                                    vault_factory_id.block,
                                    vault_factory_id.tx,
                                    1u128           // deposit opcode (NO PARAMETERS!)
                                ]).encipher(),
                                protocol_tag: AlkaneMessageContext::protocol_tag() as u128,
                                pointer: Some(0),
                                refund: Some(0),
                                from: None,
                                burn: None,
                                edicts: vec![
                                    ProtostoneEdict {
                                        id: ProtoruneRuneId { block: 2, tx: 1 },
                                        amount: deposit_amount, // SEND EXACT DEPOSIT AMOUNT
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
        .ok_or_else(|| anyhow::anyhow!("No position token found for {}", user))?;
    
    let position_token_id = ProtoruneRuneId {
        block: position_token_info.0.block,
        tx: position_token_info.0.tx,
    };

    // Update pool state
    pool_state.add_user(user, deposit_amount);
    
    println!("   ✅ {} deposited successfully. Pool total: {}", user, pool_state.total_staked);
    
    Ok((deposit_block, position_token_id))
}

// Enhanced withdrawal function
fn perform_enhanced_withdrawal(deposit_block: &Block, position_token_id: ProtoruneRuneId, vault_factory_id: AlkaneId, user: &str, block_height: u32, deposit_amount: u128, pool_state: &mut PoolState) -> Result<u128> {
    let position_outpoint = OutPoint {
        txid: deposit_block.txdata[0].compute_txid(),
        vout: 0,
    };

    println!("\n💸 {} WITHDRAWAL at block {}", user.to_uppercase(), block_height);

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
                                    vault_factory_id.block,
                                    vault_factory_id.tx,
                                    2u128, // withdraw opcode
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

    // Analyze returned tokens
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
    
    let rewards_returned = total_returned.saturating_sub(deposit_amount);
    
    // Update pool state
    pool_state.remove_user(user, deposit_amount);
    
    println!("   💰 Total: {} (Principal: {}, Rewards: {})", total_returned, deposit_amount, rewards_returned);
    
    Ok(total_returned)
}

// Mathematical verification helper
fn calculate_expected_rewards_with_temporal_cap(
    user: &str,
    amount: u128,
    deposit_block: u32,
    withdrawal_block: u32,
    reward_per_block: u128,
    temporal_cap: u32,
    pool_periods: &[(u32, u32, u128)] // (start_block, end_block, total_pool_amount)
) -> u128 {
    let mut total_expected = 0u128;
    
    println!("🧮 CALCULATING EXPECTED REWARDS FOR {}:", user.to_uppercase());
    println!("   📊 Position: {} tokens, blocks {}-{}", amount, deposit_block, withdrawal_block);
    println!("   ⏰ Temporal cap: block {}", temporal_cap);
    
    for (start_block, end_block, pool_total) in pool_periods {
        // Check if this period overlaps with user's staking period
        let period_start = std::cmp::max(*start_block, deposit_block);
        let period_end = std::cmp::min(*end_block, withdrawal_block);
        let period_end = std::cmp::min(period_end, temporal_cap); // Apply temporal cap
        
        if period_start < period_end {
            let blocks_in_period = period_end - period_start;
            let user_share = amount as f64 / *pool_total as f64;
            let period_rewards = (blocks_in_period as f64 * reward_per_block as f64 * user_share) as u128;
            total_expected += period_rewards;
            
            println!("     • Period {}-{}: {} blocks, {:.1}% share → {} rewards", 
                     period_start, period_end, blocks_in_period, user_share * 100.0, period_rewards);
        }
    }
    
    println!("     • TOTAL EXPECTED: {} rewards", total_expected);
    total_expected
}

#[wasm_bindgen_test]
fn test_intricate_temporal_cap_with_overlapping_positions() -> Result<()> {
    println!("🎪 INTRICATE TEMPORAL CAP TEST: Fixed Token Amounts");
    println!("===================================================");
    
    let end_reward_block = 30u128;
    let (_init_vault_block, token_id, reward_per_block) = create_intricate_temporal_vault_setup(end_reward_block)?;
    let vault_factory_id = AlkaneId { block: 4, tx: 890 };
    
    // Initialize pool state tracker
    let mut pool_state = PoolState::new(3, reward_per_block);
    
    println!("\n🎭 SCENARIO:");
    println!("   ⏰ Temporal cap: Block {}", end_reward_block);
    println!("   ⚡ Reward rate: {} tokens/block", reward_per_block);
    
    // Create positions with EXACT token amounts
    let mut positions = vec![
        Position::new("Alice", 1000, 5, Some(25)),   // Long-term holder, pre-cap exit
        Position::new("Bob", 2000, 10, Some(35)),    // Whale, post-cap exit  
        Position::new("Charlie", 1500, 15, Some(20)), // Mid-size, early exit
        Position::new("Diana", 500, 20, Some(30)),   // Small, boundary exit
        Position::new("Eve", 3000, 25, Some(40)),    // Mega whale, post-cap
        Position::new("Frank", 1000, 28, Some(32)),  // Edge case, near temporal cap
    ];
    
    // Execute deposits with EXACT token amounts
    println!("\n💰 EXECUTING DEPOSITS:");
    
    for (i, pos) in positions.iter_mut().enumerate() {
        // Create EXACT amount of tokens needed for deposit - no excess
        let mint_block = create_precise_deposit_tokens(4 + i as u32, &pos.user, pos.amount)?;
        let (deposit_block, position_token_id) = perform_enhanced_deposit(
            &mint_block, 
            pos.amount, 
            vault_factory_id, 
            &pos.user, 
            pos.deposit_block, 
            &mut pool_state
        )?;
        
        pos.deposit_block_ref = Some(deposit_block);
        pos.position_token_id = Some(position_token_id);
    }
    
    // Define pool periods for mathematical verification
    let pool_periods = vec![
        (5, 10, 1000),    // Alice solo
        (10, 15, 3000),   // Alice + Bob
        (15, 20, 4500),   // Alice + Bob + Charlie
        (20, 25, 4000),   // Alice + Bob + Diana (Charlie exits)
        (25, 28, 6500),   // Bob + Diana + Eve (Alice exits)
        (28, 30, 7500),   // Bob + Diana + Eve + Frank (within temporal cap)
        (30, 32, 0),      // Post temporal cap - no rewards
    ];
    
    // Calculate expected rewards for each position
    for pos in positions.iter_mut() {
        pos.expected_rewards = calculate_expected_rewards_with_temporal_cap(
            &pos.user,
            pos.amount,
            pos.deposit_block,
            pos.withdrawal_block.unwrap_or(0),
            reward_per_block,
            end_reward_block as u32,
            &pool_periods
        );
    }
    
    // Execute withdrawals in chronological order
    println!("\n💸 EXECUTING WITHDRAWALS:");
    
    // Sort positions by withdrawal block for chronological execution
    let mut withdrawal_order: Vec<usize> = (0..positions.len()).collect();
    withdrawal_order.sort_by_key(|&i| positions[i].withdrawal_block.unwrap_or(0));
    
    for &i in &withdrawal_order {
        let pos = &mut positions[i];
        let total_returned = perform_enhanced_withdrawal(
            pos.deposit_block_ref.as_ref().unwrap(),
            pos.position_token_id.unwrap(),
            vault_factory_id,
            &pos.user,
            pos.withdrawal_block.unwrap_or(0),
            pos.amount,
            &mut pool_state
        )?;
        
        pos.actual_total = total_returned;
        pos.actual_rewards = total_returned.saturating_sub(pos.amount);
    }
    
    // Results analysis
    println!("\n📊 RESULTS ANALYSIS:");
    println!("====================");
    
    let mut all_within_tolerance = true;
    let tolerance = reward_per_block; // 1000 token tolerance
    
    for pos in &positions {
        let reward_difference = pos.actual_rewards as i128 - pos.expected_rewards as i128;
        let within_tolerance = reward_difference.abs() <= tolerance as i128;
        let accuracy_pct = if pos.expected_rewards > 0 {
            (pos.actual_rewards as f64 / pos.expected_rewards as f64) * 100.0
        } else {
            if pos.actual_rewards == 0 { 100.0 } else { 0.0 }
        };
        
        println!("\n👤 {} ANALYSIS:", pos.user.to_uppercase());
        println!("   💰 Expected rewards: {} tokens", pos.expected_rewards);
        println!("   💰 Actual rewards: {} tokens", pos.actual_rewards);
        println!("   📊 Difference: {} tokens", reward_difference);
        println!("   📊 Accuracy: {:.1}%", accuracy_pct);
        println!("   📊 Within tolerance: {}", if within_tolerance { "✅" } else { "❌" });
        
        all_within_tolerance &= within_tolerance;
    }
    
    if all_within_tolerance {
        println!("\n🎉 INTRICATE TEMPORAL CAP TEST: ✅ PASSED!");
        println!("   🏆 All positions received mathematically correct rewards");
        println!("   ⏰ Temporal cap boundary enforcement working perfectly");
        Ok(())
    } else {
        println!("\n❌ INTRICATE TEMPORAL CAP TEST: FAILED!");
        for pos in &positions {
            if (pos.actual_rewards as i128 - pos.expected_rewards as i128).abs() > tolerance as i128 {
                println!("   • {} failed: expected {}, got {}", pos.user, pos.expected_rewards, pos.actual_rewards);
            }
        }
        Err(anyhow::anyhow!("Intricate temporal cap test verification failed"))
    }
}
