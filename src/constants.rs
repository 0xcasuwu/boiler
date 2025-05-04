// YieldVault Constants
// Constants for ERC-4626 Bitcoin implementation

/// Token ID constants
pub const ALKANE_FACTORY_YIELD_VAULT_ID: u128 = 0x0ffd;

/// Opcode ranges
pub const INITIALIZE_OPCODE: u32 = 0;
pub const ASSET_MANAGEMENT_OPCODE_RANGE_START: u32 = 10;
pub const ASSET_MANAGEMENT_OPCODE_RANGE_END: u32 = 19;
pub const METADATA_VIEW_OPCODE_RANGE_START: u32 = 100;
pub const METADATA_VIEW_OPCODE_RANGE_END: u32 = 199;
pub const ACCOUNTING_VIEW_OPCODE_RANGE_START: u32 = 200;
pub const ACCOUNTING_VIEW_OPCODE_RANGE_END: u32 = 299;
pub const LIMIT_VIEW_OPCODE_RANGE_START: u32 = 300;
pub const LIMIT_VIEW_OPCODE_RANGE_END: u32 = 399;
pub const PREVIEW_VIEW_OPCODE_RANGE_START: u32 = 400;
pub const PREVIEW_VIEW_OPCODE_RANGE_END: u32 = 499;
pub const CUSTOM_DATA_OPCODE_RANGE_START: u32 = 500;
pub const CUSTOM_DATA_OPCODE_RANGE_END: u32 = 599;
pub const BALANCE_MANAGEMENT_OPCODE_RANGE_START: u32 = 600;
pub const BALANCE_MANAGEMENT_OPCODE_RANGE_END: u32 = 699;
pub const ADMIN_OPCODE_RANGE_START: u32 = 900;
pub const ADMIN_OPCODE_RANGE_END: u32 = 999;

/// Opcode definitions for all operations
pub mod opcodes {
    // Initialization
    pub const INITIALIZE: u32 = 0;
    
    // Asset Management
    pub const DEPOSIT: u32 = 10;
    pub const MINT: u32 = 11;
    pub const WITHDRAW: u32 = 12;
    pub const REDEEM: u32 = 13;
    
    // Metadata View Functions
    pub const GET_NAME: u32 = 100;
    pub const GET_SYMBOL: u32 = 101;
    pub const GET_DECIMALS: u32 = 102;
    pub const GET_ASSET: u32 = 103;
    
    // Accounting View Functions
    pub const GET_TOTAL_ASSETS: u32 = 200;
    pub const CONVERT_TO_SHARES: u32 = 201;
    pub const CONVERT_TO_ASSETS: u32 = 202;
    
    // Limit View Functions
    pub const GET_MAX_DEPOSIT: u32 = 300;
    pub const GET_MAX_MINT: u32 = 301;
    pub const GET_MAX_WITHDRAW: u32 = 302;
    pub const GET_MAX_REDEEM: u32 = 303;
    
    // Preview View Functions
    pub const PREVIEW_DEPOSIT: u32 = 400;
    pub const PREVIEW_MINT: u32 = 401;
    pub const PREVIEW_WITHDRAW: u32 = 402;
    pub const PREVIEW_REDEEM: u32 = 403;
    
    // Custom Data Operations
    pub const SET_DATA: u32 = 500;
    pub const GET_DATA: u32 = 501;
    
    // Balance Management
    pub const GET_BALANCE_OF: u32 = 600;
    pub const GET_TOTAL_SUPPLY: u32 = 601;
    
    // Administrative Operations
    pub const UPDATE_YIELD_RATE: u32 = 900;
}

/// Storage key patterns
pub const NAME_KEY: &str = "/name";
pub const SYMBOL_KEY: &str = "/symbol";
pub const ASSET_NAME_KEY: &str = "/asset-name";
pub const ASSET_SYMBOL_KEY: &str = "/asset-symbol";
pub const DECIMALS_KEY: &str = "/decimals";
pub const TOTAL_SUPPLY_KEY: &str = "/total-supply";
pub const TOTAL_ASSETS_KEY: &str = "/total-assets";
pub const TX_HASHES_KEY: &str = "/tx-hashes";
pub const INITIALIZED_KEY: &str = "/initialized";
pub const YIELD_RATE_KEY: &str = "/yield-rate";
pub const LAST_YIELD_HEIGHT_KEY: &str = "/last-yield-height";
pub const BALANCES_PREFIX: &str = "/balances/";
pub const DATA_PREFIX: &str = "/data/";
