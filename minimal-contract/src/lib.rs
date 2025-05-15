use alkanes_runtime::runtime::AlkaneResponder;
use alkanes_runtime::{declare_alkane, message::MessageDispatch, token::Token, storage::StoragePointer};
use alkanes_support::response::CallResponse;
use alkanes_support::id::AlkaneId;
use anyhow::{anyhow, Result};
use metashrew_support::compat::to_arraybuffer_layout;
use metashrew_support::index_pointer::KeyValuePointer;


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
    Initialize {
        amount: u128,
    },

    #[opcode(1)]
    Authenticate,
    
    #[opcode(2)]
    SetCurrencyAlkane {
        alkane_id: String,
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

    fn initialize(&self, _amount: u128) -> Result<CallResponse> {
        // Use the standard observe_initialization method which will throw an error if already initialized
        self.observe_initialization()?;
        
        let context = self.context()?;
        let mut response: CallResponse = CallResponse::forward(&context.incoming_alkanes.clone());
        
        // Store the height as binary data using StoragePointer
        let mut start_block_pointer = self.start_block_pointer();
        let height = self.height();
        start_block_pointer.set_value::<u128>(height.into());
        
        // Store the end block (height + 10000) as binary data using StoragePointer
        let mut end_block_pointer = self.end_block_pointer();
        let end_block = height + 10000; // Example: 10000 blocks after initialization
        end_block_pointer.set_value::<u128>(end_block.into());
        
        // Hard code the currency alkane ID during initialization
        // Using 7:123456 as the example alkane ID
        let alkane_id_struct = AlkaneId { block: 7, tx: 123456 };
        
        // Store the currency alkane ID using StoragePointer for consistency
        let currency_key = "/currency_alkane".as_bytes().to_vec();
        self.store(currency_key, <AlkaneId as Into<Vec<u8>>>::into(alkane_id_struct));
        
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
    
    fn set_currency_alkane(&self, _alkane_id: String) -> Result<CallResponse> {
        // First authenticate the caller
        // self.authenticate()?;
        
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes.clone());
        
        // Hard code the currency alkane ID
        // Using 8:987654 as the example alkane ID
        let alkane_id_struct = AlkaneId { block: 8, tx: 987654 };
        
        // Store the currency alkane ID using low-level store method
        let currency_key = "/currency_alkane".as_bytes().to_vec();
        self.store(currency_key, <AlkaneId as Into<Vec<u8>>>::into(alkane_id_struct));
        
        response.data = "Currency alkane set successfully (hard-coded to 8:987654)".as_bytes().to_vec();
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
            if bytes.len() >= 16 {
                // Extract block and tx from the bytes
                // AlkaneId is stored as two u64 values in little-endian format
                let block_bytes: [u8; 8] = bytes[0..8].try_into().unwrap_or([0; 8]);
                let tx_bytes: [u8; 8] = bytes[8..16].try_into().unwrap_or([0; 8]);
                
                let block = u64::from_le_bytes(block_bytes);
                let tx = u64::from_le_bytes(tx_bytes);
                
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
        let name_pointer = self.name_pointer();
        let name_bytes = name_pointer.get();
        
        if name_bytes.len() > 0 {
            response.data = name_bytes.to_vec();
        } else {
            response.data = self.name().into_bytes();
        }
        
        Ok(response)
    }

    fn get_symbol(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes.clone());

        // Try to get the stored symbol, or use default
        let symbol_pointer = self.symbol_pointer();
        let symbol_bytes = symbol_pointer.get();
        
        if symbol_bytes.len() > 0 {
            response.data = symbol_bytes.to_vec();
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
        
        // Load the end block from storage using StoragePointer
        let end_block_pointer = self.end_block_pointer();
        let end_block_bytes = end_block_pointer.get();
        
        if end_block_bytes.len() > 0 {
            // Get the end block value using get_value
            let end_block = end_block_pointer.get_value::<u128>();
            let end_block_str = format!("End Block: {}", end_block);
            response.data = end_block_str.into_bytes();
        }
        
        Ok(response)
    }
    
    // Storage pointer methods
    fn start_block_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/start_block")
    }
    
    fn end_block_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/end_block")
    }
    
    fn name_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/name")
    }
    
    fn symbol_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/symbol")
    }
}

impl AlkaneResponder for YieldVault {}

// Use the new macro format
declare_alkane! {
    impl AlkaneResponder for YieldVault {
        type Message = YieldVaultMessage;
    }
}
