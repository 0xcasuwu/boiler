use metashrew_support::compat::to_arraybuffer_layout;
use metashrew_support::index_pointer::KeyValuePointer;

use alkanes_runtime::{
    declare_alkane, message::MessageDispatch, runtime::AlkaneResponder, token::Token,
    storage::StoragePointer,
};

use alkanes_support::{
    cellpack::Cellpack,
    id::AlkaneId,
    parcel::{AlkaneTransfer, AlkaneTransferParcel},
    response::CallResponse,
};

use anyhow::{anyhow, Result};
use std::sync::Arc;

mod svg_generator;
use svg_generator::{SvgGenerator, PositionData};

/// Trims a u128 value to a String by removing trailing zeros
pub fn trim(v: u128) -> String {
    let bytes: Vec<u8> = v.to_le_bytes()
        .into_iter()
        .take_while(|&b| b != 0)
        .collect();
    
    // Only attempt UTF-8 conversion if we have valid bytes
    if bytes.is_empty() {
        String::new()
    } else {
        String::from_utf8(bytes).unwrap_or_else(|_| {
            // Simple fallback: just use the numeric value as string
            v.to_string()
        })
    }
}

/// TokenName struct to hold two u128 values for the name
#[derive(Default, Clone, Copy)]
pub struct TokenName {
    pub part1: u128,
    pub part2: u128,
}

impl From<TokenName> for String {
    fn from(name: TokenName) -> Self {
        // Trim both parts and concatenate them
        format!("{}{}", trim(name.part1), trim(name.part2)) 
    }
}

impl TokenName {
    pub fn new(part1: u128, part2: u128) -> Self {
        Self { part1, part2 }
    }
}

/// Returns a StoragePointer for the token name
fn name_pointer() -> StoragePointer {
    StoragePointer::from_keyword("/name")
}

/// Returns a StoragePointer for the token symbol
fn symbol_pointer() -> StoragePointer {
    StoragePointer::from_keyword("/symbol")
}

/// Convert string to u128 for name encoding
fn string_to_u128(s: &str) -> u128 {
    let bytes = s.as_bytes();
    let mut result = 0u128;
    for (i, &byte) in bytes.iter().enumerate() {
        if i >= 16 { break; } // u128 can only hold 16 bytes
        result |= (byte as u128) << (i * 8);
    }
    result
}

#[derive(Default)]
pub struct PositionToken(());

impl AlkaneResponder for PositionToken {}

#[derive(MessageDispatch)]
enum PositionTokenMessage {
    #[opcode(0)]
    Initialize {
        position_id: u128,
        deposit_amount: u128,
        reward_debt: u128,
        deposit_block: u128,
        deposit_token_id: AlkaneId,
    },

    #[opcode(10)]
    #[returns(u128)]
    GetPositionId,

    #[opcode(11)]
    #[returns(u128)]
    GetDepositAmount,

    #[opcode(12)]
    #[returns(u128)]
    GetPendingRewards,

    #[opcode(13)]
    #[returns(u128)]
    GetPositionValue,

    #[opcode(14)]
    #[returns(u128)]
    GetDepositBlock,

    #[opcode(15)]
    #[returns(u128)]
    GetBlocksStaked,

    #[opcode(16)]
    #[returns(u128)]
    GetRewardDebt,

    #[opcode(17)]
    #[returns(u128)]
    GetLastClaimBlock,

    #[opcode(18)]
    #[returns(AlkaneId)]
    GetVaultId,

    #[opcode(23)]
    #[returns((u128, u128, u128, u128))]
    GetAllDetails,

    #[opcode(24)]
    #[returns(AlkaneId)]
    GetDepositTokenId,

    /// Get the token name
    #[opcode(99)]
    #[returns(String)]
    GetName,

    /// Get the token symbol
    #[opcode(100)]
    #[returns(String)]
    GetSymbol,

    /// Get the SVG data
    #[opcode(1000)]
    #[returns(Vec<u8>)]
    GetData,

    /// Get the content type
    #[opcode(1001)]
    #[returns(String)]
    GetContentType,

    /// Get the attributes (metadata)
    #[opcode(1002)]
    #[returns(String)]
    GetAttributes,
}

impl Token for PositionToken {
    fn name(&self) -> String {
        String::from_utf8(name_pointer().get().as_ref().clone())
            .unwrap_or_else(|_| {
                // Safe fallback that doesn't rely on storage that might not be initialized
                let position_id = self.position_id_pointer().get_value::<u128>();
                format!("Vault Position #{}", position_id)
            })
    }

