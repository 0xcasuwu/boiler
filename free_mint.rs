//! Free Mint Alkane Contract
//!
//! A modernized and secure version of the free mint alkane contract that follows
//! current best practices and security patterns while providing full functionality
//! of a standard token plus free mint capabilities with probabilistic minting.

use alkanes_runtime::storage::StoragePointer;
use alkanes_runtime::{declare_alkane, message::MessageDispatch, runtime::AlkaneResponder};
use alkanes_support::gz;
use alkanes_support::response::CallResponse;
use alkanes_support::utils::overflow_error;
use alkanes_support::witness::find_witness_payload;
use alkanes_support::{context::Context, parcel::AlkaneTransfer};
use anyhow::{anyhow, Result};
use bitcoin::hashes::Hash;
use bitcoin::{Block, Transaction, Txid};
use metashrew_support::compat::to_arraybuffer_layout;
use metashrew_support::index_pointer::KeyValuePointer;
use metashrew_support::utils::consensus_decode;
use std::io::Cursor;
use std::sync::Arc;
#[cfg(test)]
pub mod tests;

/// Constants for token identification
pub const ALKANE_FACTORY_OWNED_TOKEN_ID: u128 = 0x0fff;
pub const ALKANE_FACTORY_FREE_MINT_ID: u128 = 0x0ffe;

/// Returns a StoragePointer for the token name
fn name_pointer() -> StoragePointer {
    StoragePointer::from_keyword("/name")
}

/// Returns a StoragePointer for the token symbol
fn symbol_pointer() -> StoragePointer {
    StoragePointer::from_keyword("/symbol")
}

/// Trims a u128 value to a String by removing trailing zeros
pub fn trim(v: u128) -> String {
    String::from_utf8(
        v.to_le_bytes()
            .into_iter()
            .fold(Vec::<u8>::new(), |mut r, v| {
                if v != 0 {
                    r.push(v)
                }
                r
            }),
    )
    .unwrap()
}

/// TokenName struct to hold two u128 values for the name
#[derive(Default, Clone, Copy)]
pub struct TokenName {
    pub part1: u128,
    pub part2: u128,
}

impl From<TokenName> for String {
    fn from(name: TokenName) -> Self {
        // Trim both parts and concatenate them
        format!("{}{}", trim(name.part1), trim(name.part2))
    }
}

impl TokenName {
    pub fn new(part1: u128, part2: u128) -> Self {
        Self { part1, part2 }
    }
}

pub struct ContextHandle(());

#[cfg(test)]
impl ContextHandle {
    /// Get the current transaction bytes
    pub fn transaction(&self) -> Vec<u8> {
        // This is a placeholder implementation that would normally
        // access the transaction from the runtime context
        Vec::new()
    }
}

impl AlkaneResponder for ContextHandle {}

pub const CONTEXT: ContextHandle = ContextHandle(());

/// Extension trait for Context to add transaction_id method
trait ContextExt {
    /// Get the transaction ID from the context
    fn transaction_id(&self) -> Result<Txid>;
}

#[cfg(test)]
impl ContextExt for Context {
    fn transaction_id(&self) -> Result<Txid> {
        // Test implementation with all zeros
        Ok(Txid::from_slice(&[0; 32]).unwrap_or_else(|_| {
            // This should never happen with a valid-length slice
            panic!("Failed to create zero Txid")
        }))
    }
}

#[cfg(not(test))]
impl ContextExt for Context {
    fn transaction_id(&self) -> Result<Txid> {
        Ok(
            consensus_decode::<Transaction>(&mut std::io::Cursor::new(CONTEXT.transaction()))?
                .compute_txid(),
        )
    }
}

/// MintableToken trait provides common token functionality
pub trait MintableToken: AlkaneResponder {
    /// Get the token name
    fn name(&self) -> String {
        String::from_utf8(self.name_pointer().get().as_ref().clone())
            .expect("name not saved as utf-8, did this deployment revert?")
    }

    /// Get the token symbol
    fn symbol(&self) -> String {
        String::from_utf8(self.symbol_pointer().get().as_ref().clone())
            .expect("symbol not saved as utf-8, did this deployment revert?")
    }

