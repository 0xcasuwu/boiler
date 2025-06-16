use metashrew_support::compat::to_arraybuffer_layout;

use alkanes_runtime::{
    declare_alkane, message::MessageDispatch, runtime::AlkaneResponder, token::Token,
};

use alkanes_support::{
    cellpack::Cellpack,
    id::AlkaneId,
    parcel::{AlkaneTransfer, AlkaneTransferParcel},
    response::CallResponse,
};

use anyhow::{anyhow, Result};

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

    #[opcode(14)]
    #[returns(u128)]
    GetDepositBlock,

    #[opcode(16)]
    #[returns(u128)]
    GetRewardDebt,

    #[opcode(23)]
    #[returns((u128, u128, u128, u128))]
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

    fn calculate_blocks_staked(&self) -> Result<CallResponse> {
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

        // Get the vault factory reference
        let vault_id = self.vault_ref();

        // Call vault to calculate rewards
        let cellpack = Cellpack {
            target: vault_id,
            inputs: vec![
                0x20,                      // 0x20 = CalculateRewards opcode
                self.deposit_amount(), // PURE MASTERCHEF: use deposit_amount instead of current_assets
                self.deposit_block(),  // PURE MASTERCHEF: use deposit_block as start point
                u128::from(self.height()), // Current block
            ],
        };

        let vault_response =
            self.staticcall(&cellpack, &AlkaneTransferParcel::default(), self.fuel())?;
        response.data = vault_response.data;

        Ok(response)
    }

    fn get_position_value(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        // PURE MASTERCHEF: Position value is simply the deposit amount (1:1 ratio)
        response.data = self.deposit_amount().to_le_bytes().to_vec();

        Ok(response)
    }

    // Storage operations using direct store/load methods

    fn vault_ref(&self) -> AlkaneId {
        let bytes = self.load("/vault_id".as_bytes().to_vec());
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

        self.store("/vault_id".as_bytes().to_vec(), bytes);
    }

    fn position_id(&self) -> u128 {
        self.load_u128("/position_id")
    }

    fn set_position_id(&self, position_id: u128) {
        self.store(
            "/position_id".as_bytes().to_vec(),
            position_id.to_le_bytes().to_vec(),
        );
    }

    // PURE MASTERCHEF: Essential storage functions
    fn deposit_amount(&self) -> u128 {
        self.load_u128("/deposit_amount")
    }

    fn set_deposit_amount(&self, deposit_amount: u128) {
        self.store(
            "/deposit_amount".as_bytes().to_vec(),
            deposit_amount.to_le_bytes().to_vec(),
        );
    }

    fn deposit_block(&self) -> u128 {
        self.load_u128("/deposit_block")
    }

    fn set_deposit_block(&self, deposit_block: u128) {
        self.store(
            "/deposit_block".as_bytes().to_vec(),
            deposit_block.to_le_bytes().to_vec(),
        );
    }

    fn last_claim_block(&self) -> u128 {
        self.load_u128("/last_claim_block")
    }

    fn set_last_claim_block(&self, last_claim_block: u128) {
        self.store(
            "/last_claim_block".as_bytes().to_vec(),
            last_claim_block.to_le_bytes().to_vec(),
        );
    }

    fn deposit_token_id(&self) -> Result<AlkaneId> {
        let bytes = self.load("/deposit_token_id".as_bytes().to_vec());

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

        self.store("/deposit_token_id".as_bytes().to_vec(), bytes);
        Ok(())
    }

    // SUSHISWAP-STYLE: Reward debt storage functions
    fn reward_debt(&self) -> u128 {
        self.load_u128("/reward_debt")
    }

    fn set_reward_debt(&self, reward_debt: u128) {
        self.store(
            "/reward_debt".as_bytes().to_vec(),
            reward_debt.to_le_bytes().to_vec(),
        );
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
}

declare_alkane! {
  impl AlkaneResponder for PositionToken {
    type Message = PositionTokenMessage;
  }
}