    fn symbol(&self) -> String {
        String::from_utf8(symbol_pointer().get().as_ref().clone())
            .unwrap_or_else(|_| {
                // Safe fallback that doesn't rely on storage that might not be initialized
                let position_id = self.position_id_pointer().get_value::<u128>();
                format!("POS-{}", position_id)
            })
    }
}

impl PositionToken {
    fn initialize(
        &self,
        position_id: u128,
        deposit_amount: u128,
        reward_debt: u128,
        deposit_block: u128,
        deposit_token_id: AlkaneId,
    ) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::default();

        self.observe_initialization()?;

        // Simplified: Set basic name and symbol
        let name_string = format!("Position #{}", position_id);
        let symbol_string = format!("POS-{}", position_id);
        
        name_pointer().set(Arc::new(name_string.as_bytes().to_vec()));
        symbol_pointer().set(Arc::new(symbol_string.as_bytes().to_vec()));

        // PURE MASTERCHEF: Store only essential position details
        self.set_vault_id(&context.caller);
        self.set_position_id(position_id);
        self.set_deposit_amount(deposit_amount);
        self.set_reward_debt(reward_debt);
        self.set_deposit_block(deposit_block);
        self.set_deposit_token_id(&deposit_token_id)?;

        // Position token is purely authentication/tracking
        response.alkanes.0.push(AlkaneTransfer {
            id: context.myself.clone(),
            value: 1u128,
        });

