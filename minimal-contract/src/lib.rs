use alkanes_runtime::runtime::AlkaneResponder;
use alkanes_runtime::{declare_alkane, message::MessageDispatch, token::Token};
use alkanes_support::response::CallResponse;
use alkanes_support::id::AlkaneId;
use anyhow::{anyhow, Result};
use metashrew_support::compat::to_arraybuffer_layout;
use alkanes_support::parcel::AlkaneTransfer;
#[derive(Default)]
pub struct YieldVault(());

impl Token for YieldVault {
    fn name(&self) -> String {
        String::from("YieldVault")
    }
    fn symbol(&self) -> String {
        String::from("YVT")
    }
}

#[derive(MessageDispatch)]
enum YieldVaultMessage {
    #[opcode(0)]
    Initialize,

    #[opcode(1)]
    Authenticate,
    
    #[opcode(2)]
    SetCurrencyAlkane {
        block: u128,
        tx: u128,
    },
    
    // New message for setting reward alkane
    #[opcode(3)]
    SetRewardAlkane {
        block: u128,
        tx: u128,
    },

    // New messages for ERC4626 functionality
    #[opcode(10)]
    Deposit,
    
    #[opcode(11)]
    Withdraw,
    
    #[opcode(12)]
    #[returns(u128)]
    PreviewDeposit { assets: u128 },
    
    #[opcode(13)]
    #[returns(u128)]
    PreviewWithdraw { shares: u128 },
    
    #[opcode(14)]
    #[returns(u128)]
    ConvertToShares { assets: u128 },
    
    #[opcode(15)]
    #[returns(u128)]
    ConvertToAssets { shares: u128 },
    
    // Reward-related messages
    #[opcode(20)]
    ClaimRewards,
    
    #[opcode(21)]
    #[returns(u128)]
    GetAvailableRewards,
    
    #[opcode(22)]
    #[returns(u128)]
    GetNextRewardBlock,
    
    #[opcode(23)]
    #[returns(bool)]
    IsDepositAllowed,
    
    // Existing getter methods
    #[opcode(77)]
    #[returns(String)]
    GetName,

    #[opcode(101)]
    #[returns(String)]
    GetSymbol,
    
    #[opcode(102)]
    GetHeight,
    
    #[opcode(103)]
    GetEndBlock,
    
    #[opcode(104)]
    #[returns(String)]
    GetCurrencyAlkane,
    
    // New getter method for reward alkane
    #[opcode(109)]
    #[returns(String)]
    GetRewardAlkane,
    
    // New getter methods
    #[opcode(105)]
    #[returns(u128)]
    GetTotalAssets,
    
    #[opcode(106)]
    #[returns(u128)]
    GetTotalShares,
    
    #[opcode(107)]
    #[returns(u128)]
    GetDepositFee,
    
    #[opcode(108)]
    #[returns(u128)]
    GetWithdrawFee,
}

impl YieldVault {

