use crate::contracts::bond_curve::BondCurve;
use crate::helpers;
use crate::{
    CallResponse,
    AlkaneTransferParcel,
    AlkaneResponder,
    MessageDispatch,
    Result,
    anyhow,
};

/// Wrapper for BondCurve contract
pub struct BondCurveWrapper {
    curve: BondCurve,
}

/// Message enum for BondCurve opcode-based dispatch
pub enum BondCurveMessage {
    /// Initialize the bond curve
    Initialize {
        /// Virtual input reserves
        virtual_input_reserves: u128,
        /// Virtual output reserves
        virtual_output_reserves: u128,
        /// Half life in blocks
        half_life: u128,
        /// Level floor in basis points
        level_bips: u128,
        /// Term in blocks
        term_blocks: u128,
    },
    
    /// Get current token price
    GetCurrentPrice {
        /// Available debt
        available_debt: u128,
    },
    
    /// Get amount out for a given input
    GetAmountOut {
        /// Input amount
        input: u128,
        /// Available debt
        available_debt: u128,
    },
    
    /// Get redeem amount
    GetRedeemAmount {
        /// Amount owed
        owed: u128,
        /// Amount redeemed
        redeemed: u128,
        /// Creation block number
        creation_block: u128,
    },
    
    /// Purchase a bond
    PurchaseBond {
        /// Input amount
        input_amount: u128,
        /// Available debt
        available_debt: u128,
    },
}

impl Default for BondCurveWrapper {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageDispatch<BondCurveWrapper> for BondCurveMessage {
    fn from_opcode(opcode: u128, inputs: Vec<u128>) -> Result<Self> {
        match opcode {
            0 => {
                // Initialize
                if inputs.len() < 5 {
                    return Err(anyhow!("Initialize: Not enough inputs"));
                }
                
                let virtual_input_reserves = inputs[0];
                let virtual_output_reserves = inputs[1];
                let half_life = inputs[2];
                let level_bips = inputs[3];
                let term_blocks = inputs[4];
                
                Ok(BondCurveMessage::Initialize {
                    virtual_input_reserves,
                    virtual_output_reserves,
                    half_life,
                    level_bips,
                    term_blocks,
                })
            },
            100 => {
                // GetCurrentPrice
                if inputs.is_empty() {
                    return Err(anyhow!("GetCurrentPrice: Not enough inputs"));
                }
                
                let available_debt = inputs[0];
                
                Ok(BondCurveMessage::GetCurrentPrice {
                    available_debt,
                })
            },
            101 => {
                // GetAmountOut
                if inputs.len() < 2 {
                    return Err(anyhow!("GetAmountOut: Not enough inputs"));
                }
                
                let input = inputs[0];
                let available_debt = inputs[1];
                
                Ok(BondCurveMessage::GetAmountOut {
                    input,
                    available_debt,
                })
            },
            102 => {
                // GetRedeemAmount
                if inputs.len() < 3 {
                    return Err(anyhow!("GetRedeemAmount: Not enough inputs"));
                }
                
                let owed = inputs[0];
                let redeemed = inputs[1];
                let creation_block = inputs[2];
                
                Ok(BondCurveMessage::GetRedeemAmount {
                    owed,
                    redeemed,
                    creation_block,
                })
            },
            103 => {
                // PurchaseBond
                if inputs.len() < 2 {
                    return Err(anyhow!("PurchaseBond: Not enough inputs"));
                }
                
                let input_amount = inputs[0];
                let available_debt = inputs[1];
                
                Ok(BondCurveMessage::PurchaseBond {
                    input_amount,
                    available_debt,
                })
            },
            _ => Err(anyhow!("Unknown opcode: {}", opcode)),
        }
    }
    