        Ok(response)
    }

    /// Set the token name and symbol (following free-mint pattern)
    fn set_name_and_symbol(&self, name: TokenName, symbol: u128) {
        let name_string: String = name.into();
        name_pointer()
            .set(Arc::new(name_string.as_bytes().to_vec()));
        self.set_string_field(symbol_pointer(), symbol);
    }

    /// Set a string field in storage (following free-mint pattern)
    fn set_string_field(&self, mut pointer: StoragePointer, v: u128) {
        pointer.set(Arc::new(trim(v).as_bytes().to_vec()));
    }

    // Withdraw and claim rewards functionality moved to the factory

    fn get_position_id(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        response.data = self.position_id().to_le_bytes().to_vec();
        Ok(response)
    }

    fn get_deposit_amount(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        response.data = self.deposit_amount().to_le_bytes().to_vec();
        Ok(response)
    }

    fn get_reward_debt(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        response.data = self.reward_debt().to_le_bytes().to_vec();
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

    fn get_blocks_staked(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        let current_block = self.height();
        let deposit_block = self.deposit_block();

        let blocks_staked = if u128::from(current_block) > deposit_block {
            u128::from(current_block) - deposit_block
        } else {
            0
        };

        response.data = blocks_staked.to_le_bytes().to_vec();
        Ok(response)
    }

    fn get_pending_rewards(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        // FIXED: Proper MasterChef pending rewards calculation
        // Get current acc_reward_per_share from vault
        let vault_id = self.vault_ref();
        let cellpack = Cellpack {
            target: vault_id,
            inputs: vec![39u128], // GetAccRewardPerShare opcode
        };

        let vault_response = self.staticcall(&cellpack, &AlkaneTransferParcel::default(), self.fuel())?;
        
        let acc_reward_per_share = if vault_response.data.len() >= 16 {
            u128::from_le_bytes(vault_response.data[0..16].try_into().unwrap_or([0; 16]))
        } else {
            0
        };

        // Calculate proper MasterChef pending rewards
        let deposit_amount = self.deposit_amount();
        let reward_debt = self.reward_debt();
        let precision = 100_000_000u128; // 1e8 precision (Bitcoin satoshi standard)

        let accumulated_rewards = deposit_amount
            .checked_mul(acc_reward_per_share)
            .and_then(|x| x.checked_div(precision))
            .unwrap_or(0);
        
        let pending_rewards = accumulated_rewards.saturating_sub(reward_debt);

        response.data = pending_rewards.to_le_bytes().to_vec();
        Ok(response)
    }

    fn get_vault_id(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        let vault_id = self.vault_ref();
        
        // Pack AlkaneId into response (32 bytes: 16 for block, 16 for tx)
        let mut data = Vec::with_capacity(32);
        data.extend_from_slice(&vault_id.block.to_le_bytes());
        data.extend_from_slice(&vault_id.tx.to_le_bytes());
        
        response.data = data;
        Ok(response)
    }

    fn get_position_value(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        // PURE MASTERCHEF: Position value is simply the deposit amount (1:1 ratio)
        response.data = self.deposit_amount().to_le_bytes().to_vec();

        Ok(response)
    }

    // Storage operations using StoragePointer archetype pattern

    fn vault_alkane_id_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/vault-alkane-id")
    }

    fn vault_ref(&self) -> AlkaneId {
        let data = self.vault_alkane_id_pointer().get();
        if data.len() == 0 {
            panic!("Vault reference not found");
        }
        
        let bytes = data.as_ref();
        AlkaneId {
            block: u128::from_le_bytes(bytes[0..16].try_into().unwrap()),
            tx: u128::from_le_bytes(bytes[16..32].try_into().unwrap()),
        }
    }

    fn set_vault_id(&self, id: &AlkaneId) {
        let mut bytes = Vec::with_capacity(32);
        bytes.extend_from_slice(&id.block.to_le_bytes());
        bytes.extend_from_slice(&id.tx.to_le_bytes());
        
        self.vault_alkane_id_pointer().set(Arc::new(bytes));
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

    // PURE MASTERCHEF: Essential storage functions using StoragePointer pattern
    fn deposit_amount_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/deposit_amount")
    }

    fn deposit_amount(&self) -> u128 {
        self.deposit_amount_pointer().get_value::<u128>()
    }

    fn set_deposit_amount(&self, deposit_amount: u128) {
        self.deposit_amount_pointer().set_value::<u128>(deposit_amount);
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

    fn deposit_token_alkane_id_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/deposit-token-alkane-id")
    }

    fn deposit_token_id(&self) -> Result<AlkaneId> {
        let data = self.deposit_token_alkane_id_pointer().get();
        if data.len() == 0 {
            return Err(anyhow!("Deposit token ID not set"));
        }
        
        let bytes = data.as_ref();
        Ok(AlkaneId {
            block: u128::from_le_bytes(bytes[0..16].try_into().unwrap()),
            tx: u128::from_le_bytes(bytes[16..32].try_into().unwrap()),
        })
    }

    fn set_deposit_token_id(&self, id: &AlkaneId) -> Result<()> {
        let mut bytes = Vec::with_capacity(32);
        bytes.extend_from_slice(&id.block.to_le_bytes());
        bytes.extend_from_slice(&id.tx.to_le_bytes());
        
        self.deposit_token_alkane_id_pointer().set(Arc::new(bytes));
        Ok(())
    }

    // MASTERCHEF: Reward debt storage functions using StoragePointer pattern
    fn reward_debt_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/reward_debt")
    }

    fn reward_debt(&self) -> u128 {
        self.reward_debt_pointer().get_value::<u128>()
    }

    fn set_reward_debt(&self, reward_debt: u128) {
        self.reward_debt_pointer().set_value::<u128>(reward_debt);
    }

    // PURE MASTERCHEF: No internal update functions needed
    // All data is immutable after position creation

    fn get_all_details(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        // PURE MASTERCHEF: Return only essential fields (4 values total)
        let position_id = self.position_id();
        let deposit_amount = self.deposit_amount();
        let reward_debt = self.reward_debt();
        let deposit_block = self.deposit_block();

        // Pack only essential values into a single byte array
        // Each value is 16 bytes (128 bits) - 4 values total for pure MasterChef
        let mut data = Vec::with_capacity(16 * 4);
        data.extend_from_slice(&position_id.to_le_bytes());
        data.extend_from_slice(&deposit_amount.to_le_bytes());
        data.extend_from_slice(&reward_debt.to_le_bytes());
        data.extend_from_slice(&deposit_block.to_le_bytes());

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

    /// Get the token name (following free-mint pattern)
    fn get_name(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        response.data = self.name().into_bytes().to_vec();

        Ok(response)
    }

    /// Get the token symbol (following free-mint pattern)
    fn get_symbol(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        response.data = self.symbol().into_bytes().to_vec();

        Ok(response)
    }

    /// Get the SVG data for this position token
    fn get_data(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        // Calculate pending rewards by calling the vault
        let vault_id = self.vault_ref();
        let cellpack = Cellpack {
            target: vault_id,
            inputs: vec![
                0x20,                      // 0x20 = CalculateRewards opcode
                self.deposit_amount(),
                self.deposit_block(),
                u128::from(self.height()),
            ],
        };
        
        let vault_response = self.staticcall(&cellpack, &AlkaneTransferParcel::default(), self.fuel())?;
        let pending_rewards = if vault_response.data.len() >= 16 {
            u128::from_le_bytes(vault_response.data[0..16].try_into().unwrap_or([0; 16]))
        } else {
            0
        };

        // Get the actual token symbol by calling the token contract
        let token_id = self.deposit_token_id()?;
        let cellpack_symbol = Cellpack {
            target: token_id,
            inputs: vec![100], // GetSymbol opcode
        };
        
        let symbol_response = self.staticcall(&cellpack_symbol, &AlkaneTransferParcel::default(), self.fuel())?;
        let token_symbol = if !symbol_response.data.is_empty() {
            String::from_utf8(symbol_response.data).unwrap_or_else(|_| format!("TOK-{}", token_id.tx % 10000))
        } else {
            format!("TOK-{}", token_id.tx % 10000)
        };

        // Gather all position data
        let position_data = PositionData {
            position_id: self.position_id(),
            deposit_amount: self.deposit_amount(),
            reward_debt: self.reward_debt(),
            deposit_block: self.deposit_block(),
            deposit_token_id: self.deposit_token_id()?,
            current_block: u128::from(self.height()),
            pending_rewards,
            token_symbol,
        };

        // Generate the SVG
        let svg = SvgGenerator::generate_svg(position_data)?;
        response.data = svg.into_bytes();

        Ok(response)
    }

    /// Get the content type for the SVG
    fn get_content_type(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        response.data = String::from("image/svg+xml").into_bytes();

        Ok(response)
    }

    /// Get the attributes (metadata) for this position token
    fn get_attributes(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        // Calculate pending rewards by calling the vault
        let vault_id = self.vault_ref();
        let cellpack = Cellpack {
            target: vault_id,
            inputs: vec![
                0x20,                      // 0x20 = CalculateRewards opcode
                self.deposit_amount(),
                self.deposit_block(),
                u128::from(self.height()),
            ],
        };
        
        let vault_response = self.staticcall(&cellpack, &AlkaneTransferParcel::default(), self.fuel())?;
        let pending_rewards = if vault_response.data.len() >= 16 {
            u128::from_le_bytes(vault_response.data[0..16].try_into().unwrap_or([0; 16]))
        } else {
            0
        };

        // Get the actual token symbol by calling the token contract
        let token_id = self.deposit_token_id()?;
        let cellpack_symbol = Cellpack {
            target: token_id,
            inputs: vec![100], // GetSymbol opcode
        };
        
        let symbol_response = self.staticcall(&cellpack_symbol, &AlkaneTransferParcel::default(), self.fuel())?;
        let token_symbol = if !symbol_response.data.is_empty() {
            String::from_utf8(symbol_response.data).unwrap_or_else(|_| format!("TOK-{}", token_id.tx % 10000))
        } else {
            format!("TOK-{}", token_id.tx % 10000)
        };

        // Gather all position data
        let position_data = PositionData {
            position_id: self.position_id(),
            deposit_amount: self.deposit_amount(),
            reward_debt: self.reward_debt(),
            deposit_block: self.deposit_block(),
            deposit_token_id: self.deposit_token_id()?,
            current_block: u128::from(self.height()),
            pending_rewards,
            token_symbol,
        };

        // Generate the attributes JSON
        let attributes = SvgGenerator::get_attributes(position_data)?;
        response.data = attributes.into_bytes();

        Ok(response)
    }
}

