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

    #[opcode(100)]
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
}

impl YieldVault {

    fn initialize(&self) -> Result<CallResponse> {
        // Use the standard observe_initialization method which will throw an error if already initialized
        self.observe_initialization()?;
        
        let context = self.context()?;
        let mut response: CallResponse = CallResponse::forward(&context.incoming_alkanes.clone());
        
        // Store the height as binary data directly
        let height = self.height();
        let height_key = "/start_block".as_bytes().to_vec();
        let height_bytes = height.to_le_bytes().to_vec();
        self.store(height_key, height_bytes);
        
        // Store the end block (height + 10000) as binary data directly
        let end_block = height + 10000; // Example: 10000 blocks after initialization
        let end_block_key = "/end_block".as_bytes().to_vec();
        let end_block_bytes = end_block.to_le_bytes().to_vec();
        self.store(end_block_key, end_block_bytes);
        
        // // Hard code the currency alkane ID during initialization
        // // Using 7:123456 as the example alkane ID
        
        // // Store the currency alkane ID using StoragePointer for consistency
        // let currency_key = "/currency_alkane".as_bytes().to_vec();
        // self.store(currency_key, <AlkaneId as Into<Vec<u8>>>::into(alkane_id_struct));
        
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
}

impl AlkaneResponder for YieldVault {}

// Use the new macro format
declare_alkane! {
    impl AlkaneResponder for YieldVault {
        type Message = YieldVaultMessage;
    }
}
