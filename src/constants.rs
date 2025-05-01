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
pub const LAST_YIELD_UPDATE_KEY: &str = "/last-yield-update";
pub const BALANCES_PREFIX: &str = "/balances/";
pub const DATA_PREFIX: &str = "/data/";
