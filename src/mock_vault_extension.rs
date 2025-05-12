use crate::mock_vault::MockYieldVault;

/// Extension methods for MockYieldVault to support testing
impl MockYieldVault {
    /// Get total assets (API method matching real implementation)
    pub fn total_assets(&self) -> u128 {
        self.get_total_assets()
    }

    /// Get total supply (API method matching real implementation)
    pub fn total_supply(&self) -> u128 {
        self.get_total_issuance()
    }

    /// Set total assets directly (for testing)
    pub fn set_total_assets(&self, amount: u128) {
        self.set_value("total_assets", amount);
    }

    /// Update yield for a specified number of blocks (for testing)
    pub fn update_yield_for_blocks(&self, blocks: u64) -> Result<(), &'static str> {
        let current_height = self.get_block_height();
        self.set_block_height(current_height + blocks);
        self.update_yield()
    }

    /// Convert assets to shares (ERC-4626 API method)
    pub fn convert_to_shares(&self, assets: u128) -> u128 {
        // Update yield first for accurate calculation
        if let Err(_) = self.update_yield() {
            return 0;
        }

        let total_assets = self.get_total_assets();
        let total_issuance = self.get_total_issuance();
        
        match self.convert_assets_to_tokens(assets, total_assets, total_issuance) {
            Ok(shares) => shares,
            Err(_) => 0
        }
    }

    /// Convert shares to assets (ERC-4626 API method)
    pub fn convert_to_assets(&self, shares: u128) -> u128 {
        // Update yield first for accurate calculation
        if let Err(_) = self.update_yield() {
            return 0;
        }

        let total_assets = self.get_total_assets();
        let total_issuance = self.get_total_issuance();
        
        match self.convert_tokens_to_assets(shares, total_assets, total_issuance) {
            Ok(assets) => assets,
            Err(_) => 0
        }
    }

    /// Maximum deposit allowed (ERC-4626 API method)
    pub fn max_deposit(&self, _account: &str) -> u128 {
        // Simplified implementation - just return a very large number
        // as if there's no cap
        u128::MAX / 2
    }

    /// Maximum mint allowed (ERC-4626 API method)
    pub fn max_mint(&self, _account: &str) -> u128 {
        // Simplified implementation - just return a very large number
        // as if there's no cap
        u128::MAX / 2
    }

    /// Maximum withdraw allowed (ERC-4626 API method)
    pub fn max_withdraw(&self, account: &str) -> u128 {
        // Update yield first for accurate calculation
        if let Err(_) = self.update_yield() {
            return 0;
        }
        
        // Check user's balance
        let user_shares = self.get_token_balance(account);
        if user_shares == 0 {
            return 0;
        }
        
        // Convert user's shares to assets
        let total_assets = self.get_total_assets();
        let total_issuance = self.get_total_issuance();
        
        match self.convert_tokens_to_assets(user_shares, total_assets, total_issuance) {
            Ok(assets) => assets,
            Err(_) => 0
        }
    }

    /// Maximum redeem allowed (ERC-4626 API method)
    pub fn max_redeem(&self, account: &str) -> u128 {
        // Simply return the user's balance
        self.get_token_balance(account)
    }

    /// Mint shares by depositing assets (ERC-4626 API method)
    pub fn mint(&self, _caller: &str, receiver: &str, shares: u128) -> Result<u128, &'static str> {
        // Validate non-zero shares
        if shares == 0 {
            return Err("Cannot mint zero shares");
        }

        // Update yield accrual first
        self.update_yield()?;

        // Get current state
        let total_assets = self.get_total_assets();
        let total_issuance = self.get_total_issuance();

        // Calculate assets needed
        let assets = if total_issuance == 0 {
            // If no supply, use 1:1 ratio
            shares
        } else {
            self.convert_tokens_to_assets(shares, total_assets, total_issuance)?
        };

        if assets == 0 {
            return Err("Zero assets required");
        }

        // Update state
        self.set_value("total_assets", total_assets + assets);
        self.set_total_issuance(total_issuance + shares);

        // Issue tokens to receiver (simulating token transfer)
        self.issue_tokens(receiver, shares);

        Ok(assets)
    }

    /// Withdraw assets by burning shares (ERC-4626 API method)
    pub fn withdraw(&self, _caller: &str, _receiver: &str, owner: &str, assets: u128) -> Result<u128, &'static str> {
        // Validate non-zero assets
        if assets == 0 {
            return Ok(0); // ERC-4626 specifies this as a no-op that succeeds
        }

        // Update yield accrual first
        self.update_yield()?;

        // Get current state
        let total_assets = self.get_total_assets();
        let total_issuance = self.get_total_issuance();

        if assets > total_assets {
            return Err("Insufficient assets in vault");
        }

        // Calculate shares needed
        let shares = if total_assets == 0 {
            return Err("No assets in vault");
        } else {
            self.convert_assets_to_tokens(assets, total_assets, total_issuance)?
        };

        // Check token ownership
        let owner_tokens = self.get_token_balance(owner);
        if owner_tokens < shares {
            return Err("Insufficient shares");
        }

        // Update state
        self.set_value("total_assets", total_assets - assets);
        self.set_total_issuance(total_issuance - shares);

        // Update token balances (burn owner's tokens)
        self.set_value(&format!("token_{}", owner), owner_tokens - shares);

        Ok(shares)
    }

    /// Preview mint (ERC-4626 API method)
    pub fn preview_mint(&self, shares: u128) -> u128 {
        // Update yield first for accurate calculation
        if let Err(_) = self.update_yield() {
            return 0;
        }

        let total_assets = self.get_total_assets();
        let total_issuance = self.get_total_issuance();
        
        if total_issuance == 0 {
            // If no supply, use 1:1 ratio
            return shares;
        }
        
        match self.convert_tokens_to_assets(shares, total_assets, total_issuance) {
            Ok(assets) => assets,
            Err(_) => 0
        }
    }

    /// Preview withdraw (ERC-4626 API method)
    pub fn preview_withdraw(&self, assets: u128) -> u128 {
        // Update yield first for accurate calculation
        if let Err(_) = self.update_yield() {
            return 0;
        }

        let total_assets = self.get_total_assets();
        let total_issuance = self.get_total_issuance();
        
        if total_assets == 0 {
            return 0;
        }
        
        // Round up when calculating shares needed for withdrawal
        // to ensure user gets exactly the requested assets
        let shares = match total_assets.checked_mul(assets) {
            Some(product) => {
                let div = product / total_issuance;
                let remainder = product % total_issuance;
                
                if remainder > 0 {
                    div + 1 // Round up
                } else {
                    div
                }
            },
            None => 0
        };
        
        shares
    }

    /// Update yield rate (used for testing)
    pub fn set_yield_rate_and_update(&self, rate_bps: u128) -> Result<(), &'static str> {
        self.set_yield_rate(rate_bps);
        self.update_yield()
    }
}
