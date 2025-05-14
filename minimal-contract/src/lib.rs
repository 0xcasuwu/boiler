use alkanes_runtime::runtime::AlkaneResponder;
use alkanes_runtime::{declare_alkane, message::MessageDispatch, token::Token};
use alkanes_support::response::CallResponse;
use anyhow::{anyhow, Result};
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
    #[returns(u128)]
    GetYieldRate,
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
            
            // Set a hard-coded yield rate (500 basis points = 5%)
            let rate = self.set_yield_rate(500)?;
            
            response.data = "Initialized".as_bytes().to_vec();
            Ok(response)
        } else {
            return Err(anyhow!("already initialized"));
        }
    }

    fn get_yield_rate(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes.clone());

        let yield_rate = self.yield_rate()?;
        let yield_rate_bytes = yield_rate.to_le_bytes().to_vec();
        response.data = yield_rate_bytes;
        
        Ok(response)
    }
    
    // Helper functions for yield rate
    fn yield_rate(&self) -> Result<u128> {
        let yield_rate_key = "/yield-rate".as_bytes().to_vec();
        let yield_rate_bytes = self.load(yield_rate_key);
        
        if yield_rate_bytes.len() == 16 {  // u128 is 16 bytes
            let mut bytes = [0u8; 16];
            bytes.copy_from_slice(&yield_rate_bytes);
            Ok(u128::from_le_bytes(bytes))
        } else {
            // Default to 0 if not found
            Ok(0)
        }
    }
    
    fn yield_rate_pointer(&self) -> alkanes_support::storage::StoragePointer {
        alkanes_support::storage::StoragePointer::from_keyword("/yield-rate")
    }
    
    fn set_yield_rate(&self, rate: u128) -> Result<u128> {
        // Check if the current rate exists
        let current_rate = self.yield_rate()?;
        
        // Create bytes for the new rate
        let rate_bytes = rate.to_le_bytes().to_vec();
        
        // Use the pointer approach similar to the template
        let mut rate_pointer = self.yield_rate_pointer();
        rate_pointer.set(Arc::new(rate_bytes));
        
        Ok(rate)
    }
}

impl AlkaneResponder for YieldVault {}

// Use the new macro format
declare_alkane! {
    impl AlkaneResponder for YieldVault {
        type Message = YieldVaultMessage;
    }
}