    fn initialize(&self) -> Result<CallResponse> {
        // Use the standard observe_initialization method which will throw an error if already initialized
        self.observe_initialization()?;
        
        let context = self.context()?;
        let mut response: CallResponse = CallResponse::forward(&context.incoming_alkanes.clone());
        
        // Get current block height
        let height = self.height();
        
        // Store the start block height
        self.store("/start_block".as_bytes().to_vec(), height.to_le_bytes().to_vec());
        
        // Store the last reward block (same as start block initially)
        self.store("/last_reward_block".as_bytes().to_vec(), height.to_le_bytes().to_vec());
        
        // Set reward period length (144 blocks, approximately daily)
        let reward_period_length: u128 = 144;
        self.store("/reward_period_length".as_bytes().to_vec(), reward_period_length.to_le_bytes().to_vec());
        
        // Set anti-exploitation window (10 blocks before reward)
        let anti_exploit_window: u128 = 10;
        self.store("/anti_exploit_window".as_bytes().to_vec(), anti_exploit_window.to_le_bytes().to_vec());
        
        // Set maximum reward supply (1,000,000 tokens)
        let max_reward_supply: u128 = 1_000_000;
        self.store("/max_reward_supply".as_bytes().to_vec(), max_reward_supply.to_le_bytes().to_vec());
        
        // Initialize reward tracking
        self.store("/total_rewards_minted".as_bytes().to_vec(), 0u128.to_le_bytes().to_vec());
        self.store("/available_rewards".as_bytes().to_vec(), 0u128.to_le_bytes().to_vec());
        
        // Initialize total assets and shares to zero
        self.store("/total_assets".as_bytes().to_vec(), 0u128.to_le_bytes().to_vec());
        self.store("/total_shares".as_bytes().to_vec(), 0u128.to_le_bytes().to_vec());
        
        // Initialize fees accumulated to zero
        self.store("/fees_accumulated".as_bytes().to_vec(), 0u128.to_le_bytes().to_vec());
        
        // Set virtual assets and shares to prevent inflation attacks
        let virtual_assets: u128 = 1000;
        let virtual_shares: u128 = 1000;
        self.store("/virtual_assets".as_bytes().to_vec(), virtual_assets.to_le_bytes().to_vec());
        self.store("/virtual_shares".as_bytes().to_vec(), virtual_shares.to_le_bytes().to_vec());
        
        // Set default fees
        let deposit_fee_bp: u128 = 100; // 1%
        let withdraw_fee_bp: u128 = 50; // 0.5%
        self.store("/deposit_fee_bp".as_bytes().to_vec(), deposit_fee_bp.to_le_bytes().to_vec());
        self.store("/withdraw_fee_bp".as_bytes().to_vec(), withdraw_fee_bp.to_le_bytes().to_vec());
        
        // Store the end block (height + 144*100) - 100 periods of 144 blocks
        let end_block = height + 144 * 100;
        self.store("/end_block".as_bytes().to_vec(), end_block.to_le_bytes().to_vec());
        
        // Mint auth token during initialization
        response.alkanes = context.incoming_alkanes.clone();
        response.alkanes.0.push(AlkaneTransfer {
            id: context.myself.clone(),
            value: 1,   
        });
        
        response.data = "Initialized".as_bytes().to_vec();
        Ok(response)
    }
    
    fn authenticate(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response: CallResponse = CallResponse::forward(&context.incoming_alkanes.clone());

        if context.incoming_alkanes.0.len() != 1 {
            return Err(anyhow!(
                "did not authenticate with only the authentication token"
            ));
        }
        let transfer = context.incoming_alkanes.0[0].clone();
        if transfer.id != context.myself.clone() {
            return Err(anyhow!("supplied alkane is not authentication token"));
        }
        if transfer.value < 1 {
            return Err(anyhow!(
                "less than 1 unit of authentication token supplied to authenticate"
            ));
        }
        response.data = vec![0x01];
        response.alkanes.0.push(transfer);
        Ok(response)
    }
    
    fn set_currency_alkane(&self, block: u128, tx: u128) -> Result<CallResponse> {
        // self.authenticate()?;
        
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes.clone());
        
        // Create the AlkaneId struct directly with the provided values
        let alkane_id_struct = AlkaneId { block, tx };
        
        // Store the currency alkane ID using low-level store method
        let currency_key = "/currency_alkane".as_bytes().to_vec();
        self.store(currency_key, <AlkaneId as Into<Vec<u8>>>::into(alkane_id_struct));
        
