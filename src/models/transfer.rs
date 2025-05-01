use serde::{Deserialize, Serialize};

/// A structure representing an Alkane token transfer
/// 
/// This is used by multiple modules to represent token transfers
/// between accounts. This common structure ensures consistency
/// across all token operations in the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlkaneTransfer {
    /// Token identifier as a byte array
    pub id: Vec<u8>,
    
    /// Value/amount of the token
    pub value: u128,
    
    /// Optional source account
    pub from: Option<String>,
    
    /// Optional destination account
    pub to: Option<String>,
}
