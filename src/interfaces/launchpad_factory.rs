use crate::contracts::launchpad_factory::LaunchpadFactory;
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

/// Wrapper for LaunchpadFactory contract
pub struct LaunchpadFactoryWrapper {
    factory: LaunchpadFactory,
}

/// Message enum for LaunchpadFactory opcode-based dispatch
pub enum LaunchpadFactoryMessage {
    /// Initialize the factory
    Initialize {
        /// Version string
        version: String,
        /// Default maturity blocks
        default_maturity_blocks: u128,
        /// Default interest rate in basis points
        default_interest_rate_bps: u128,
    },
    
    /// Create a new bond collection
    CreateCollection {
        /// Collection name
        name: String,
        /// Collection symbol
        symbol: String,
        /// Collection description
        description: String,
        /// Interest rate in basis points
        interest_rate_bps: u128,
        /// Maturity period in blocks
        maturity_blocks: u128,
    },
    
    /// Mint a new bond
    MintBond {
        /// Collection ID
        collection_id: String,
        /// Orbital token ID
        orbital_token_id: String,
        /// Bond amount
        amount: u128,
        /// Owner ID
        owner_id: String,
    },
    
    /// Redeem a bond
    RedeemBond {
        /// Collection ID
        collection_id: String,
        /// Redeemer ID
        redeemer_id: String,
    },
    
    /// Get collection details
    GetCollection {
        /// Collection ID to retrieve
        collection_id: String,
    },
}

impl Default for LaunchpadFactoryWrapper {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageDispatch<LaunchpadFactoryWrapper> for LaunchpadFactoryMessage {
    fn from_opcode(opcode: u128, inputs: Vec<u128>) -> Result<Self> {
        match opcode {
            0 => {
                // Initialize
                if inputs.len() < 3 {
                    return Err(anyhow!("Initialize: Not enough inputs"));
                }
                
                let version = helpers::u128_vec_to_string(&inputs[0..1])?;
                let default_maturity_blocks = inputs[1];
                let default_interest_rate_bps = inputs[2];
                
                Ok(LaunchpadFactoryMessage::Initialize {
                    version,
                    default_maturity_blocks,
                    default_interest_rate_bps,
                })
            },
            1 => {
                // CreateCollection
                if inputs.len() < 5 {
                    return Err(anyhow!("CreateCollection: Not enough inputs"));
                }
                
                let name = helpers::u128_vec_to_string(&inputs[0..1])?;
                let symbol = helpers::u128_vec_to_string(&inputs[1..2])?;
                let description = helpers::u128_vec_to_string(&inputs[2..3])?;
                let interest_rate_bps = inputs[3];
                let maturity_blocks = inputs[4];
                
                Ok(LaunchpadFactoryMessage::CreateCollection {
                    name,
                    symbol,
                    description,
                    interest_rate_bps,
                    maturity_blocks,
                })
            },
            2 => {
                // MintBond
                if inputs.len() < 4 {
                    return Err(anyhow!("MintBond: Not enough inputs"));
                }
                
                let collection_id = helpers::u128_vec_to_string(&inputs[0..1])?;
                let orbital_token_id = helpers::u128_vec_to_string(&inputs[1..2])?;
                let amount = inputs[2];
                let owner_id = helpers::u128_vec_to_string(&inputs[3..4])?;
                
                Ok(LaunchpadFactoryMessage::MintBond {
                    collection_id,
                    orbital_token_id,
                    amount,
                    owner_id,
                })
            },
            3 => {
                // RedeemBond
                if inputs.len() < 2 {
                    return Err(anyhow!("RedeemBond: Not enough inputs"));
                }
                
                let collection_id = helpers::u128_vec_to_string(&inputs[0..1])?;
                let redeemer_id = helpers::u128_vec_to_string(&inputs[1..2])?;
                
                Ok(LaunchpadFactoryMessage::RedeemBond {
                    collection_id,
                    redeemer_id,
                })
            },
            100 => {
                // GetCollection
                if inputs.is_empty() {
                    return Err(anyhow!("GetCollection: Not enough inputs"));
                }
                
                let collection_id = helpers::u128_vec_to_string(&inputs[0..1])?;
                
                Ok(LaunchpadFactoryMessage::GetCollection {
                    collection_id,
                })
            },
            _ => Err(anyhow!("Unknown opcode: {}", opcode)),
        }
    }
    
    fn dispatch(&self, wrapper: &LaunchpadFactoryWrapper) -> Result<CallResponse> {
        let mut mutable_wrapper = LaunchpadFactoryWrapper::new(); // Create a new instance for mutation
        mutable_wrapper.factory = wrapper.factory.clone(); // Clone the factory data
        
        match self {
            LaunchpadFactoryMessage::Initialize { version, default_maturity_blocks, default_interest_rate_bps } => {
                mutable_wrapper.init_factory(version.clone(), *default_maturity_blocks, *default_interest_rate_bps)
            },
            LaunchpadFactoryMessage::CreateCollection { name, symbol, description, interest_rate_bps, maturity_blocks } => {
                mutable_wrapper.create_collection(name.clone(), symbol.clone(), description.clone(), *interest_rate_bps, *maturity_blocks)
            },
            LaunchpadFactoryMessage::MintBond { collection_id, orbital_token_id, amount, owner_id } => {
                mutable_wrapper.mint_bond(collection_id.clone(), orbital_token_id.clone(), *amount, owner_id.clone())
            },
            LaunchpadFactoryMessage::RedeemBond { collection_id, redeemer_id } => {
                mutable_wrapper.redeem_bond(collection_id.clone(), redeemer_id.clone())
            },
            LaunchpadFactoryMessage::GetCollection { collection_id } => {
                mutable_wrapper.get_collection(collection_id.clone())
            }
        }
    }
    
