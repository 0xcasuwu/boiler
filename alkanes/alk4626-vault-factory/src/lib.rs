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
    reward_per_block: u128,
    start_block: u128,
    reward_token_id: AlkaneId,
    fee_percentage: u128,
  },
  
  #[opcode(4)]
  WithdrawFees,

  #[opcode(1)]
  Deposit {
    assets: u128,
  },

  #[opcode(2)]
  Withdraw {
    position_id: u128,
  },
  
  #[opcode(3)]
  ClaimRewards {
    position_id: u128,
  },
  
  #[opcode(99)]
  #[returns(u128)]
  TestPing,

  #[opcode(10)]
  #[returns(u128)]
  GetTotalAssets,

  #[opcode(11)]
  #[returns(u128)]
  GetTotalShares,

  #[opcode(12)]
  #[returns(u128)]
  GetPositionCount,

  #[opcode(13)]
  #[returns(AlkaneId)]
  GetPositionById {
    position_id: u128,
  },

  #[opcode(20)]
  #[returns(u128)]
  CalculateRewards {
    amount: u128,
    from_block: u128,
    to_block: u128,
  },

  #[opcode(21)]
  #[returns(u128)]
  ConvertToShares {
    assets: u128,
  },

  #[opcode(22)]
  #[returns(u128)]
  ConvertToAssets {
    shares: u128,
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
  fn initialize(&self, reward_per_block: u128, start_block: u128, reward_token_id: AlkaneId, fee_percentage: u128) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);
    
    self.observe_initialization()?;
    
    // Store initial parameters
    self.set_reward_per_block(reward_per_block);
    self.set_start_block(start_block);
    self.set_reward_token_id(&reward_token_id)?;
    
    // Validate and store fee percentage (in basis points, 1% = 100)
    if fee_percentage > 10000 { // Max fee is 100%
      return Err(anyhow!("Fee percentage cannot exceed 10000 (100%)"));
    }
    self.set_fee_percentage(fee_percentage);
    self.set_owner(&context.caller);
    
    // Initialize counters
    self.set_position_count(0);
    self.set_total_assets(0);
    self.set_total_shares(0);
    self.set_last_update_block(u128::from(self.height()));
    self.set_collected_fees(0);
    
    // Factory token acts as auth token
    response.alkanes.0.push(AlkaneTransfer {
      id: context.myself.clone(),
      value: 10u128,
    });
    
    Ok(response)
  }
  
  fn deposit(&self, assets: u128) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::default();
    
    // check if deposit token matches expected underlying defined in initialization
    if assets == 0 {
      return Err(anyhow!("Cannot deposit zero assets"));
    }
    
    // Verify that a deposit token is provided
    if context.incoming_alkanes.0.len() != 1 {
      return Err(anyhow!("Expected exactly one token type for deposit"));
    }
    
    // Get the deposit token info
    let deposit_token = &context.incoming_alkanes.0[0];
    if deposit_token.value < assets {
      return Err(anyhow!("Insufficient token value for deposit amount"));
    }
    
    // Calculate 5% deposit fee
    let fee_percentage = self.fee_percentage();
    let fee_amount = assets
      .checked_mul(fee_percentage)
      .unwrap_or(0)
      .checked_div(10000)
      .unwrap_or(0);
    
    // Amount after fee (what actually gets deposited)
    let assets_after_fee = assets.checked_sub(fee_amount).unwrap_or(0);
    
    // Add fee to collected fees
    let new_collected_fees = self.collected_fees().checked_add(fee_amount).unwrap_or(self.collected_fees());
    self.set_collected_fees(new_collected_fees);
    
    // Calculate shares based on assets after fee
    let shares = self.convert_to_shares_internal(assets_after_fee)?;
    if shares == 0 {
      return Err(anyhow!("Deposit would result in zero shares"));
    }
    
    // Update total assets with amount after fee (fee stays in vault)
    let new_total_assets = self.total_assets().checked_add(assets_after_fee)
      .ok_or_else(|| anyhow!("Total assets overflow"))?;
    self.set_total_assets(new_total_assets);
    
    let new_total_shares = self.total_shares().checked_add(shares)
      .ok_or_else(|| anyhow!("Total shares overflow"))?;
    self.set_total_shares(new_total_shares);
    
    // Create a new position token
    let position_id = self.position_count();
    let next_position_id = position_id.checked_add(1)
      .ok_or_else(|| anyhow!("Position count overflow"))?;
    
    // Get current block height for position creation
    let current_block = u128::from(self.height());
    
    // Get the deposit token from the incoming transfer
    // if context.incoming_alkanes.0.len() != 1 {
    //   return Err(anyhow!("Expected exactly one token type for deposit"));
    // }
    let deposit_token_id = context.incoming_alkanes.0[0].id.clone();
    
    let cellpack = Cellpack {
      target: AlkaneId {
        block: 6,
        tx: POSITION_TOKEN_TEMPLATE_ID,
      },
      inputs: vec![0x0, position_id, assets_after_fee, shares, current_block, deposit_token_id.block, deposit_token_id.tx],
    };
    
    let create_response = self.call(&cellpack, &AlkaneTransferParcel::default(), self.fuel())?;
    
    if create_response.alkanes.0.len() < 1 {
      return Err(anyhow!("Position token not returned by factory"));
    }
    
    // Get the position token ID
    let position_token = create_response.alkanes.0[0].clone();
    
    // Add the position to our registry
    self.add_position(&position_token.id)?;
    self.set_position_count(next_position_id);
    
    // Return the position token to the user
    response.alkanes.0.push(position_token);
    
    Ok(response)
  }
  
  // Helper function to query position details from the position token
  fn get_position_details(&self, position_alkane: &AlkaneId) -> Result<(u128, u128, u128, u128, u128)> {
    // Single call to get all position details at once
    let cellpack = Cellpack {
      target: position_alkane.clone(),
      inputs: vec![0x17],  // 0x17 = GetAllDetails opcode (23 in decimal)
    };
    
    let response = self.staticcall(&cellpack, &AlkaneTransferParcel::default(), self.fuel())?;
    
    // The data contains 5 u128 values, each 16 bytes
    if response.data.len() < 16 * 5 {
      return Err(anyhow!("Invalid response from position token"));
    }
    
    // Extract each value from the packed data
    let position_id = u128::from_le_bytes(response.data[0..16].try_into().unwrap());
    let current_assets = u128::from_le_bytes(response.data[16..32].try_into().unwrap());
    let shares = u128::from_le_bytes(response.data[32..48].try_into().unwrap());
    let deposit_block = u128::from_le_bytes(response.data[48..64].try_into().unwrap());
    let last_claim_block = u128::from_le_bytes(response.data[64..80].try_into().unwrap());
    
    Ok((position_id, current_assets, shares, deposit_block, last_claim_block))
  }

  fn withdraw(&self, position_id: u128) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);
    
    // Verify the caller is a valid position token
    self.authenticate_position(&context)?;
    
    // Verify position_id exists
    let position_alkane = self.find_position_by_id(position_id)?;
    if position_alkane.block == 0 && position_alkane.tx == 0 {
      return Err(anyhow!("Position not found"));
    }
    
    // Query position token for its current state
    let (_position_id, current_assets, shares, _deposit_block, last_claim_block) = 
      self.get_position_details(&position_alkane)?;
    
    // Always withdraw the full amount from the position
    let assets = current_assets;
    
    // First process any pending rewards
    let current_block = u128::from(self.height());
    
    // Calculate rewards based on the staked amount and time period
    let rewards = if current_assets > 0 && last_claim_block < current_block {
      let blocks_elapsed = current_block.checked_sub(last_claim_block).unwrap_or(0);
      let precision = 1_000_000u128; // 10^6 precision - FIXED for meaningful rewards
      
      current_assets
        .checked_mul(self.reward_per_block())
        .unwrap_or(0)
        .checked_mul(blocks_elapsed)
        .unwrap_or(0)
        .checked_div(precision)
        .unwrap_or(0)
    } else {
      0
    };
    
    // Calculate fee amount
    let fee_percentage = self.fee_percentage();
    let fee_amount = assets
      .checked_mul(fee_percentage)
      .unwrap_or(0)
      .checked_div(10000)
      .unwrap_or(0);

    // Amount after fee - APPLY WITHDRAWAL FEES
    let withdrawal_after_fee = assets.checked_sub(fee_amount).unwrap_or(assets);
    
    // Add fee to collected fees
    let new_collected_fees = self.collected_fees().checked_add(fee_amount).unwrap_or(self.collected_fees());
    self.set_collected_fees(new_collected_fees);
    
    // Update total assets (assets is the full amount including fee)
    let new_total_assets = self.total_assets()
      .checked_sub(assets)
      .ok_or_else(|| anyhow!("Insufficient total assets"))?;
    self.set_total_assets(new_total_assets);
    
    // Calculate shares to burn - using safer arithmetic with more defensive checks
    let shares_to_burn = if current_assets > 0 && shares > 0 {
      // Add extra safety checks for overflow prevention
      if assets >= current_assets {
        // If withdrawing all or more than current assets, just use all shares
        shares
      } else {
        // When withdrawing partial amount, calculate proportional shares
        // Formula: shares_to_burn = (assets * shares) / current_assets
        let calc_result = assets
          .checked_mul(shares)
          .ok_or_else(|| anyhow!("Calculation overflow in shares_to_burn - multiplication"))?;
        
        // Add extra debug info in case of division issues
        if current_assets == 0 {
          return Err(anyhow!("Division by zero: current_assets is 0"));
        }
        
        calc_result
          .checked_div(current_assets)
          .ok_or_else(|| anyhow!("Division error in shares_to_burn calculation"))?
      }
    } else {
      // If no assets or no shares, then nothing to burn
      0
    };
    
    // Update total shares
    let new_total_shares = self.total_shares()
      .checked_sub(shares_to_burn)
      .ok_or_else(|| anyhow!("Insufficient total shares"))?;
    self.set_total_shares(new_total_shares);
    
    // Update the position token's current assets 
    let new_position_assets = current_assets.checked_sub(assets)
      .ok_or_else(|| anyhow!("Assets underflow"))?;
    
    let update_assets_cellpack = Cellpack {
      target: position_alkane.clone(),
      inputs: vec![0x5, new_position_assets], // 0x5 = UpdateCurrentAssets opcode
    };
    
    // Create authentication parcel with our token
    let mut auth_parcel = AlkaneTransferParcel::default();
    auth_parcel.0.push(AlkaneTransfer {
      id: context.myself.clone(),
      value: 1u128,
    });
    
    // Call with authentication token
    self.call(&update_assets_cellpack, &auth_parcel, self.fuel())?;
    
    // Get the original deposit token ID from the position token
    let deposit_token_id_cellpack = Cellpack {
      target: position_alkane.clone(),
      inputs: vec![0x18], // 0x18 = GetDepositTokenId opcode (24 in decimal)
    };
    
    let deposit_token_response = self.staticcall(&deposit_token_id_cellpack, &AlkaneTransferParcel::default(), self.fuel())?;
    
    if deposit_token_response.data.len() < 32 {
      return Err(anyhow!("Invalid deposit token ID response"));
    }
    
    let deposit_token_id = AlkaneId {
      block: u128::from_le_bytes(deposit_token_response.data[0..16].try_into().unwrap()),
      tx: u128::from_le_bytes(deposit_token_response.data[16..32].try_into().unwrap()),
    };
    
    // 1. Return the original deposit tokens (minus fee)
    response.alkanes.0.push(AlkaneTransfer {
      id: deposit_token_id, // Return the same token ID that was originally deposited
      value: withdrawal_after_fee,
    });
    
    // 2. Add reward tokens if there are any
    if rewards > 0 {
      let reward_token_id = self.reward_token_id()?;
      response.alkanes.0.push(AlkaneTransfer {
        id: reward_token_id,
        value: rewards,
      });
      
      // Update last claim block in position token
      let update_claim_block_cellpack = Cellpack {
        target: position_alkane,
        inputs: vec![0x4, current_block], // 0x4 = UpdateLastClaimBlock opcode
      };
      
      // Create authentication parcel with our token
      let mut auth_parcel = AlkaneTransferParcel::default();
      auth_parcel.0.push(AlkaneTransfer {
        id: context.myself.clone(),
        value: 1u128,
      });
      
      // Call with authentication token
      self.call(&update_claim_block_cellpack, &auth_parcel, self.fuel())?;
    }
    
    Ok(response)
  }
  
  fn claim_rewards(&self, position_id: u128) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);
    
    // Verify the caller is a valid position token
    self.authenticate_position(&context)?;
    
    // Verify position_id exists
    let position_alkane = self.find_position_by_id(position_id)?;
    if position_alkane.block == 0 && position_alkane.tx == 0 {
      return Err(anyhow!("Position not found"));
    }
    
    // 1. Get position details (deposit_block, amount, last_claim_block)
    let (_position_id, current_assets, _shares, _deposit_block, last_claim_block) = 
      self.get_position_details(&position_alkane)?;
    
    // 2. Calculate rewards from last_claim_block to current block
    let current_block = u128::from(self.height());
    
    // Calculate rewards based on the staked amount and time period - simplified calculation
    let rewards = if current_assets > 0 && last_claim_block < current_block {
      // Simple formula: amount * reward_per_block * blocks_elapsed / precision
      let blocks_elapsed = current_block.checked_sub(last_claim_block).unwrap_or(0);
      let precision = 1_000_000_000_000u128; // 10^12 precision
      
      current_assets
        .checked_mul(self.reward_per_block())
        .unwrap_or(0)
        .checked_mul(blocks_elapsed)
        .unwrap_or(0)
        .checked_div(precision)
        .unwrap_or(0)
    } else {
      0
    };
    
    if rewards == 0 {
      return Err(anyhow!("No rewards to claim"));
    }
    
    // 3. Transfer rewards to the user (the position token)
    // Use the reward token ID that was set during initialization
    let reward_token_id = self.reward_token_id()?;
    response.alkanes.0.push(AlkaneTransfer {
      id: reward_token_id, // ID of the reward token being transferred, not the caller
      value: rewards,
    });
    
  // 4. Call back to position token to update its last claim block
  // This ensures only the factory can update this critical state
  let update_claim_block_cellpack = Cellpack {
    target: position_alkane,
    inputs: vec![0x4, current_block], // 0x4 = UpdateLastClaimBlock opcode
  };
  
  // Create authentication parcel with our token
  let mut auth_parcel = AlkaneTransferParcel::default();
  auth_parcel.0.push(AlkaneTransfer {
    id: context.myself.clone(), // Send our factory token as authentication
    value: 1u128,
  });
  
  // Call with authentication token
  self.call(&update_claim_block_cellpack, &auth_parcel, self.fuel())?;
    
    Ok(response)
  }
  
  // Check if a given AlkaneId is in our position registry
  fn is_position_in_registry(&self, position_id: &AlkaneId) -> bool {
    let position_count = self.position_count();
    
    // Iterate through all registered positions to find a match
    for i in 0..position_count {
      let id_bytes = i.to_le_bytes().to_vec();
      let bytes = self.load_position_by_id(&id_bytes);
      
      if bytes.len() >= 32 {
        let stored_id = AlkaneId {
          block: u128::from_le_bytes(bytes[0..16].try_into().unwrap_or([0; 16])),
          tx: u128::from_le_bytes(bytes[16..32].try_into().unwrap_or([0; 16])),
        };
        
        if stored_id.block == position_id.block && stored_id.tx == position_id.tx {
          return true;
        }
      }
    }
    
    false
  }
  
  fn authenticate_position(&self, context: &Context) -> Result<()> {
    // Check that we received exactly one position token
    // if context.incoming_alkanes.0.len() != 1 {
    //   return Err(anyhow!("Position did not authenticate with exactly one token"));
    // }
    
    // The value should be at least 1
    let transfer = &context.incoming_alkanes.0[0];
    if transfer.value < 1 {
      return Err(anyhow!("Less than 1 unit of token supplied"));
    }
    
    // Check that the token is in our position registry
    if !self.is_position_in_registry(&transfer.id) {
      return Err(anyhow!("Token is not a registered position token"));
    }
    
    Ok(())
  }
  
  fn get_total_assets(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);
    response.data = self.total_assets().to_le_bytes().to_vec();
    Ok(response)
  }
  
  fn get_total_shares(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);
    response.data = self.total_shares().to_le_bytes().to_vec();
    Ok(response)
  }
  
  fn get_position_count(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);
    response.data = self.position_count().to_le_bytes().to_vec();
    Ok(response)
  }
  
  fn find_position_by_id(&self, position_id: u128) -> Result<AlkaneId> {
    let position_id_bytes = position_id.to_le_bytes().to_vec();
    let bytes = self.load_position_by_id(&position_id_bytes);
    
    if bytes.len() < 32 {
      return Ok(AlkaneId { block: 0, tx: 0 });
    }
    
    Ok(AlkaneId {
      block: u128::from_le_bytes(bytes[0..16].try_into().unwrap()),
      tx: u128::from_le_bytes(bytes[16..32].try_into().unwrap()),
    })
  }
  
  fn get_position_by_id(&self, position_id: u128) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);
    
    let alkane_id = self.find_position_by_id(position_id)?;
    
    let mut bytes = Vec::with_capacity(32);
    bytes.extend_from_slice(&alkane_id.block.to_le_bytes());
    bytes.extend_from_slice(&alkane_id.tx.to_le_bytes());
    
    response.data = bytes;
    Ok(response)
  }
  
  fn add_position(&self, position_id: &AlkaneId) -> Result<()> {
    let position_count = self.position_count();
    let position_id_bytes = position_count.to_le_bytes().to_vec();
    
    let mut bytes = Vec::with_capacity(32);
    bytes.extend_from_slice(&position_id.block.to_le_bytes());
    bytes.extend_from_slice(&position_id.tx.to_le_bytes());
    
    self.store_position_by_id(&position_id_bytes, bytes);
    
    Ok(())
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
    
  fn convert_to_shares(&self, assets: u128) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);
    
    let shares = self.convert_to_shares_internal(assets)?;
    
    response.data = shares.to_le_bytes().to_vec();
    Ok(response)
  }
  
  fn convert_to_shares_internal(&self, assets: u128) -> Result<u128> {
    if self.total_assets() == 0 {
      return Ok(assets); // Initial exchange rate 1:1
    }
    
    // shares = assets * total_shares / total_assets
    let shares = assets
      .checked_mul(self.total_shares())
      .ok_or_else(|| anyhow!("Calculation overflow"))?
      .checked_div(self.total_assets())
      .ok_or_else(|| anyhow!("Division by zero"))?;
      
    Ok(shares)
  }
  
  fn convert_to_assets(&self, shares: u128) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);
    
    let assets = self.convert_to_assets_internal(shares)?;
    
    response.data = assets.to_le_bytes().to_vec();
    Ok(response)
  }
  
  fn convert_to_assets_internal(&self, shares: u128) -> Result<u128> {
    if self.total_shares() == 0 {
      return Ok(0); // No assets if no shares
    }
    
    // assets = shares * total_assets / total_shares
    let assets = shares
      .checked_mul(self.total_assets())
      .ok_or_else(|| anyhow!("Calculation overflow"))?
      .checked_div(self.total_shares())
      .ok_or_else(|| anyhow!("Division by zero"))?;
      
    Ok(assets)
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
  
  fn total_shares(&self) -> u128 {
    self.load_u128("/total_shares")
  }
  
  fn set_total_shares(&self, total_shares: u128) {
    self.store("/total_shares".as_bytes().to_vec(), total_shares.to_le_bytes().to_vec());
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
  
  fn load_position_by_id(&self, id_bytes: &Vec<u8>) -> Vec<u8> {
    let key = format!("/positions_by_id/{}", hex::encode(id_bytes)).into_bytes();
    self.load(key)
  }
  
  fn store_position_by_id(&self, id_bytes: &Vec<u8>, value: Vec<u8>) {
    let key = format!("/positions_by_id/{}", hex::encode(id_bytes)).into_bytes();
    self.store(key, value);
  }
  
  fn reward_token_id(&self) -> Result<AlkaneId> {
    let bytes = self.load("/reward_token_id".as_bytes().to_vec());
    
    if bytes.len() < 32 {
      return Err(anyhow!("Reward token ID not set"));
    }
    
    Ok(AlkaneId {
      block: u128::from_le_bytes(bytes[0..16].try_into().unwrap()),
      tx: u128::from_le_bytes(bytes[16..32].try_into().unwrap()),
    })
  }
  
  fn set_reward_token_id(&self, id: &AlkaneId) -> Result<()> {
    let mut bytes = Vec::with_capacity(32);
    bytes.extend_from_slice(&id.block.to_le_bytes());
    bytes.extend_from_slice(&id.tx.to_le_bytes());
    
    self.store("/reward_token_id".as_bytes().to_vec(), bytes);
    Ok(())
  }
  
  fn withdraw_fees(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);
    
    // Only the owner can withdraw fees
    let owner_id = self.owner();
    if context.caller.block != owner_id.block || context.caller.tx != owner_id.tx {
      return Err(anyhow!("Only the owner can withdraw fees"));
    }
    
    // Get the collected fees
    let fees = self.collected_fees();
    if fees == 0 {
      return Err(anyhow!("No fees to withdraw"));
    }
    
    // Get the deposit token ID (same as the reward token)
    let deposit_token_id = self.reward_token_id()?;
    
    // Transfer the fees to the owner
    response.alkanes.0.push(AlkaneTransfer {
      id: deposit_token_id,
      value: fees,
    });
    
    // Reset the collected fees
    self.set_collected_fees(0);
    
    Ok(response)
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
  
  fn fee_percentage(&self) -> u128 {
    self.load_u128("/fee_percentage")
  }
  
  fn set_fee_percentage(&self, fee_percentage: u128) {
    self.store("/fee_percentage".as_bytes().to_vec(), fee_percentage.to_le_bytes().to_vec());
  }
  
  fn owner(&self) -> AlkaneId {
    let bytes = self.load("/owner".as_bytes().to_vec());
    if bytes.len() < 32 {
      panic!("Owner not set");
    }
    
    AlkaneId {
      block: u128::from_le_bytes(bytes[0..16].try_into().unwrap()),
      tx: u128::from_le_bytes(bytes[16..32].try_into().unwrap()),
    }
  }
  
  fn set_owner(&self, id: &AlkaneId) {
    let mut bytes = Vec::with_capacity(32);
    bytes.extend_from_slice(&id.block.to_le_bytes());
    bytes.extend_from_slice(&id.tx.to_le_bytes());
    
    self.store("/owner".as_bytes().to_vec(), bytes);
  }
  
  fn collected_fees(&self) -> u128 {
    self.load_u128("/collected_fees")
  }
  
  fn set_collected_fees(&self, collected_fees: u128) {
    self.store("/collected_fees".as_bytes().to_vec(), collected_fees.to_le_bytes().to_vec());
  }
  
  // Test function to verify if code updates are working
  fn test_ping(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);
    
    // Return 12345 as a unique identifier for this function
    let ping_value = 12345u128;
    response.data = ping_value.to_le_bytes().to_vec();
    
    Ok(response)
  }
}

declare_alkane! {
  impl AlkaneResponder for VaultFactory {
    type Message = VaultFactoryMessage;
  }
}
