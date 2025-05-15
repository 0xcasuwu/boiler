use alkanes_runtime::runtime::AlkaneResponder;
use alkanes_runtime::{declare_alkane, message::MessageDispatch, token::Token};
use alkanes_support::response::CallResponse;
use anyhow::{anyhow, Result};
use metashrew_support::compat::to_arraybuffer_layout;


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
}

impl YieldVault {

    
    // Method for MessageDispatch to get height
    fn get_height(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes.clone());
        
        let height = self.height();
        response.data = height.to_le_bytes().to_vec();
        
        Ok(response)
    }
    
    // Method for MessageDispatch to get end block
    fn get_end_block(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes.clone());
        
        // Load the end block from storage
        let end_block_key = "/end_block".as_bytes().to_vec();
        let end_block_bytes = self.load(end_block_key);
        
        if end_block_bytes.len() > 0 {
            response.data = end_block_bytes;
        } else {
            // If not found, use current height + 10000 as default
            let height = self.height();
            let end_block = height + 10000;
            response.data = end_block.to_le_bytes().to_vec();
        }
        
        Ok(response)
    }

    fn initialize(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes.clone());

        // Check if already initialized
        let initialized_key = "/initialized".as_bytes().to_vec();
        if self.load(initialized_key.clone()).len() == 0 {

            // Mark as initialized
            self.store(initialized_key, vec![0x01]);
            // Store the height as binary data
            let height_key = "/start_block".as_bytes().to_vec();
            let height = self.height();
            let height_bytes = height.to_le_bytes().to_vec();
            self.store(height_key, height_bytes);
            
            // Store the end block (height + 10000) as binary data
            let end_block_key = "/end_block".as_bytes().to_vec();
            let end_block = height + 10000; // Example: 10000 blocks after initialization
            let end_block_bytes = end_block.to_le_bytes().to_vec();
            self.store(end_block_key, end_block_bytes);


            response.data = "Initialized".as_bytes().to_vec();
            Ok(response)
        } else {
            return Err(anyhow!("already initialized"));
        }
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
            response.data = self.name().into_bytes().to_vec();
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
            response.data = self.symbol().into_bytes().to_vec();
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