    fn dispatch(&self, wrapper: &BondCurveWrapper) -> Result<CallResponse> {
        let mut mutable_wrapper = BondCurveWrapper::new(); // Create a new instance for mutation
        mutable_wrapper.curve = wrapper.curve.clone(); // Clone the curve data
        
        match self {
            BondCurveMessage::Initialize { 
                virtual_input_reserves, 
                virtual_output_reserves, 
                half_life, 
                level_bips, 
                term_blocks 
            } => {
                mutable_wrapper.init_curve(
                    *virtual_input_reserves, 
                    *virtual_output_reserves, 
                    *half_life, 
                    *level_bips, 
                    *term_blocks
                )
            },
            BondCurveMessage::GetCurrentPrice { available_debt } => {
                mutable_wrapper.get_current_price(*available_debt)
            },
            BondCurveMessage::GetAmountOut { input, available_debt } => {
                mutable_wrapper.get_amount_out(*input, *available_debt)
            },
            BondCurveMessage::GetRedeemAmount { owed, redeemed, creation_block } => {
                mutable_wrapper.get_redeem_amount(*owed, *redeemed, *creation_block)
            },
            BondCurveMessage::PurchaseBond { input_amount, available_debt } => {
                mutable_wrapper.purchase_bond(*input_amount, *available_debt)
            }
        }
    }
    
    fn export_abi() -> Vec<u8> {
        // This would generate an ABI description of the interface
        // For now, return a simple indication that it's implemented
        "BondCurveMessage ABI".as_bytes().to_vec()
    }
}

impl BondCurveWrapper {
    pub fn new() -> Self {
        // Create a placeholder curve that will be initialized later
        Self {
            curve: BondCurve::new(
                100000,  // Default virtual input reserves
                50000,   // Default virtual output reserves
                3600,    // Default half-life (1 hour in blocks)
                5000,    // Default level floor (50%)
                86400,   // Default term (1 day in blocks)
            ),
        }
    }

    pub fn init_curve(
        &mut self, 
        virtual_input_reserves: u128, 
        virtual_output_reserves: u128, 
        half_life: u128, 
        level_bips: u128, 
        term_blocks: u128
    ) -> Result<CallResponse> {
        // Convert MessageDispatch compatible types to contract types
        let half_life_u64 = helpers::u128_to_u64(half_life);
        let level_bips_u64 = helpers::u128_to_u64(level_bips);
        let term_blocks_u64 = helpers::u128_to_u64(term_blocks);
        
        // Initialize is a constructor, just update our curve instance
        self.curve = BondCurve::new(
            virtual_input_reserves,
            virtual_output_reserves,
            half_life_u64,
            level_bips_u64,
            term_blocks_u64
        );
        
        // Return success response with log
        Ok(helpers::call_response_with_log("Bond curve initialized".to_string()))
    }

    pub fn get_current_price(&self, available_debt: u128) -> Result<CallResponse> {
        // Call the actual contract method
        let price = self.curve.get_current_price(available_debt);
        
        // Return the price as a CallResponse
        Ok(CallResponse {
            alkanes: AlkaneTransferParcel::default(),
            data: serde_json::to_vec(&price).unwrap_or_default(),
        })
    }

    pub fn get_amount_out(&self, input: u128, available_debt: u128) -> Result<CallResponse> {
        // Call the actual contract method
        let amount_out = self.curve.get_amount_out(input, available_debt);
        
        // Return the amount as a CallResponse
        Ok(CallResponse {
            alkanes: AlkaneTransferParcel::default(),
            data: serde_json::to_vec(&amount_out).unwrap_or_default(),
        })
    }

    pub fn get_redeem_amount(&self, owed: u128, redeemed: u128, creation_block: u128) -> Result<CallResponse> {
        // Convert from MessageDispatch u128 to contract u64
        let creation_block_u64 = helpers::u128_to_u64(creation_block);
        
        // Call the actual contract method
        let result = self.curve.get_redeem_amount(owed, redeemed, creation_block_u64);
        
        // Convert the result to CallResponse
        match result {
            Ok(amount) => {
                Ok(CallResponse {
                    alkanes: AlkaneTransferParcel::default(),
                    data: serde_json::to_vec(&amount).unwrap_or_default(),
                })
            },
            Err(e) => Err(anyhow!(e.to_string())),
        }
    }

    pub fn purchase_bond(&self, input_amount: u128, available_debt: u128) -> Result<CallResponse> {
        // This would normally integrate with a bond purchasing system
        // For now, we'll just calculate the output amount
        let output_amount = self.curve.get_amount_out(input_amount, available_debt);
        
        Ok(CallResponse {
            alkanes: AlkaneTransferParcel::default(),
            data: serde_json::to_vec(&output_amount).unwrap_or_default(),
        })
    }
}

// Implement AlkaneResponder to make the wrapper compatible with the runtime
impl AlkaneResponder for BondCurveWrapper {
    // Add any required AlkaneResponder methods here if needed
}

// Constructor is exported from lib.rs instead
