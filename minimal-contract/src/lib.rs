use alkanes_runtime::runtime::AlkaneResponder;
use alkanes_runtime::{declare_alkane, message::MessageDispatch, token::Token};
use alkanes_support::{context::Context, response::CallResponse};
use anyhow::{anyhow, Result};
use metashrew_support::compat::to_arraybuffer_layout;
use std::sync::Arc;

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
}

impl YieldVault {
    fn initialize(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes.clone());

        // Check if already initialized
        let initialized_key = "/initialized".as_bytes().to_vec();
        if self.load(initialized_key.clone()).len() == 0 {
            // Mark as initialized
            self.store(initialized_key, vec![0x01]);
            
            // Hard-coded arbitrary values
            let name = String::from("OylVault");
            let symbol = String::from("OYL");
            let asset_name = String::from("Bitcoin");
            let asset_symbol = String::from("BTC");
            let decimal_offset = 8u128;
            
            // Store basic token metadata
            self.store("/name".as_bytes().to_vec(), name.as_bytes().to_vec());
            self.store("/symbol".as_bytes().to_vec(), symbol.as_bytes().to_vec());
            self.store("/asset-name".as_bytes().to_vec(), asset_name.as_bytes().to_vec());
            self.store("/asset-symbol".as_bytes().to_vec(), asset_symbol.as_bytes().to_vec());
            
            // Convert u128 to u8 for decimal_offset, as needed
            let decimal_offset_u8 = if decimal_offset > u8::MAX.into() {
                // Default to 8 if out of range
                8u8
            } else {
                decimal_offset as u8
            };
            self.store("/decimals".as_bytes().to_vec(), vec![decimal_offset_u8]);
            
            // Initialize accounting state
            self.store("/total-supply".as_bytes().to_vec(), to_arraybuffer_layout(&0u128));
            self.store("/total-assets".as_bytes().to_vec(), to_arraybuffer_layout(&0u128));
            
            // Initialize yield rate (basis points, e.g. 500 = 5%)
            self.store("/yield-rate".as_bytes().to_vec(), to_arraybuffer_layout(&0u128));
            
            // Initialize the last yield block height
            let height = context.height;
            self.store("/last-yield-height".as_bytes().to_vec(), to_arraybuffer_layout(&height));
            
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
