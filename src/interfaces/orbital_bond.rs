use crate::contracts::orbital_bond_collection::OrbitalBondCollection;
use crate::utils::{StandaloneBlockContext, MockTransactionContext};
use crate::helpers;
use crate::{
    CallResponse,
    AlkaneTransferParcel,
    AlkaneResponder,
    MessageDispatch,
    Result,
    anyhow,
};

/// Wrapper for OrbitalBondCollection contract
pub struct OrbitalBondWrapper {
    collection: OrbitalBondCollection,
}

/// Message enum for OrbitalBondCollection opcode-based dispatch
pub enum OrbitalBondMessage {
    /// Initialize a new bond collection
    Initialize {
        /// Collection ID
        id: String,
        /// Collection name
        name: String,
        /// Collection symbol
        symbol: String,
        /// Interest rate in basis points
        interest_rate_bps: u128,
        /// Maturity period in blocks
        maturity_blocks: u128,
        /// Collection description (optional)
        description: String,
    },
    
    /// Mint a new bond
    MintBond {
        /// Orbital token ID
        orbital_token_id: String,
        /// Bond amount
        amount: u128,
        /// Owner ID
        owner_id: String,
    },
    
    /// Redeem a bond
    RedeemBond {
        /// Redeemer ID
        redeemer_id: String,
    },
    
    /// Get collection info
    GetInfo,
    
    /// Get bond details
    GetBond {
        /// Bond ID to retrieve
        bond_id: String,
    },
    
    /// Mint tokens (placeholder)
    MintTokens,
    
    /// Get collection name
    GetName,
    
    /// Get collection symbol
    GetSymbol,
}

impl Default for OrbitalBondWrapper {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageDispatch<OrbitalBondWrapper> for OrbitalBondMessage {
    fn from_opcode(opcode: u128, inputs: Vec<u128>) -> Result<Self> {
        match opcode {
            0 => {
                // Initialize
                if inputs.len() < 6 {
                    return Err(anyhow!("Initialize: Not enough inputs"));
                }
                
                // Convert binary data inputs to strings
                let id = helpers::u128_vec_to_string(&inputs[0..1])?;
                let name = helpers::u128_vec_to_string(&inputs[1..2])?;
                let symbol = helpers::u128_vec_to_string(&inputs[2..3])?;
                let interest_rate_bps = inputs[3];
                let maturity_blocks = inputs[4];
                let description = helpers::u128_vec_to_string(&inputs[5..6])?;
                
                Ok(OrbitalBondMessage::Initialize {
                    id,
                    name,
                    symbol,
                    interest_rate_bps,
                    maturity_blocks,
                    description,
                })
            },
            1 => {
                // MintBond
                if inputs.len() < 3 {
                    return Err(anyhow!("MintBond: Not enough inputs"));
                }
                
                let orbital_token_id = helpers::u128_vec_to_string(&inputs[0..1])?;
                let amount = inputs[1];
                let owner_id = helpers::u128_vec_to_string(&inputs[2..3])?;
                
                Ok(OrbitalBondMessage::MintBond {
                    orbital_token_id,
                    amount,
                    owner_id,
                })
            },
            2 => {
                // RedeemBond
                if inputs.is_empty() {
                    return Err(anyhow!("RedeemBond: Not enough inputs"));
                }
                
                let redeemer_id = helpers::u128_vec_to_string(&inputs[0..1])?;
                
                Ok(OrbitalBondMessage::RedeemBond {
                    redeemer_id,
                })
            },
            100 => Ok(OrbitalBondMessage::GetInfo),
            101 => {
                // GetBond
                if inputs.is_empty() {
                    return Err(anyhow!("GetBond: Not enough inputs"));
                }
                
                let bond_id = helpers::u128_vec_to_string(&inputs[0..1])?;
                
                Ok(OrbitalBondMessage::GetBond {
                    bond_id,
                })
            },
            102 => Ok(OrbitalBondMessage::MintTokens),
            103 => Ok(OrbitalBondMessage::GetName),
            104 => Ok(OrbitalBondMessage::GetSymbol),
            _ => Err(anyhow!("Unknown opcode: {}", opcode)),
        }
    }
    
    fn dispatch(&self, wrapper: &OrbitalBondWrapper) -> Result<CallResponse> {
        let mut mutable_wrapper = OrbitalBondWrapper::new(); // Create a new instance for mutation
        mutable_wrapper.collection = wrapper.collection.clone(); // Clone the collection data
        
        match self {
            OrbitalBondMessage::Initialize { id, name, symbol, interest_rate_bps, maturity_blocks, description } => {
                mutable_wrapper.init_bond_collection(id.clone(), name.clone(), symbol.clone(), *interest_rate_bps, *maturity_blocks, description.clone())
            },
            OrbitalBondMessage::MintBond { orbital_token_id, amount, owner_id } => {
                mutable_wrapper.mint_bond(orbital_token_id.clone(), *amount, owner_id.clone())
            },
            OrbitalBondMessage::RedeemBond { redeemer_id } => {
                mutable_wrapper.redeem_bond(redeemer_id.clone())
            },
            OrbitalBondMessage::GetInfo => {
                mutable_wrapper.get_info()
            },
            OrbitalBondMessage::GetBond { bond_id } => {
                mutable_wrapper.get_bond(bond_id.clone())
            },
            OrbitalBondMessage::MintTokens => {
                mutable_wrapper.mint_tokens()
            },
            OrbitalBondMessage::GetName => {
                mutable_wrapper.get_name()
            },
            OrbitalBondMessage::GetSymbol => {
                mutable_wrapper.get_symbol()
            }
        }
    }
    
