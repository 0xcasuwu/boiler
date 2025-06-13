use metashrew_support::compat::to_arraybuffer_layout;
use alkanes_support::context::Context;

use alkanes_runtime::{
  declare_alkane, message::MessageDispatch, token::Token,
  runtime::AlkaneResponder
};

use alkanes_support::{
  cellpack::Cellpack, id::AlkaneId,
  parcel::{AlkaneTransfer, AlkaneTransferParcel}, response::CallResponse
};

use anyhow::{anyhow, Result};

/// Position token template ID
const POSITION_TOKEN_TEMPLATE_ID: u128 = 0x379;

#[derive(Default)]
pub struct VaultFactory(());

impl AlkaneResponder for VaultFactory {}

#[derive(MessageDispatch)]
enum VaultFactoryMessage {
  #[opcode(0)]
  Initialize {
    deposit_token_id: AlkaneId,    // Token users must deposit (e.g., USDC)
    reward_token_id: AlkaneId,     // Token for rewards (can be same or different)
    reward_per_block: u128,        // Emission rate per block
    start_block: u128,             // When rewards begin
    preloaded_rewards: u128,       // Total reward pool loaded at init
  },
  

  #[opcode(1)]
  Deposit {
    assets: u128,
  },

  #[opcode(2)]
  Withdraw,


  #[opcode(10)]
  #[returns(u128)]
  GetTotalAssets,

  #[opcode(20)]
  #[returns(u128)]
  CalculateRewards {
    amount: u128,
    from_block: u128,
    to_block: u128,
  },
}

impl Token for VaultFactory {
  fn name(&self) -> String {
    String::from("Vault Factory")
  }

  fn symbol(&self) -> String {
    String::from("VAULT")
  }
}

impl VaultFactory {
  fn initialize(&self, deposit_token_id: AlkaneId, reward_token_id: AlkaneId, reward_per_block: u128, start_block: u128, preloaded_rewards: u128) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::default(); // Don't forward all tokens - process selectively
    
    self.observe_initialization()?;
    
    // Validate preloaded rewards
    if preloaded_rewards == 0 {
      return Err(anyhow!("Must preload reward pool with tokens"));
    }
    
    // CRITICAL: Validate that reward tokens were actually sent to vault
    let mut reward_tokens_received = 0u128;
    let mut _auth_tokens_to_return: Vec<AlkaneTransfer> = Vec::new();
    
    for transfer in &context.incoming_alkanes.0 {
      if transfer.id.block == reward_token_id.block && transfer.id.tx == reward_token_id.tx {
        reward_tokens_received += transfer.value;
      }
    }
    
    // Ensure exactly the preloaded amount was provided
    if reward_tokens_received != preloaded_rewards {
      return Err(anyhow!("Reward token amount ({}) doesn't match preloaded_rewards parameter ({})", reward_tokens_received, preloaded_rewards));
    }
    
    // Store all parameters
    self.set_deposit_token_id(&deposit_token_id)?;
    self.set_reward_token_id(&reward_token_id)?;
    self.set_reward_per_block(reward_per_block);
    self.set_start_block(start_block);
    
    // Initialize reward pool tracking
    self.set_total_reward_pool(preloaded_rewards);
    self.set_distributed_rewards(0);
    
    // Initialize counters
    self.set_position_count(0);
    self.set_total_assets(0);
    self.set_last_update_block(u128::from(self.height()));
    
    // PURE MASTERCHEF: Initialize global accumulator
    self.set_acc_reward_per_share(0);
    self.set_last_reward_block(start_block);
    