    fn export_abi() -> Vec<u8> {
        // This would generate an ABI description of the interface
        // For now, return a simple indication that it's implemented
        "LaunchpadFactoryMessage ABI".as_bytes().to_vec()
    }
}

impl LaunchpadFactoryWrapper {
    pub fn new() -> Self {
        // Create a placeholder factory that will be initialized later
        let context = StandaloneBlockContext::new();
        Self {
            factory: LaunchpadFactory::new(
                "0.0.0".to_string(), // Placeholder version
                100,                 // Default maturity blocks
                500,                 // Default interest rate (5%)
                &context,
            ),
        }
    }
    
    pub fn init_factory(&mut self, version: String, default_maturity_blocks: u128, default_interest_rate_bps: u128) -> Result<CallResponse> {
        // Convert MessageDispatch compatible types to contract types
        let maturity = helpers::u128_to_u64(default_maturity_blocks);
        let interest_rate = helpers::u128_to_u16(default_interest_rate_bps);
        
        // Initialize is a constructor, just update our factory instance
        let context = StandaloneBlockContext::new();
        self.factory = LaunchpadFactory::new(
            version,
            maturity,
            interest_rate,
            &context
        );
        
        // Return success response with log
        Ok(helpers::call_response_with_log("Launchpad factory initialized".to_string()))
    }
    
    pub fn create_collection(
        &mut self, 
        name: String, 
        symbol: String, 
        description: String, 
        interest_rate_bps: u128, 
        maturity_blocks: u128
    ) -> Result<CallResponse> {
        // Convert MessageDispatch compatible types to contract types
        let interest_rate = helpers::u128_to_u16(interest_rate_bps);
        let maturity = helpers::u128_to_u64(maturity_blocks);
        let desc_option = helpers::string_to_option_string(description);
        
        // Call the actual contract method with proper parameters and correct order
        let context = StandaloneBlockContext::new();
        let result = self.factory.create_collection(
            name,                   // First param is name
            symbol,                 // Second param is symbol
            desc_option,            // Third param is description
            Some(interest_rate.into()), // Fourth param is interest_rate_bps as Option<u16> converted to u64
            Some(maturity),         // Fifth param is maturity_blocks
            &context                // Last param is block_context
        );
        
        // Convert the result to a proper CallResponse
        Ok(CallResponse {
            alkanes: AlkaneTransferParcel::default(),
            data: result.as_bytes().to_vec(),
        })
    }
    
    pub fn mint_bond(&mut self, collection_id: String, orbital_token_id: String, amount: u128, owner_id: String) -> Result<CallResponse> {
        // Convert u128 to u64 for contract call
        let amount_u64 = helpers::u128_to_u64(amount);
        let context = StandaloneBlockContext::new();
        
        // Call the actual contract method with correct param order
        let result = self.factory.mint_bond(
            &collection_id,      // First param is collection_id as &str
            orbital_token_id,    // Second param is orbital_token_id as String
            amount_u64,          // Third param is amount as u64
            owner_id,            // Fourth param is owner_id as String
            &context             // Last param is block_context
        );
        
        // Convert the result to CallResponse
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
    
    pub fn redeem_bond(&mut self, collection_id: String, redeemer_id: String) -> Result<CallResponse> {
        let context = StandaloneBlockContext::new();
        let tx_context = MockTransactionContext::new_with_caller("test-caller");
        
        // Call the contract method with correct parameters and order
        let result = self.factory.redeem_bond_secure(
            &collection_id,    // collection_id as &str
            &tx_context,       // tx_context as TransactionContextExt
            &redeemer_id,      // redeemer_id as &str
            &context           // context as BlockContext
        );
        
        // Convert the result to CallResponse
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
    
    pub fn get_collection(&self, collection_id: String) -> Result<CallResponse> {
        // Get collection info
        if let Some(collection) = self.factory.get_collection(&collection_id) {
            let collection_json = serde_json::json!({
                "name": collection.name,
                "symbol": collection.symbol,
                "active": collection.is_active(),
                "total_bonds": collection.total_bonds(),
            });
            
            Ok(CallResponse {
                alkanes: AlkaneTransferParcel::default(),
                data: serde_json::to_vec(&collection_json).unwrap_or_default(),
            })
        } else {
            Ok(helpers::call_response_with_log(format!("Collection not found: {}", collection_id)))
        }
    }
}

// Implement AlkaneResponder to make the wrapper compatible with the runtime
impl AlkaneResponder for LaunchpadFactoryWrapper {
    // Add any required AlkaneResponder methods here if needed
}

// Constructor is exported from lib.rs instead