impl PositionToken {
    fn handle(&self, message: PositionTokenMessage) -> Result<CallResponse> {
        match message {
            PositionTokenMessage::Initialize {
                position_id,
                deposit_amount,
                reward_debt,
                deposit_block,
                deposit_token_id,
            } => self.initialize(position_id, deposit_amount, reward_debt, deposit_block, deposit_token_id),
            PositionTokenMessage::GetPositionId => self.get_position_id(),
            PositionTokenMessage::GetDepositAmount => self.get_deposit_amount(),
            PositionTokenMessage::GetPendingRewards => self.get_pending_rewards(),
            PositionTokenMessage::GetPositionValue => self.get_position_value(),
            PositionTokenMessage::GetDepositBlock => self.get_deposit_block(),
            PositionTokenMessage::GetBlocksStaked => self.get_blocks_staked(),
            PositionTokenMessage::GetRewardDebt => self.get_reward_debt(),
            PositionTokenMessage::GetLastClaimBlock => self.get_last_claim_block(),
            PositionTokenMessage::GetVaultId => self.get_vault_id(),
            PositionTokenMessage::GetAllDetails => self.get_all_details(),
            PositionTokenMessage::GetDepositTokenId => self.get_deposit_token_id(),
            PositionTokenMessage::GetName => self.get_name(),
            PositionTokenMessage::GetSymbol => self.get_symbol(),
            PositionTokenMessage::GetData => self.get_data(),
            PositionTokenMessage::GetContentType => self.get_content_type(),
            PositionTokenMessage::GetAttributes => self.get_attributes(),
        }
    }
}

declare_alkane! {
  impl AlkaneResponder for PositionToken {
    type Message = PositionTokenMessage;
  }
}