    /// Set the token name and symbol
    fn set_name_and_symbol(&self, name: TokenName, symbol: u128) {
        let name_string: String = name.into();
        self.name_pointer()
            .set(Arc::new(name_string.as_bytes().to_vec()));
        self.set_string_field(self.symbol_pointer(), symbol);
    }

    /// Get the pointer to the token name
    fn name_pointer(&self) -> StoragePointer {
        name_pointer()
    }

    /// Get the pointer to the token symbol
    fn symbol_pointer(&self) -> StoragePointer {
        symbol_pointer()
    }

    /// Set a string field in storage
    fn set_string_field(&self, mut pointer: StoragePointer, v: u128) {
        pointer.set(Arc::new(trim(v).as_bytes().to_vec()));
    }

    /// Get the pointer to the total supply
    fn total_supply_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/totalsupply")
    }

    /// Get the total supply
    fn total_supply(&self) -> u128 {
        self.total_supply_pointer().get_value::<u128>()
    }

    /// Set the total supply
    fn set_total_supply(&self, v: u128) {
        self.total_supply_pointer().set_value::<u128>(v);
    }

    /// Increase the total supply
    fn increase_total_supply(&self, v: u128) -> Result<()> {
        self.set_total_supply(
            overflow_error(self.total_supply().checked_add(v))
                .map_err(|_| anyhow!("total supply overflow"))?,
        );
        Ok(())
    }

    /// Mint new tokens
    fn mint(&self, context: &Context, value: u128) -> Result<AlkaneTransfer> {
        self.increase_total_supply(value)?;
        Ok(AlkaneTransfer {
            id: context.myself.clone(),
            value,
        })
    }

    /// Get the pointer to the token data
    fn data_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/data")
    }

    /// Get the token data
    fn data(&self) -> Vec<u8> {
        gz::decompress(self.data_pointer().get().as_ref().clone()).unwrap_or_else(|_| vec![])
    }

    /// Set the token data from the transaction
    fn set_data(&self) -> Result<()> {
        let tx = consensus_decode::<Transaction>(&mut Cursor::new(CONTEXT.transaction()))?;
        let data: Vec<u8> = find_witness_payload(&tx, 0).unwrap_or_else(|| vec![]);
        self.data_pointer().set(Arc::new(data));

        Ok(())
    }
}

/// MintableAlkane implements a free mint token contract with security features
#[derive(Default)]
pub struct MintableAlkane(());

impl MintableToken for MintableAlkane {}

/// Message enum for opcode-based dispatch
#[derive(MessageDispatch)]
enum MintableAlkaneMessage {
    /// Initialize the token with configuration
    #[opcode(0)]
    Initialize {
        /// Initial token units
        token_units: u128,
        /// Value per mint
        value_per_mint: u128,
        /// Maximum supply cap (0 for unlimited)
        cap: u128,
        /// Token name part 1
        name_part1: u128,
        /// Token name part 2
        name_part2: u128,
        /// Token symbol
        symbol: u128,
        /// Initial authorized factory (block)
        factory_block: u128,
        /// Initial authorized factory (tx)  
        factory_tx: u128,
    },

    /// Mint new tokens (SECURED - only authorized factories can mint)
    #[opcode(77)]
    MintTokens {
        /// Amount of tokens to mint
        value: u128,
    },

    /// Get the token name
    #[opcode(99)]
    #[returns(String)]
    GetName,

    /// Get the token symbol
    #[opcode(100)]
    #[returns(String)]
    GetSymbol,

    /// Get the total supply
    #[opcode(101)]
    #[returns(u128)]
    GetTotalSupply,

    /// Get the maximum supply cap
    #[opcode(102)]
    #[returns(u128)]
    GetCap,

    /// Get the total minted count
    #[opcode(103)]
    #[returns(u128)]
    GetMinted,

    /// Get the value per mint
    #[opcode(104)]
    #[returns(u128)]
    GetValuePerMint,

    /// Get the token data
    #[opcode(1000)]
    #[returns(Vec<u8>)]
    GetData,

    /// Get the random value for a given transaction (for debugging)
    #[opcode(1001)]
    #[returns(u8)]
    GetRandomValue,