        response.data = format!("Currency alkane set successfully to {}:{}", block, tx).into_bytes();
        Ok(response)
    }
    
    fn get_currency_alkane(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes.clone());
        
        // Try to get the stored currency alkane ID using low-level load method
        let currency_key = "/currency_alkane".as_bytes().to_vec();
        let bytes = self.load(currency_key);
        
        if bytes.len() > 0 {
            // Try to parse the bytes as an AlkaneId
            if bytes.len() >= 32 {  // AlkaneId has two u128 fields (16 bytes each)
                // Extract block and tx from the bytes
                // AlkaneId is stored as two u128 values in little-endian format
                let block_bytes: [u8; 16] = bytes[0..16].try_into().unwrap_or([0; 16]);
                let tx_bytes: [u8; 16] = bytes[16..32].try_into().unwrap_or([0; 16]);
                
                let block = u128::from_le_bytes(block_bytes);
                let tx = u128::from_le_bytes(tx_bytes);
                
                response.data = format!("Currency alkane: {}:{}", block, tx).into_bytes();
            } else {
                // If the bytes don't match the expected format, return them as hex
                let hex_string = bytes.iter()
                    .map(|b| format!("{:02x}", b))
                    .collect::<String>();
                
                response.data = format!("Currency alkane bytes (invalid format): {}", hex_string).into_bytes();
            }
        } else {
            response.data = "No currency alkane set".as_bytes().to_vec();
        }
        
        Ok(response)
    }

    fn get_name(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes.clone());

        // Try to get the stored name, or use default
        let name_key = "/name".as_bytes().to_vec();
        let name_bytes = self.load(name_key);
        
        if name_bytes.len() > 0 {
            response.data = name_bytes;
        } else {
            response.data = self.name().into_bytes();
        }
        
        Ok(response)
    }

    fn get_symbol(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes.clone());

        // Try to get the stored symbol, or use default
        let symbol_key = "/symbol".as_bytes().to_vec();
        let symbol_bytes = self.load(symbol_key);
        
        if symbol_bytes.len() > 0 {
            response.data = symbol_bytes;
        } else {
            response.data = self.symbol().into_bytes();
        }
        
        Ok(response)
    }

    fn get_height(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes.clone());
        
        let height = self.height();
        // Convert the height to a string for better readability
        let height_str = format!("Height: {}", height);
        response.data = height_str.into_bytes();
        
        Ok(response)
    }
    
    fn get_end_block(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes.clone());
        
        // Load the end block from storage directly
        let end_block_key = "/end_block".as_bytes().to_vec();
        let end_block_bytes = self.load(end_block_key);
        
        if end_block_bytes.len() >= 16 {
            // Parse the end block value from bytes
            let end_block_bytes_array: [u8; 16] = end_block_bytes[0..16].try_into().unwrap_or([0; 16]);
            let end_block = u128::from_le_bytes(end_block_bytes_array);
            let end_block_str = format!("End Block: {}", end_block);
            response.data = end_block_str.into_bytes();
        } else {
            response.data = "End block not set or invalid format".as_bytes().to_vec();
        }
        
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
    
    // Helper function to load currency alkane ID
    fn load_currency_alkane(&self) -> Result<AlkaneId> {
        let bytes = self.load("/currency_alkane".as_bytes().to_vec());
        if bytes.len() < 32 {
            return Err(anyhow!("Currency alkane not set"));
        }
        
        let block_bytes: [u8; 16] = bytes[0..16].try_into().unwrap_or([0; 16]);
        let tx_bytes: [u8; 16] = bytes[16..32].try_into().unwrap_or([0; 16]);
        
        let block = u128::from_le_bytes(block_bytes);
        let tx = u128::from_le_bytes(tx_bytes);
        
        Ok(AlkaneId { block, tx })
    }
    
    // Helper function to load reward alkane ID
    fn load_reward_alkane(&self) -> Result<AlkaneId> {
        let bytes = self.load("/reward_alkane".as_bytes().to_vec());
        if bytes.len() < 32 {
            // If reward alkane is not set, use the contract's ID as the reward token
            let context = self.context()?;
            return Ok(context.myself.clone());
        }
        
        let block_bytes: [u8; 16] = bytes[0..16].try_into().unwrap_or([0; 16]);
        let tx_bytes: [u8; 16] = bytes[16..32].try_into().unwrap_or([0; 16]);
        
        let block = u128::from_le_bytes(block_bytes);
        let tx = u128::from_le_bytes(tx_bytes);
        
        Ok(AlkaneId { block, tx })
    }
    
    // Function to set reward alkane
    fn set_reward_alkane(&self, block: u128, tx: u128) -> Result<CallResponse> {
        // Authenticate first
        self.authenticate()?;
        
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes.clone());
        
        // Create the AlkaneId struct directly with the provided values
        let alkane_id_struct = AlkaneId { block, tx };
        
        // Store the reward alkane ID
        self.store("/reward_alkane".as_bytes().to_vec(), <AlkaneId as Into<Vec<u8>>>::into(alkane_id_struct));
        
        response.data = format!("Reward alkane set successfully to {}:{}", block, tx).into_bytes();
        Ok(response)
    }
    
    // Function to get reward alkane
    fn get_reward_alkane(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes.clone());
        
        // Try to get the stored reward alkane ID
        let bytes = self.load("/reward_alkane".as_bytes().to_vec());
        
        if bytes.len() > 0 {
            // Try to parse the bytes as an AlkaneId
            if bytes.len() >= 32 {  // AlkaneId has two u128 fields (16 bytes each)
                // Extract block and tx from the bytes
                let block_bytes: [u8; 16] = bytes[0..16].try_into().unwrap_or([0; 16]);
                let tx_bytes: [u8; 16] = bytes[16..32].try_into().unwrap_or([0; 16]);
                
                let block = u128::from_le_bytes(block_bytes);
                let tx = u128::from_le_bytes(tx_bytes);
                
                response.data = format!("Reward alkane: {}:{}", block, tx).into_bytes();
            } else {
                // If the bytes don't match the expected format, return them as hex
                let hex_string = bytes.iter()
                    .map(|b| format!("{:02x}", b))
                    .collect::<String>();
                
                response.data = format!("Reward alkane bytes (invalid format): {}", hex_string).into_bytes();
            }
        } else {
            // If reward alkane is not set, use the contract's ID as the reward token
            let context_id = context.myself.clone();
            response.data = format!("Using contract ID as reward alkane: {}:{}", context_id.block, context_id.tx).into_bytes();
        }
        
        Ok(response)
    }
    
    // Function to update rewards based on current block height
    fn update_rewards(&self) -> Result<u128> {
        let current_height = u128::from(self.height());
        let last_reward_block = self.load_u128("/last_reward_block");
        let reward_period_length = self.load_u128("/reward_period_length");
        
        // Calculate how many complete periods have passed
        let blocks_passed = if current_height > last_reward_block {
            current_height - last_reward_block
        } else {
            0
        };
        
        let periods_passed = blocks_passed / reward_period_length;
        
        if periods_passed == 0 {
            // No new rewards to add
            return Ok(self.load_u128("/available_rewards"));
        }
        
        // Calculate new rewards (1% of max supply per period)
        let max_reward_supply = self.load_u128("/max_reward_supply");
        let reward_per_period = max_reward_supply / 100; // 1%
        let new_rewards = reward_per_period * periods_passed;
        
        // Check if we would exceed max supply
        let total_rewards_minted = self.load_u128("/total_rewards_minted");
        let remaining_rewards = max_reward_supply - total_rewards_minted;
        
        let actual_new_rewards = if new_rewards > remaining_rewards {
            remaining_rewards
        } else {
            new_rewards
        };
        
        // Update available rewards
        let mut available_rewards = self.load_u128("/available_rewards");
        available_rewards += actual_new_rewards;
        self.store("/available_rewards".as_bytes().to_vec(), available_rewards.to_le_bytes().to_vec());
        
        // Update last reward block
        // Only move forward by the number of periods we've actually processed
        let new_last_reward_block = last_reward_block + (periods_passed * reward_period_length);
        self.store("/last_reward_block".as_bytes().to_vec(), new_last_reward_block.to_le_bytes().to_vec());
        
        Ok(available_rewards)
    }
    
    // Function to check if deposits are allowed (anti-exploitation)
    fn check_deposit_allowed(&self) -> bool {
        let current_height = u128::from(self.height());
        let last_reward_block = self.load_u128("/last_reward_block");
        let reward_period_length = self.load_u128("/reward_period_length");
        let anti_exploit_window = self.load_u128("/anti_exploit_window");
        
        // Calculate the next reward block
        let next_reward_block = last_reward_block + reward_period_length;
        
        // Check if we're within the anti-exploitation window
        let blocks_until_reward = if next_reward_block > current_height {
            next_reward_block - current_height
        } else {
            0
        };
        
        // Deposits are not allowed if we're within the anti-exploitation window
        blocks_until_reward > anti_exploit_window
    }
    
    // Stub functions for message dispatch opcodes
    
    // Deposit function stub
    fn deposit(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes.clone());
        
        // Check if deposits are allowed
        if !self.check_deposit_allowed() {
            return Err(anyhow!("Deposits are not allowed at this time due to anti-exploitation protection"));
        }
        
        // For now, just return a message that the function is not fully implemented
        response.data = "Deposit function not yet fully implemented".as_bytes().to_vec();
        Ok(response)
    }
    
    // Withdraw function stub
    fn withdraw(&self) -> Result<CallResponse> {
        Err(anyhow!("Withdraw function not yet implemented"))
    }
    
    // Preview deposit function stub
    fn preview_deposit(&self, assets: u128) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes.clone());
        
        // For now, just return the assets as shares (1:1 ratio)
        response.data = assets.to_le_bytes().to_vec();
        
        Ok(response)
    }
    
    // Preview withdraw function stub
    fn preview_withdraw(&self, shares: u128) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes.clone());
        
        // For now, just return the shares as assets (1:1 ratio)
        response.data = shares.to_le_bytes().to_vec();
        
        Ok(response)
    }
    
    // Convert to shares function stub
    fn convert_to_shares(&self, assets: u128) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes.clone());
        
        // For now, just return the assets as shares (1:1 ratio)
        response.data = assets.to_le_bytes().to_vec();
        
        Ok(response)
    }
    
    // Convert to assets function stub
    fn convert_to_assets(&self, shares: u128) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes.clone());
        
        // For now, just return the shares as assets (1:1 ratio)
        response.data = shares.to_le_bytes().to_vec();
        
        Ok(response)
    }
    
    // Claim rewards function stub
    fn claim_rewards(&self) -> Result<CallResponse> {
        Err(anyhow!("Claim rewards function not yet implemented"))
    }
    
    // Helper function to calculate the next reward block
    fn calculate_next_reward_block(&self) -> u128 {
        let last_reward_block = self.load_u128("/last_reward_block");
        let reward_period_length = self.load_u128("/reward_period_length");
        
        last_reward_block + reward_period_length
    }
    
    // Getter functions for new storage variables
    fn get_total_assets(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes.clone());
        
        let total_assets = self.load_u128("/total_assets");
        response.data = total_assets.to_le_bytes().to_vec();
        
        Ok(response)
    }
    
    fn get_total_shares(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes.clone());
        
        let total_shares = self.load_u128("/total_shares");
        response.data = total_shares.to_le_bytes().to_vec();
        
        Ok(response)
    }
    
    fn get_deposit_fee(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes.clone());
        
        let deposit_fee_bp = self.load_u128("/deposit_fee_bp");
        response.data = deposit_fee_bp.to_le_bytes().to_vec();
        
        Ok(response)
    }
    
    fn get_withdraw_fee(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes.clone());
        
        let withdraw_fee_bp = self.load_u128("/withdraw_fee_bp");
        response.data = withdraw_fee_bp.to_le_bytes().to_vec();
        
        Ok(response)
    }
    
    fn get_available_rewards(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes.clone());
        
        // Update rewards based on current block height
        let available_rewards = self.update_rewards()?;
        
        response.data = available_rewards.to_le_bytes().to_vec();
        Ok(response)
    }
    
    fn get_next_reward_block(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes.clone());
        
        let last_reward_block = self.load_u128("/last_reward_block");
        let reward_period_length = self.load_u128("/reward_period_length");
        let next_reward_block = last_reward_block + reward_period_length;
        
        response.data = next_reward_block.to_le_bytes().to_vec();
        Ok(response)
    }
    
    fn is_deposit_allowed(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes.clone());
        
        let current_height = u128::from(self.height());
        let last_reward_block = self.load_u128("/last_reward_block");
        let reward_period_length = self.load_u128("/reward_period_length");
        let anti_exploit_window = self.load_u128("/anti_exploit_window");
        
        // Calculate the next reward block
        let next_reward_block = last_reward_block + reward_period_length;
        
        // Check if we're within the anti-exploitation window
        let blocks_until_reward = if next_reward_block > current_height {
            next_reward_block - current_height
        } else {
            0
        };
        
        // Deposits are not allowed if we're within the anti-exploitation window
        let allowed = blocks_until_reward > anti_exploit_window;
        
        // Convert bool to u8 (0 or 1) and then to bytes
        let result: u8 = if allowed { 1 } else { 0 };
        response.data = vec![result];
        
        Ok(response)
    }
}

impl AlkaneResponder for YieldVault {}

// Use the new macro format
declare_alkane! {
    impl AlkaneResponder for YieldVault {
        type Message = YieldVaultMessage;
    }
}