    // Factory token acts as auth token
    response.alkanes.0.push(AlkaneTransfer {
      id: context.myself.clone(),
      value: 1u128,
    });
  
    
    Ok(response)
  }
  
  fn deposit(&self, assets: u128) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::default();
    
    if assets == 0 {
      return Err(anyhow!("Cannot deposit zero assets"));
    }

    // SECURITY: Minimum deposit requirement to prevent precision exploits
    // Small deposits can cause reward_debt calculation errors leading to reward theft
    const MINIMUM_DEPOSIT: u128 = 100;
    if assets < MINIMUM_DEPOSIT {
      return Err(anyhow!("Minimum deposit is {} tokens (provided: {}). This prevents precision exploits in reward calculations.", MINIMUM_DEPOSIT, assets));
    }

    // Get the deposit token info first
    let deposit_token = &context.incoming_alkanes.0[0];
    
    // CRITICAL: Validate that the deposit token matches the expected deposit_token_id
    let expected_deposit_token_id = self.deposit_token_id()?;
    if deposit_token.id.block != expected_deposit_token_id.block || 
       deposit_token.id.tx != expected_deposit_token_id.tx {
      return Err(anyhow!("Invalid deposit token - expected AlkaneId {{ block: {}, tx: {} }}, got AlkaneId {{ block: {}, tx: {} }}", 
                        expected_deposit_token_id.block, expected_deposit_token_id.tx,
                        deposit_token.id.block, deposit_token.id.tx));
    }
    // SECURITY: Precise deposit validation - sent amount must equal intended deposit amount
    // This prevents users from accidentally sending more tokens than they intend to deposit
    // if deposit_token.value != assets {
    //   return Err(anyhow!("Sent token amount ({}) must exactly equal deposit amount ({}). Cannot send more or less than intended deposit.", 
    //                     deposit_token.value, assets));
    // }
    
    // NEW: REWARD POOL EXHAUSTION CHECK
    // Check if reward pool has sufficient rewards to support this deposit
    let remaining_rewards = self.remaining_rewards();
    
    if remaining_rewards == 0 {
      return Err(anyhow!("Reward pool exhausted - no rewards available for new deposits"));
    }
    
    // PURE MASTERCHEF: Update global rewards before changing total assets
    self.update_rewards();
        
    // PURE MASTERCHEF: 1:1 deposit ratio (no conversion)
    let deposit_amount = assets;
    
    // PURE MASTERCHEF: Calculate reward debt for this position with overflow protection
    let current_acc_reward_per_share = self.acc_reward_per_share();
    let precision = 1_000_000_000_000u128; // 10^12 precision
    let reward_debt = deposit_amount
        .checked_mul(current_acc_reward_per_share)
        .and_then(|x| x.checked_div(precision))
        .unwrap_or(0);
    
    // Update total assets (no total_shares in pure MasterChef)
    let new_total_assets = self.total_assets().checked_add(deposit_amount)
      .ok_or_else(|| anyhow!("Total assets overflow"))?;
    self.set_total_assets(new_total_assets);
    
    // CRITICAL: Hold the deposit tokens in the vault (don't forward them - vault keeps them)
    // The vault now holds these tokens to return them later during withdrawal
    
    // Create a new position token
    let position_id = self.position_count();
    let next_position_id = position_id.checked_add(1)
      .ok_or_else(|| anyhow!("Position count overflow"))?;
    
    // Get current block height for position creation
    let current_block = u128::from(self.height());
    
    // Get the deposit token from the incoming transfer
    let deposit_token_id = context.incoming_alkanes.0[0].id.clone();
    
    let cellpack = Cellpack {
      target: AlkaneId {
        block: 6,
        tx: POSITION_TOKEN_TEMPLATE_ID,
      },
      // PURE MASTERCHEF: Include reward_debt in position token creation (simplified inputs)
      inputs: vec![0x0, position_id, deposit_amount, reward_debt, current_block, deposit_token_id.block, deposit_token_id.tx],
    };
    
    // Position token receives NO underlying assets. It's purely an authentication and tracking token.
    let position_parcel = AlkaneTransferParcel::default();
    
    let create_response = self.call(&cellpack, &position_parcel, self.fuel())?;
    
    if create_response.alkanes.0.len() < 1 {
      return Err(anyhow!("Position token not returned by factory"));
    }
    
    // Get the position token (only one token returned now)
    let position_token = create_response.alkanes.0[0].clone();
    
    // SECURITY CRITICAL: Register this position token as our child
    self.register_child(&position_token.id);
    
    // Update the position counter for sequential IDs
    self.set_position_count(next_position_id);
    
    // Return the position token to the user
    response.alkanes.0.push(position_token);
    
    Ok(response)
  }
  
  // Helper function to query position details from the position token (PURE MASTERCHEF)
  fn get_position_details(&self, position_alkane: &AlkaneId) -> Result<(u128, u128, u128, u128)> {
    // Single call to get all position details at once (simplified for pure MasterChef)
    let cellpack = Cellpack {
      target: position_alkane.clone(),
      inputs: vec![0x17],  // 0x17 = GetAllDetails opcode (23 in decimal)
    };
    
    let response = self.staticcall(&cellpack, &AlkaneTransferParcel::default(), self.fuel())?;
    
    // The data contains 4 u128 values, each 16 bytes (pure MasterChef essentials)
    if response.data.len() < 16 * 4 {
      return Err(anyhow!("Invalid response from position token - insufficient data length"));
    }
    
    // Extract each value from the packed data with proper error handling
    let position_id = u128::from_le_bytes(
      response.data[0..16].try_into()
        .map_err(|_| anyhow!("Failed to parse position_id from position token response"))?
    );
    let deposit_amount = u128::from_le_bytes(
      response.data[16..32].try_into()
        .map_err(|_| anyhow!("Failed to parse deposit_amount from position token response"))?
    );
    let reward_debt = u128::from_le_bytes(
      response.data[32..48].try_into()
        .map_err(|_| anyhow!("Failed to parse reward_debt from position token response"))?
    );
    let deposit_block = u128::from_le_bytes(
      response.data[48..64].try_into()
        .map_err(|_| anyhow!("Failed to parse deposit_block from position token response"))?
    );
    
    Ok((position_id, deposit_amount, reward_debt, deposit_block))
  }

  fn withdraw(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::default();
    
    // Verify the caller is a valid position token
    self.authenticate_position(&context)?;
    
    // PURE MASTERCHEF: Update global rewards before calculating user rewards
    self.update_rewards();
    
    // Get the position token from context - this IS the authentication
    let position_token = &context.incoming_alkanes.0[0];
    let position_alkane = &position_token.id;
    
    // Query position token for its current state including reward debt
    let position_details = self.get_position_details(position_alkane)?;
    let (_position_id, deposit_amount, reward_debt, _deposit_block) = position_details;
    
    // PURE MASTERCHEF REWARD CALCULATION WITH OVERFLOW PROTECTION: 
    // pending_rewards = (deposit_amount * accRewardPerShare / 1e12) - rewardDebt
    let current_acc_reward_per_share = self.acc_reward_per_share();
    let precision = 1_000_000_000_000u128; // 10^12 precision
    
    // CRITICAL FIX: Use checked arithmetic to prevent overflow corruption
    let accumulated_rewards = deposit_amount
        .checked_mul(current_acc_reward_per_share)
        .and_then(|x| x.checked_div(precision))
        .unwrap_or(0);
    let pending_rewards = accumulated_rewards.saturating_sub(reward_debt);
    
    // CRITICAL: Real-time pool validation to prevent over-distribution
    let remaining_pool = self.remaining_rewards();
    let actual_rewards = std::cmp::min(pending_rewards, remaining_pool);
    
    // Update distributed_rewards with ACTUAL payout only
    let new_distributed = self.distributed_rewards() + actual_rewards;
    self.set_distributed_rewards(new_distributed);
    
    // Update vault state - remove deposit amount (pure 1:1 MasterChef)
    let new_total_assets = self.total_assets()
      .checked_sub(deposit_amount)
      .ok_or_else(|| anyhow!("Insufficient total assets"))?;
    self.set_total_assets(new_total_assets);
    
    // Get the original deposit token ID
    let deposit_token_id_cellpack = Cellpack {
      target: position_alkane.clone(),
      inputs: vec![0x18], // 0x18 = GetDepositTokenId opcode
    };
    
    let deposit_token_response = self.staticcall(&deposit_token_id_cellpack, &AlkaneTransferParcel::default(), self.fuel())?;
    
    if deposit_token_response.data.len() < 32 {
      return Err(anyhow!("Invalid deposit token ID response - insufficient data length"));
    }
    
    let deposit_token_id = AlkaneId {
      block: u128::from_le_bytes(
        deposit_token_response.data[0..16].try_into()
          .map_err(|_| anyhow!("Failed to parse deposit token block ID from position token response"))?
      ),
      tx: u128::from_le_bytes(
        deposit_token_response.data[16..32].try_into()
          .map_err(|_| anyhow!("Failed to parse deposit token tx ID from position token response"))?
      ),
    };
    
    // PURE MASTERCHEF: Return original deposit (1:1) + capped rewards
    if deposit_amount > 0 {
      response.alkanes.0.push(AlkaneTransfer {
        id: deposit_token_id.clone(),
        value: deposit_amount,  // Always 1:1 principal return
      });
    }
    
    // Return actual rewards (capped to available pool)
    if actual_rewards > 0 {
      let reward_token_id = self.reward_token_id()?;
      response.alkanes.0.push(AlkaneTransfer {
        id: reward_token_id,
        value: actual_rewards,  // Pool-validated reward distribution
      });
    }
    
    Ok(response)
  }
  
  fn authenticate_position(&self, context: &Context) -> Result<()> {
    // Validate incoming alkanes structure
    if context.incoming_alkanes.0.is_empty() {
      return Err(anyhow!("No incoming alkanes for position authentication"));
    }
    
    // The value should be at least 1
    let transfer = &context.incoming_alkanes.0[0];
    if transfer.value < 1 {
      return Err(anyhow!("Less than 1 unit of token supplied"));
    }
    
    // SECURITY CRITICAL: Verify this position token was created by US FIRST
    // This prevents calling malicious contracts that could cause panics
    if !self.is_registered_child(&transfer.id) {
      return Err(anyhow!("Position token not our registered child - potential spoofing attack"));
    }
    
    // SECONDARY: Only query position for additional validation if it passed registry check
    // This is safe because we know it's our registered child
    match self.get_position_details(&transfer.id) {
      Ok((_position_id, _deposit_amount, _reward_debt, _deposit_block)) => {
        // Position token is registered child and responded correctly
        Ok(())
      }
      Err(e) => {
        // Even registered children should respond properly
        Err(anyhow!("Registered position token failed to provide details: {}", e))
      }
    }
  }
  
  fn get_total_assets(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);
    response.data = self.total_assets().to_le_bytes().to_vec();
    Ok(response)
  }
  
  fn calculate_rewards(&self, amount: u128, from_block: u128, to_block: u128) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);
    
    // Don't calculate rewards before start_block
    let effective_from = std::cmp::max(from_block, self.start_block());
    
    // Don't calculate beyond current block
    let current_block = u128::from(self.height());
    let effective_to = std::cmp::min(to_block, current_block);
    
    let rewards = if effective_from >= effective_to {
      0
    } else {
      // Simple reward calculation: amount * reward_per_block * blocks_elapsed / precision
      let blocks_elapsed = effective_to - effective_from;
      let precision = 1_000_000_000_000u128; // 10^12 precision
      
      amount
        .checked_mul(self.reward_per_block())
        .unwrap_or(0)
        .checked_mul(blocks_elapsed)
        .unwrap_or(0)
        .checked_div(precision)
        .unwrap_or(0)
    };
    
    response.data = rewards.to_le_bytes().to_vec();
    Ok(response)
  }
  
  // Storage operations using direct store/load methods
  
  fn reward_per_block(&self) -> u128 {
    self.load_u128("/reward_per_block")
  }
  
  fn set_reward_per_block(&self, reward_per_block: u128) {
    self.store("/reward_per_block".as_bytes().to_vec(), reward_per_block.to_le_bytes().to_vec());
  }
  
  fn start_block(&self) -> u128 {
    self.load_u128("/start_block")
  }
  
  fn set_start_block(&self, start_block: u128) {
    self.store("/start_block".as_bytes().to_vec(), start_block.to_le_bytes().to_vec());
  }
    
  fn total_assets(&self) -> u128 {
    self.load_u128("/total_assets")
  }
  
  fn set_total_assets(&self, total_assets: u128) {
    self.store("/total_assets".as_bytes().to_vec(), total_assets.to_le_bytes().to_vec());
  }
  
  fn last_update_block(&self) -> u128 {
    self.load_u128("/last_update_block")
  }
  
  fn set_last_update_block(&self, last_update_block: u128) {
    self.store("/last_update_block".as_bytes().to_vec(), last_update_block.to_le_bytes().to_vec());
  }
  
  fn position_count(&self) -> u128 {
    self.load_u128("/position_count")
  }
  
  fn set_position_count(&self, position_count: u128) {
    self.store("/position_count".as_bytes().to_vec(), position_count.to_le_bytes().to_vec());
  }
  
  fn reward_token_id(&self) -> Result<AlkaneId> {
    let bytes = self.load("/reward_token_id".as_bytes().to_vec());
    
    if bytes.len() < 32 {
      return Err(anyhow!("Reward token ID not set"));
    }
    
    Ok(AlkaneId {
      block: u128::from_le_bytes(
        bytes[0..16].try_into()
          .map_err(|_| anyhow!("Failed to parse reward token block ID from storage"))?
      ),
      tx: u128::from_le_bytes(
        bytes[16..32].try_into()
          .map_err(|_| anyhow!("Failed to parse reward token tx ID from storage"))?
      ),
    })
  }
  
  fn set_reward_token_id(&self, id: &AlkaneId) -> Result<()> {
    let mut bytes = Vec::with_capacity(32);
    bytes.extend_from_slice(&id.block.to_le_bytes());
    bytes.extend_from_slice(&id.tx.to_le_bytes());
    
    self.store("/reward_token_id".as_bytes().to_vec(), bytes);
    Ok(())
  }

  // Helper function to load u128 values from storage
  fn load_u128(&self, key_str: &str) -> u128 {
    let key = key_str.as_bytes().to_vec();
    let bytes = self.load(key);
    if bytes.len() >= 16 {
      let bytes_array: [u8; 16] = bytes[0..16].try_into().unwrap_or([0; 16]);
      u128::from_le_bytes(bytes_array)
    } else {
      0
    }
  }
  
  fn deposit_token_id(&self) -> Result<AlkaneId> {
    let bytes = self.load("/deposit_token_id".as_bytes().to_vec());
    
    if bytes.len() < 32 {
      return Err(anyhow!("Deposit token ID not set"));
    }
    
    Ok(AlkaneId {
      block: u128::from_le_bytes(
        bytes[0..16].try_into()
          .map_err(|_| anyhow!("Failed to parse deposit token block ID from storage"))?
      ),
      tx: u128::from_le_bytes(
        bytes[16..32].try_into()
          .map_err(|_| anyhow!("Failed to parse deposit token tx ID from storage"))?
      ),
    })
  }
  
  fn set_deposit_token_id(&self, id: &AlkaneId) -> Result<()> {
    let mut bytes = Vec::with_capacity(32);
    bytes.extend_from_slice(&id.block.to_le_bytes());
    bytes.extend_from_slice(&id.tx.to_le_bytes());
    
    self.store("/deposit_token_id".as_bytes().to_vec(), bytes);
    Ok(())
  }
  
  fn total_reward_pool(&self) -> u128 {
    self.load_u128("/total_reward_pool")
  }
  
  fn set_total_reward_pool(&self, total_reward_pool: u128) {
    self.store("/total_reward_pool".as_bytes().to_vec(), total_reward_pool.to_le_bytes().to_vec());
  }
  
  fn distributed_rewards(&self) -> u128 {
    self.load_u128("/distributed_rewards")
  }
  
  fn set_distributed_rewards(&self, distributed_rewards: u128) {
    self.store("/distributed_rewards".as_bytes().to_vec(), distributed_rewards.to_le_bytes().to_vec());
  }
  
  fn remaining_rewards(&self) -> u128 {
    let total_pool = self.total_reward_pool();
    let distributed = self.distributed_rewards();
    total_pool.saturating_sub(distributed)
  }

  
  fn is_registered_child(&self, child_id: &AlkaneId) -> bool {
    let key = format!("/registered_children/{}_{}", child_id.block, child_id.tx).into_bytes();
    let bytes = self.load(key);
    !bytes.is_empty() && bytes[0] == 1
  }
  
  fn register_child(&self, child_id: &AlkaneId) {
    let key = format!("/registered_children/{}_{}", child_id.block, child_id.tx).into_bytes();
    self.store(key, vec![1u8]);
  }
  
  // PURE MASTERCHEF REWARD-PER-SHARE STORAGE FUNCTIONS
  
  fn acc_reward_per_share(&self) -> u128 {
    self.load_u128("/acc_reward_per_share")
  }
  
  fn set_acc_reward_per_share(&self, acc_reward_per_share: u128) {
    self.store("/acc_reward_per_share".as_bytes().to_vec(), acc_reward_per_share.to_le_bytes().to_vec());
  }
  
  fn last_reward_block(&self) -> u128 {
    self.load_u128("/last_reward_block")
  }
  
  fn set_last_reward_block(&self, last_reward_block: u128) {
    self.store("/last_reward_block".as_bytes().to_vec(), last_reward_block.to_le_bytes().to_vec());
  }
  
  fn update_rewards(&self) {
    let current_block = u128::from(self.height());
    let last_reward_block = self.last_reward_block();
    let total_assets = self.total_assets();
    
    // Skip if already updated this block
    if current_block <= last_reward_block {
      return;
    }
    
    // PURE MASTERCHEF: If no assets exist, only update last_reward_block to current
    // This prevents accumulating rewards for periods with no stakers
    if total_assets == 0 {
      self.set_last_reward_block(current_block);
      return;
    }
    
    // CRITICAL MASTERCHEF ARCHITECTURE FIX:
    // The accumulator MUST track theoretical rewards without pool limits
    // Pool limits are only applied during withdrawal/harvest, NOT during accumulation
    // This is fundamental to how SushiSwap MasterChef V2 works
    
    // 1. Calculate blocks elapsed with overflow protection
    let blocks_elapsed = current_block - last_reward_block;
    let reward_per_block = self.reward_per_block();
    
    // 2. Calculate theoretical period rewards (uncapped - critical for MasterChef math)
    let theoretical_period_rewards = blocks_elapsed
        .checked_mul(reward_per_block)
        .unwrap_or(0);
    
    // 3. Update accumulator with THEORETICAL rewards (SushiSwap MasterChef pattern)
    // accRewardPerShare += (theoreticalPeriodRewards * 1e12) / totalStaked
    let precision = 1_000_000_000_000u128; // 1e12
    let additional_acc_reward_per_share = theoretical_period_rewards
      .checked_mul(precision)
      .and_then(|x| x.checked_div(total_assets))
      .unwrap_or(0);
    
    let new_acc_reward_per_share = self.acc_reward_per_share()
      .checked_add(additional_acc_reward_per_share)
      .unwrap_or(self.acc_reward_per_share());
    
    // 4. Update state - accumulator tracks theoretical, pool limits applied at withdrawal
    self.set_acc_reward_per_share(new_acc_reward_per_share);
    self.set_last_reward_block(current_block);
  }
}

declare_alkane! {
  impl AlkaneResponder for VaultFactory {
    type Message = VaultFactoryMessage;
  }
}
