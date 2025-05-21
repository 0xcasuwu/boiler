use metashrew_support::index_pointer::KeyValuePointer;
use metashrew_support::compat::to_arraybuffer_layout;

use alkanes_runtime::{
  declare_alkane, message::MessageDispatch, storage::StoragePointer, token::Token,
  runtime::AlkaneResponder
};

use alkanes_support::{
  cellpack::Cellpack, id::AlkaneId,
  parcel::{AlkaneTransfer, AlkaneTransferParcel}, response::CallResponse
};

use anyhow::{anyhow, Result};
use std::sync::Arc;

#[derive(Default)]
pub struct PositionToken(());

impl AlkaneResponder for PositionToken {}

#[derive(MessageDispatch)]
enum PositionTokenMessage {
  #[opcode(0)]
  Initialize {
    position_id: u128,
    initial_assets: u128,
    shares: u128,
    deposit_block: u128,
    deposit_token_id: AlkaneId,
  },
  
  #[opcode(4)]
  UpdateLastClaimBlock {
    new_block: u128,
  },

  #[opcode(5)]
  UpdateCurrentAssets {
    new_amount: u128,
  },

  #[opcode(10)]
  #[returns(u128)]
  GetPositionId,

  #[opcode(11)]
  #[returns(u128)]
  GetInitialAssets,

  #[opcode(12)]
  #[returns(u128)]
  GetCurrentAssets,

  #[opcode(13)]
  #[returns(u128)]
  GetShares,

  #[opcode(14)]
  #[returns(u128)]
  GetDepositBlock,

  #[opcode(15)]
  #[returns(u128)]
  GetLastClaimBlock,

  #[opcode(20)]
  #[returns(u128)]
  CalculateBlocksStaked,

  #[opcode(21)]
  #[returns(u128)]
  GetPendingRewards,

  #[opcode(22)]
  #[returns(u128)]
  GetPositionValue,
  
  #[opcode(23)]
  #[returns((u128, u128, u128, u128, u128))]
  GetAllDetails,
  
  #[opcode(24)]
  #[returns(AlkaneId)]
  GetDepositTokenId,
}

impl Token for PositionToken {
  fn name(&self) -> String {
    format!("Vault Position #{}", self.position_id())
  }

  fn symbol(&self) -> String {
    format!("POS-{}", self.position_id())
  }
}