    fn export_abi() -> Vec<u8> {
        // This would generate an ABI description of the interface
        // For now, return a simple indication that it's implemented
        "OrbitalBondMessage ABI".as_bytes().to_vec()
    }
}

impl OrbitalBondWrapper {
    pub fn new() -> Self {
        let context = StandaloneBlockContext::new();
        Self {
            collection: OrbitalBondCollection::new(
                "placeholder".to_string(),
                "placeholder".to_string(),
                "PLC".to_string(),
                500, // Default interest rate as u16
                100, // Default maturity period
                &context,
            ),
        }
    }
    
    pub fn init_bond_collection(&mut self, id: String, name: String, symbol: String, interest_rate_bps: u128, maturity_blocks: u128, description: String) -> Result<CallResponse> {
        // Convert MessageDispatch compatible types to contract types
        let interest_rate = helpers::u128_to_u16(interest_rate_bps);
        let maturity = helpers::u128_to_u64(maturity_blocks);
        let desc_option = helpers::string_to_option_string(description);
        
        // Create a new collection with the parameters
        let context = StandaloneBlockContext::new();
        let mut new_collection = OrbitalBondCollection::new(
            id.clone(), name.clone(), symbol, interest_rate, maturity, 
            &context
        );
        
        // Conditionally add the description
        if let Some(desc) = desc_option {
            // Create a new collection with description (handles ownership properly)
            new_collection = OrbitalBondCollection::new(
                id, name.clone(), new_collection.symbol.clone(), 
                interest_rate, maturity, &context
            ).with_description(desc);
        }
        
        // Update our stored collection
        self.collection = new_collection;
        
        // Return success response with log
        Ok(helpers::call_response_with_log(format!("Initialized collection: {}", name)))
    }
    
    pub fn mint_bond(&mut self, orbital_token_id: String, amount: u128, owner_id: String) -> Result<CallResponse> {
        // Convert u128 to u64 for contract call
        let amount_u64 = helpers::u128_to_u64(amount);
        let context = StandaloneBlockContext::new();
        
        // Call the actual contract method with proper parameter order
        let result = self.collection.mint_bond(
            orbital_token_id,
            amount_u64, 
            owner_id,
            &context
        );
        
        match result {
            Ok(mint_result) => {
                Ok(CallResponse {
                    alkanes: AlkaneTransferParcel::default(),
                    data: serde_json::to_vec(&mint_result).unwrap_or_default(),
                })
            },
            Err(err) => Err(anyhow!(err)),
        }
    }
    
    pub fn redeem_bond(&mut self, redeemer_id: String) -> Result<CallResponse> {
        let context = StandaloneBlockContext::new();
        let tx_context = MockTransactionContext::new_with_caller("test-caller");
        
        // Call the contract method with correct parameters
        let result = self.collection.redeem_bond_secure(
            &tx_context,       // First param is TransactionContextExt
            &redeemer_id,      // Second param is the redeemer_id string  
            &context           // Third param is BlockContext
        );
        
        // Convert the result to a CallResponse
        match result {
            Ok(redeem_amount) => {
                Ok(CallResponse {
                    alkanes: AlkaneTransferParcel::default(),
                    data: serde_json::to_vec(&redeem_amount).unwrap_or_default(),
                })
            },
            Err(err) => Err(anyhow!(err)),
        }
    }
    
    pub fn get_info(&self) -> Result<CallResponse> {
        // Get collection info as JSON
        let info = serde_json::json!({
            "name": self.collection.name,
            "symbol": self.collection.symbol,
            "active": self.collection.is_active(),
            "total_bonds": self.collection.total_bonds(),
        });
        
        // Convert to CallResponse with data
        Ok(CallResponse {
            alkanes: AlkaneTransferParcel::default(),
            data: serde_json::to_vec(&info).unwrap_or_default(),
        })
    }
    
    pub fn get_bond(&self, bond_id: String) -> Result<CallResponse> {
        // Get bond info
        if let Some(bond) = self.collection.get_bond(&bond_id) {
            let bond_json = serde_json::to_value(bond).unwrap_or_default();
            Ok(CallResponse {
                alkanes: AlkaneTransferParcel::default(),
                data: serde_json::to_vec(&bond_json).unwrap_or_default(),
            })
        } else {
            Ok(helpers::call_response_with_log(format!("Bond not found: {}", bond_id)))
        }
    }
    
    pub fn mint_tokens(&mut self) -> Result<CallResponse> {
        // This is a placeholder - in a real implementation it would integrate with a token system
        Ok(helpers::call_response_with_log("Tokens minted".to_string()))
    }
    
    pub fn get_name(&self) -> Result<CallResponse> {
        Ok(CallResponse {
            alkanes: AlkaneTransferParcel::default(),
            data: self.collection.name.as_bytes().to_vec(),
        })
    }
    
    pub fn get_symbol(&self) -> Result<CallResponse> {
        Ok(CallResponse {
            alkanes: AlkaneTransferParcel::default(),
            data: self.collection.symbol.as_bytes().to_vec(),
        })
    }
}

// Implement AlkaneResponder to make the wrapper compatible with the runtime
impl AlkaneResponder for OrbitalBondWrapper {
    // Add any required AlkaneResponder methods here if needed
}

// Constructor is exported from lib.rs instead
