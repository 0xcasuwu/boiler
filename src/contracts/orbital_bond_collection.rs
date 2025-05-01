use std::collections::HashMap;
use crate::models::bond::Bond;
use crate::utils::BlockContext;
use crate::utils::TransactionContextExt;
use anyhow::{Result, anyhow};

#[derive(Clone)]
pub struct OrbitalBondCollection {
    pub id: String,
    pub name: String,
    pub symbol: String,
    pub description: Option<String>,
    pub bonds: HashMap<String, Bond>,
    pub interest_rate_bps: u16, // basis points
    pub maturity_blocks: u64,
    pub active: bool,
}

impl OrbitalBondCollection {
    pub fn new(id: String, name: String, symbol: String, interest_rate_bps: u16, maturity_blocks: u64, _context: &impl BlockContext) -> Self {
        Self {
            id,
            name,
            symbol,
            description: None,
            bonds: HashMap::new(),
            interest_rate_bps,
            maturity_blocks,
            active: true,
        }
    }
    
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
    
    pub fn is_active(&self) -> bool {
        self.active
    }
    
    pub fn total_bonds(&self) -> usize {
        self.bonds.len()
    }
    
    pub fn get_bond(&self, bond_id: &str) -> Option<&Bond> {
        self.bonds.get(bond_id)
    }
    
    pub fn mint_bond(&mut self, orbital_token_id: String, amount: u64, _owner_id: String, context: &impl BlockContext) -> Result<String> {
        let bond_id = format!("bond-{}-{}", orbital_token_id, context.get_current_block_height());
        
        // Create bond with maturity block based on current height plus maturity period
        let _maturity_block = context.get_current_block_height() + self.maturity_blocks;
        
        let bond = Bond::new(
            bond_id.clone(),
            orbital_token_id,
            amount,
            context.get_current_block_height(),
            self.maturity_blocks,
            self.interest_rate_bps,
        );
        
        self.bonds.insert(bond_id.clone(), bond);
        Ok(bond_id)
    }
    
    pub fn redeem_bond_secure(&mut self, tx_context: &impl TransactionContextExt, redeemer_id: &str, block_context: &impl BlockContext) -> Result<u64> {
        // Security check: Verify the redeemer signature
        if !tx_context.verify_signature(redeemer_id) {
            return Err(anyhow!("Signature verification failed"));
        }
        
        // Find a bond owned by the redeemer (we'd need to track ownership separately or add owner to the bond)
        // For now, just find the first active bond
        let bond_id_opt = self.bonds.iter()
            .find(|(_, bond)| bond.status == crate::models::bond::BondStatus::Active)
            .map(|(id, _)| id.clone());
        
        let bond_id = match bond_id_opt {
            Some(id) => id,
            None => return Err(anyhow!("No unredeemed bond found for this redeemer")),
        };
        
        // Get the bond
        let bond = self.bonds.get_mut(&bond_id)
            .ok_or_else(|| anyhow!("Bond not found"))?;
        
        // Check maturity and redeem the bond
        match bond.redeem(block_context) {
            Ok(total_amount) => Ok(total_amount),
            Err(msg) => Err(anyhow!(msg))
        }
    }
    
    // This method is now handled by Bond::new since interest calculation
    // is now part of the Bond's responsibility
}