impl PositionToken {
  fn initialize(&self, position_id: u128, initial_assets: u128, shares: u128, deposit_block: u128, deposit_token_id: AlkaneId) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);
    
    self.observe_initialization()?;
    
    // Store position details
    self.set_vault_id(&context.caller);
    self.set_position_id(position_id);
    self.set_initial_assets(initial_assets);
    self.set_current_assets(initial_assets);
    self.set_shares(shares);
    
    // Record block heights and deposit token ID
    self.set_deposit_block(deposit_block);
    self.set_last_claim_block(deposit_block);
    self.set_deposit_token_id(&deposit_token_id)?;
    
    // Set position token
    response.alkanes.0.push(AlkaneTransfer {
      id: context.myself.clone(),
      value: 1u128,
    });
    
    Ok(response)
  }
  
  // Withdraw and claim rewards functionality moved to the factory
  
  fn get_position_id(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);
    response.data = self.position_id().to_le_bytes().to_vec();
    Ok(response)
  }
  
  fn get_initial_assets(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);
    response.data = self.initial_assets().to_le_bytes().to_vec();
    Ok(response)
  }
  
  fn get_current_assets(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);
    response.data = self.current_assets().to_le_bytes().to_vec();
    Ok(response)
  }
  
  fn get_shares(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);
    response.data = self.shares().to_le_bytes().to_vec();
    Ok(response)
  }
  
  fn get_deposit_block(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);
    response.data = self.deposit_block().to_le_bytes().to_vec();
    Ok(response)
  }
  
  fn get_last_claim_block(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);
    response.data = self.last_claim_block().to_le_bytes().to_vec();
    Ok(response)
  }
  
  fn calculate_blocks_staked(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);
    
    let current_block = self.height();
    let deposit_block = self.deposit_block();
    
    let blocks_staked = if current_block > deposit_block {
      current_block - deposit_block
    } else {
      0
    };
    
    response.data = blocks_staked.to_le_bytes().to_vec();
    Ok(response)
  }
  
  fn get_pending_rewards(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);
    
    // Get the vault factory reference
    let vault_id = self.vault_ref();
    
    // Call vault to calculate rewards
    let cellpack = Cellpack {
      target: vault_id,
      inputs: vec![
        0x20,  // 0x20 = CalculateRewards opcode
        self.current_assets(),
        self.last_claim_block(),
        self.height(),  // Current block
      ],
    };
    
    let vault_response = self.staticcall(&cellpack, &AlkaneTransferParcel::default(), self.fuel())?;
    response.data = vault_response.data;
    
    Ok(response)
  }
  
  fn get_position_value(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);
    
    // Get the vault factory reference
    let vault_id = self.vault_ref();
    
    // Call vault to convert shares to assets
    let cellpack = Cellpack {
      target: vault_id,
      inputs: vec![
        0x22,  // 0x22 = ConvertToAssets opcode
        self.shares(),
      ],
    };
    
    let vault_response = self.staticcall(&cellpack, &AlkaneTransferParcel::default(), self.fuel())?;
    response.data = vault_response.data;
    
    Ok(response)
  }
  
  // Storage pointers and accessors
  
  fn vault_id_pointer(&self) -> StoragePointer {
    StoragePointer::from_keyword("/vault_id")
  }
  
  fn vault_ref(&self) -> AlkaneId {
    let bytes = self.vault_id_pointer().get();
    if bytes.len() < 32 {
      panic!("Vault reference not found");
    }
    
    AlkaneId {
      block: u128::from_le_bytes(bytes[0..16].try_into().unwrap()),
      tx: u128::from_le_bytes(bytes[16..32].try_into().unwrap()),
    }
  }
  
  fn set_vault_id(&self, id: &AlkaneId) {
    let mut bytes = Vec::with_capacity(32);
    bytes.extend_from_slice(&id.block.to_le_bytes());
    bytes.extend_from_slice(&id.tx.to_le_bytes());
    
    self.vault_id_pointer().set(Arc::new(bytes));
  }
  
  fn position_id_pointer(&self) -> StoragePointer {
    StoragePointer::from_keyword("/position_id")
  }
  
  fn position_id(&self) -> u128 {
    self.position_id_pointer().get_value::<u128>()
  }
  
  fn set_position_id(&self, position_id: u128) {
    self.position_id_pointer().set_value::<u128>(position_id);
  }
  
  fn initial_assets_pointer(&self) -> StoragePointer {
    StoragePointer::from_keyword("/initial_assets")
  }
  
  fn initial_assets(&self) -> u128 {
    self.initial_assets_pointer().get_value::<u128>()
  }
  
  fn set_initial_assets(&self, initial_assets: u128) {
    self.initial_assets_pointer().set_value::<u128>(initial_assets);
  }
  
  fn current_assets_pointer(&self) -> StoragePointer {
    StoragePointer::from_keyword("/current_assets")
  }
  
  fn current_assets(&self) -> u128 {
    self.current_assets_pointer().get_value::<u128>()
  }
  
  fn set_current_assets(&self, current_assets: u128) {
    self.current_assets_pointer().set_value::<u128>(current_assets);
  }
  
  fn shares_pointer(&self) -> StoragePointer {
    StoragePointer::from_keyword("/shares")
  }
  
  fn shares(&self) -> u128 {
    self.shares_pointer().get_value::<u128>()
  }
  
  fn set_shares(&self, shares: u128) {
    self.shares_pointer().set_value::<u128>(shares);
  }
  
  fn deposit_block_pointer(&self) -> StoragePointer {
    StoragePointer::from_keyword("/deposit_block")
  }
  
  fn deposit_block(&self) -> u128 {
    self.deposit_block_pointer().get_value::<u128>()
  }
  
  fn set_deposit_block(&self, deposit_block: u128) {
    self.deposit_block_pointer().set_value::<u128>(deposit_block);
  }
  
  fn last_claim_block_pointer(&self) -> StoragePointer {
    StoragePointer::from_keyword("/last_claim_block")
  }
  
  fn last_claim_block(&self) -> u128 {
    self.last_claim_block_pointer().get_value::<u128>()
  }
  
  fn set_last_claim_block(&self, last_claim_block: u128) {
    self.last_claim_block_pointer().set_value::<u128>(last_claim_block);
  }
  
  fn deposit_token_id_pointer(&self) -> StoragePointer {
    StoragePointer::from_keyword("/deposit_token_id")
  }
  
  fn deposit_token_id(&self) -> Result<AlkaneId> {
    let bytes = self.deposit_token_id_pointer().get();
    
    if bytes.len() < 32 {
      return Err(anyhow!("Deposit token ID not set"));
    }
    
    Ok(AlkaneId {
      block: u128::from_le_bytes(bytes[0..16].try_into().unwrap()),
      tx: u128::from_le_bytes(bytes[16..32].try_into().unwrap()),
    })
  }
  
  fn set_deposit_token_id(&self, id: &AlkaneId) -> Result<()> {
    let mut bytes = Vec::with_capacity(32);
    bytes.extend_from_slice(&id.block.to_le_bytes());
    bytes.extend_from_slice(&id.tx.to_le_bytes());
    
    self.deposit_token_id_pointer().set(Arc::new(bytes));
    Ok(())
  }
  
  fn update_current_assets(&self, new_amount: u128) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);
    
    // Only the vault factory can update the current assets
    let vault_id = self.vault_ref();
    
    if context.incoming_alkanes.0.len() != 1 {
      return Err(anyhow!("Did not authenticate with only the vault factory token"));
    }
    
    let transfer = context.incoming_alkanes.0[0].clone();
    if transfer.id != vault_id {
      return Err(anyhow!("Supplied alkane is not the vault factory token"));
    }
    
    if transfer.value < 1 {
      return Err(anyhow!("Less than 1 unit of vault factory token supplied"));
    }
    
    // Update the current assets
    self.set_current_assets(new_amount);
    
    Ok(response)
  }
  
  fn update_last_claim_block(&self, new_block: u128) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);
    
    // Only the vault factory can update the last claim block
    // Authentication by vault token, not by caller address
    let vault_id = self.vault_ref();
    
    if context.incoming_alkanes.0.len() != 1 {
      return Err(anyhow!("Did not authenticate with only the vault factory token"));
    }
    
    let transfer = context.incoming_alkanes.0[0].clone();
    if transfer.id != vault_id {
      return Err(anyhow!("Supplied alkane is not the vault factory token"));
    }
    
    if transfer.value < 1 {
      return Err(anyhow!("Less than 1 unit of vault factory token supplied"));
    }
    
    // Update the last claim block
    self.set_last_claim_block(new_block);
    
    Ok(response)
  }
  
  fn get_all_details(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);
    
    let position_id = self.position_id();
    let current_assets = self.current_assets();
    let shares = self.shares();
    let deposit_block = self.deposit_block();
    let last_claim_block = self.last_claim_block();
    
    // Pack all values into a single byte array
    // Each value is 16 bytes (128 bits)
    let mut data = Vec::with_capacity(16 * 5);
    data.extend_from_slice(&position_id.to_le_bytes());
    data.extend_from_slice(&current_assets.to_le_bytes());
    data.extend_from_slice(&shares.to_le_bytes());
    data.extend_from_slice(&deposit_block.to_le_bytes());
    data.extend_from_slice(&last_claim_block.to_le_bytes());
    
    response.data = data;
    Ok(response)
  }
  
  fn get_deposit_token_id(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);
    
    let deposit_token_id = self.deposit_token_id()?;
    
    // Pack deposit token ID into the response
    let mut data = Vec::with_capacity(32);
    data.extend_from_slice(&deposit_token_id.block.to_le_bytes());
    data.extend_from_slice(&deposit_token_id.tx.to_le_bytes());
    
    response.data = data;
    Ok(response)
  }
}

declare_alkane! {
  impl AlkaneResponder for PositionToken {
    type Message = PositionTokenMessage;
  }
}