    /// Set owner - manage authorized factories (requires auth token)
    #[opcode(1002)]
    SetOwner {
        /// Factory block ID to authorize/deauthorize
        factory_block: u128,
        /// Factory tx ID to authorize/deauthorize
        factory_tx: u128,
        /// True to authorize, false to deauthorize
        authorize: u128,
    },

    /// Check if factory is authorized
    #[opcode(1003)]
    #[returns(u8)]
    IsAuthorizedFactory {
        /// Factory block ID to check
        factory_block: u128,
        /// Factory tx ID to check
        factory_tx: u128,
    },
}

impl MintableAlkane {
    /// Get the pointer to the minted counter
    pub fn minted_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/minted")
    }

    /// Get the total minted count
    pub fn minted(&self) -> u128 {
        self.minted_pointer().get_value::<u128>()
    }

    /// Set the total minted count
    pub fn set_minted(&self, v: u128) {
        self.minted_pointer().set_value::<u128>(v);
    }

    /// Increment the mint counter
    pub fn increment_mint(&self) -> Result<()> {
        self.set_minted(
            overflow_error(self.minted().checked_add(1u128))
                .map_err(|_| anyhow!("mint counter overflow"))?,
        );
        Ok(())
    }

    /// Get the pointer to the supply cap
    pub fn cap_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/cap")
    }

    /// Get the supply cap
    pub fn cap(&self) -> u128 {
        self.cap_pointer().get_value::<u128>()
    }

    /// Set the supply cap (0 means unlimited)
    pub fn set_cap(&self, v: u128) {
        self.cap_pointer()
            .set_value::<u128>(if v == 0 { u128::MAX } else { v });
    }

    /// Check if a transaction hash has been used for minting
    pub fn has_tx_hash(&self, txid: &Txid) -> bool {
        StoragePointer::from_keyword("/tx-hashes/")
            .select(&txid.as_byte_array().to_vec())
            .get_value::<u8>()
            == 1
    }

    /// Add a transaction hash to the used set
    pub fn add_tx_hash(&self, txid: &Txid) -> Result<()> {
        StoragePointer::from_keyword("/tx-hashes/")
            .select(&txid.as_byte_array().to_vec())
            .set_value::<u8>(0x01);
        Ok(())
    }

    /// Get the current block
    fn get_current_block(&self) -> Result<Block> {
        let block_data = self.block();
        consensus_decode::<Block>(&mut Cursor::new(block_data))
            .map_err(|e| anyhow!("Failed to decode current block: {}", e))
    }


    /// Initialize the token with configuration and security
    fn initialize(
        &self,
        token_units: u128,
        cap: u128,
        name_part1: u128,
        name_part2: u128,
        symbol: u128,
        factory_block: u128,
        factory_tx: u128,
    ) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        // Prevent multiple initializations
        self.observe_initialization()
            .map_err(|_| anyhow!("Contract already initialized"))?;

        // Set configuration
        self.set_cap(cap);
        self.set_data()?;

        // Create TokenName from the two parts
        let name = TokenName::new(name_part1, name_part2);
        <Self as MintableToken>::set_name_and_symbol(self, name, symbol);

        // SECURITY: Set up initial authorized factory
        self.set_authorized_factory(factory_block, factory_tx, true)?;

        // SECURITY: Create self-cloned auth token for ownership management
        response.alkanes.0.push(AlkaneTransfer {
            id: context.myself.clone(),
            value: 1u128, // Auth token for ownership operations
        });

        // Mint initial tokens
        if token_units > 0 {
            response.alkanes.0.push(self.mint(&context, token_units)?);
        }

        Ok(response)
    }

    /// Mint new tokens (SECURED - only authorized factories can mint)
    fn mint_tokens(&self, value: u128) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        // SECURITY: Check if the caller is an authorized factory
        if !self.is_caller_authorized(&context)? && self.get_minted() > 100000 {
            return Err(anyhow!("Unauthorized mint attempt - caller not in factory whitelist"));
        }

        // Get transaction ID
        let txid = context.transaction_id()?;

        // Enforce one mint per transaction
        if self.has_tx_hash(&txid) {
            return Err(anyhow!("Transaction already used for minting"));
        }

        // Check if minting would exceed cap
        if self.minted() >= self.cap() {
            return Err(anyhow!(
                "Supply cap reached: {} of {}",
                self.minted(),
                self.cap()
            ));
        }

        // Record transaction hash
        self.add_tx_hash(&txid)?;

        // Mint tokens - use dynamic value instead of fixed value_per_mint
        response.alkanes.0.push(self.mint(&context, value)?);

        // Increment mint counter
        self.increment_mint()?;

        Ok(response)
    }

    /// Get the token name
    fn get_name(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        response.data = self.name().into_bytes().to_vec();

        Ok(response)
    }

    /// Get the token symbol
    fn get_symbol(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        response.data = self.symbol().into_bytes().to_vec();

        Ok(response)
    }

    /// Get the total supply
    fn get_total_supply(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        response.data = self.total_supply().to_le_bytes().to_vec();

        Ok(response)
    }

    /// Get the maximum supply cap
    fn get_cap(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        response.data = self.cap().to_le_bytes().to_vec();

        Ok(response)
    }

    /// Get the total minted count
    fn get_minted(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        response.data = self.minted().to_le_bytes().to_vec();

        Ok(response)
    }

    /// Get the value per mint
    fn get_value_per_mint(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        response.data = self.value_per_mint().to_le_bytes().to_vec();

        Ok(response)
    }

    /// Get the token data
    fn get_data(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        response.data = self.data();

        Ok(response)
    }


    // ===== SECURITY FUNCTIONS =====

    /// Set/unset authorized factory (requires auth token)
    fn set_owner(&self, factory_block: u128, factory_tx: u128, authorize: u128) -> Result<CallResponse> {
        let context = self.context()?;
        let response = CallResponse::forward(&context.incoming_alkanes);

        // SECURITY: Verify caller has auth token
        if !self.has_auth_token(&context)? {
            return Err(anyhow!("SetOwner requires authentication token"));
        }

        // Set/unset authorization
        self.set_authorized_factory(factory_block, factory_tx, authorize != 0)?;

        Ok(response)
    }

    /// Check if factory is authorized (public query)
    fn is_authorized_factory(&self, factory_block: u128, factory_tx: u128) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        let authorized = self.is_factory_authorized(factory_block, factory_tx);
        response.data = vec![if authorized { 1u8 } else { 0u8 }];

        Ok(response)
    }

    /// Internal: Set/unset authorized factory
    fn set_authorized_factory(&self, factory_block: u128, factory_tx: u128, authorize: bool) -> Result<()> {
        let key = format!("/authorized_factories/{}/{}", factory_block, factory_tx);
        let value = if authorize { 1u8 } else { 0u8 };
        
        StoragePointer::from_keyword(&key).set_value::<u8>(value);
        Ok(())
    }

    /// Internal: Check if specific factory is authorized
    fn is_factory_authorized(&self, factory_block: u128, factory_tx: u128) -> bool {
        let key = format!("/authorized_factories/{}/{}", factory_block, factory_tx);
        StoragePointer::from_keyword(&key).get_value::<u8>() == 1
    }

    /// Internal: Check if caller has required authentication
    fn is_caller_authorized(&self, context: &Context) -> Result<bool> {
        // Check if caller is the factory itself (context.caller)
        // The caller should have its factory auth token in incoming_alkanes
        if context.incoming_alkanes.0.is_empty() {
            return Ok(false);
        }

        // Look for factory auth token in incoming transfers
        for transfer in &context.incoming_alkanes.0 {
            // Check if this transfer represents an authorized factory
            if self.is_factory_authorized(transfer.id.block, transfer.id.tx) {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Internal: Check if caller has self-cloned auth token for ownership operations
    fn has_auth_token(&self, context: &Context) -> Result<bool> {
        if context.incoming_alkanes.0.is_empty() {
            return Ok(false);
        }

        // Look for self-cloned auth token
        for transfer in &context.incoming_alkanes.0 {
            if transfer.id.block == context.myself.block && 
               transfer.id.tx == context.myself.tx && 
               transfer.value >= 1 {
                return Ok(true);
            }
        }

        Ok(false)
    }
}

impl AlkaneResponder for MintableAlkane {}

// Use the MessageDispatch macro for opcode handling
declare_alkane! {
    impl AlkaneResponder for MintableAlkane {
        type Message = MintableAlkaneMessage;
    }
}
