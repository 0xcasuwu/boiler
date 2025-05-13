use crate::YieldVault;
use crate::utils::Conversion;
use alkanes_proc_macros::MessageDispatch;
use alkanes_support::response::CallResponse;

/// YieldVaultMessage defines all the messages that can be handled by the YieldVault contract
/// Each variant corresponds to an opcode and maps to a method in the YieldVault implementation
#[derive(MessageDispatch)]
pub enum YieldVaultMessage {
    /// Initialize the vault with its base parameters
    #[opcode(0)]
    Initialize {
        name: String,
        symbol: String,
        asset_name: String,
        asset_symbol: String,
        decimal_offset: u128,
    },

    /// Deposit assets into the vault
    #[opcode(10)]
    Deposit {
        tx_hash: String,
        caller: String,
        receiver: String,
        assets: u128,
    },

    /// Mint shares from the vault
    #[opcode(11)]
    Mint {
        tx_hash: String,
        caller: String,
        receiver: String,
        shares: u128,
    },

    /// Withdraw assets from the vault
    #[opcode(12)]
    Withdraw {
        tx_hash: String,
        caller: String,
        receiver: String,
        owner: String,
        assets: u128,
    },

    /// Redeem shares from the vault
    #[opcode(13)]
    Redeem {
        tx_hash: String,
        caller: String,
        receiver: String,
        owner: String,
        shares: u128,
    },

    /// Get the name of the vault token
    #[opcode(100)]
    #[returns(String)]
    GetName,

    /// Get the symbol of the vault token
    #[opcode(101)]
    #[returns(String)]
    GetSymbol,

    /// Get the decimals of the vault token
    #[opcode(102)]
    #[returns(u8)]
    GetDecimals,

    /// Get the asset symbol of the underlying asset
    #[opcode(103)]
    #[returns(String)]
    GetAsset,

    /// Get the total assets in the vault
    #[opcode(200)]
    #[returns(u128)]
    GetTotalAssets,

    /// Convert assets to shares
    #[opcode(201)]
    #[returns(u128)]
    ConvertToShares {
        assets: u128,
    },

    /// Convert shares to assets
    #[opcode(202)]
    #[returns(u128)]
    ConvertToAssets {
        shares: u128,
    },

    /// Get the maximum deposit amount for a receiver
    #[opcode(300)]
    #[returns(u128)]
    GetMaxDeposit {
        receiver: String,
    },

    /// Get the maximum mint amount for a receiver
    #[opcode(301)]
    #[returns(u128)]
    GetMaxMint {
        receiver: String,
    },

    /// Get the maximum withdraw amount for an owner
    #[opcode(302)]
    #[returns(u128)]
    GetMaxWithdraw {
        owner: String,
    },

    /// Get the maximum redeem amount for an owner
    #[opcode(303)]
    #[returns(u128)]
    GetMaxRedeem {
        owner: String,
    },

    /// Preview deposit - calculate shares for assets
    #[opcode(400)]
    #[returns(CallResponse)]
    PreviewDepositWrapper {
        assets: u128,
    },

    /// Preview mint - calculate assets for shares
    #[opcode(401)]
    #[returns(CallResponse)]
    PreviewMintWrapper {
        shares: u128,
    },

    /// Preview withdraw - calculate shares for assets
    #[opcode(402)]
    #[returns(CallResponse)]
    PreviewWithdrawWrapper {
        assets: u128,
    },

    /// Preview redeem - calculate assets for shares
    #[opcode(403)]
    #[returns(CallResponse)]
    PreviewRedeemWrapper {
        shares: u128,
    },

    /// Get custom data by key
    #[opcode(501)]
    #[returns(String)]
    GetData {
        key: String,
    },

    /// Get total supply of shares
    #[opcode(601)]
    #[returns(u128)]
    GetTotalSupply,
}

// The MessageDispatch derive macro automatically implements the MessageDispatch trait
// which maps each enum variant to the corresponding method in YieldVault
// 
// The method names must match exactly with the method names in the YieldVault implementation:
// - For Initialize, the method is called initialize
// - For Deposit, the method is called deposit
// - For Mint, the method is called mint
// - For Withdraw, the method is called withdraw
// - For Redeem, the method is called redeem
// - For GetName, the method is called get_name
// - For GetSymbol, the method is called get_symbol
// - For GetDecimals, the method is called get_decimals
// - For GetAsset, the method is called get_asset
// - For GetTotalAssets, the method is called get_total_assets
// - For ConvertToShares, the method is called convert_to_shares
// - For ConvertToAssets, the method is called convert_to_assets
// - For GetMaxDeposit, the method is called get_max_deposit
// - For GetMaxMint, the method is called get_max_mint
// - For GetMaxWithdraw, the method is called get_max_withdraw
// - For GetMaxRedeem, the method is called get_max_redeem
// - For PreviewDepositWrapper, the method is called preview_deposit_wrapper
// - For PreviewMintWrapper, the method is called preview_mint_wrapper
// - For PreviewWithdrawWrapper, the method is called preview_withdraw_wrapper
// - For PreviewRedeemWrapper, the method is called preview_redeem_wrapper
// - For GetData, the method is called get_data
// - For GetTotalSupply, the method is called get_total_supply
