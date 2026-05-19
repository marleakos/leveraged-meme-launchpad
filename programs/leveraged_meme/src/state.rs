use anchor_lang::prelude::*;

#[account]
pub struct TokenState {
    /// Token creator
    pub creator: Pubkey,
    
    /// Token mint address
    pub token_mint: Pubkey,
    
    /// Token name (max 32 chars)
    pub name: String,
    
    /// Token symbol (max 10 chars)
    pub symbol: String,
    
    /// Metadata URI
    pub uri: String,
    
    /// Leverage multiplier (2x, 3x, 5x)
    pub leverage: u8,
    
    /// Direction: Long or Short
    pub direction: Direction,
    
    /// Perp market index (SOL=0, BTC=1, ETH=2)
    pub perp_market_index: u16,
    
    /// Curve state
    pub curve_state: CurveState,
    
    /// Perp position info
    pub perp_position: PerpPosition,
    
    /// Fee vault
    pub fee_vault: Pubkey,
    
    /// Graduation status
    pub graduated: bool,
    
    /// Graduated AMM pool (if graduated)
    pub amm_pool: Option<Pubkey>,
    
    /// Creation timestamp
    pub created_at: i64,
    
    /// Last updated timestamp
    pub updated_at: i64,
    
    /// Pause status
    pub paused: bool,
    
    /// Total fees collected
    pub total_fees_collected: u64,
}

impl TokenState {
    pub const SIZE: usize = 
        32 + // creator
        32 + // token_mint
        4 + 32 + // name (String)
        4 + 10 + // symbol (String)
        4 + 200 + // uri (String)
        1 + // leverage
        1 + // direction
        2 + // perp_market_index
        CurveState::SIZE + // curve_state
        PerpPosition::SIZE + // perp_position
        32 + // fee_vault
        1 + // graduated
        1 + 32 + // amm_pool (Option<Pubkey>)
        8 + // created_at
        8 + // updated_at
        1 + // paused
        8; // total_fees_collected
    
    /// Calculate current token price in SOL
    pub fn calculate_price(&self) -> Result<u64> {
        let base_price = self.curve_state.calculate_base_price()?;
        let perp_multiplier = self.perp_position.get_price_multiplier()?;
        
        // Price = base_price * perp_multiplier / 1_000_000
        let price = (base_price as u128)
            .checked_mul(perp_multiplier as u128)
            .ok_or(LeveragedMemeError::MathOverflow)?
            .checked_div(1_000_000)
            .ok_or(LeveragedMemeError::MathOverflow)? as u64;
        
        Ok(price)
    }
    
    /// Calculate market cap
    pub fn market_cap(&self) -> Result<u64> {
        let price = self.calculate_price()?;
        let supply = self.curve_state.real_token_reserve;
        
        let market_cap = (price as u128)
            .checked_mul(supply as u128)
            .ok_or(LeveragedMemeError::MathOverflow)?
            .checked_div(1_000_000) // Adjust for decimals
            .ok_or(LeveragedMemeError::MathOverflow)? as u64;
        
        Ok(market_cap)
    }
    
    /// Check if ready for graduation
    pub fn can_graduate(&self) -> Result<bool> {
        if self.graduated {
            return Ok(false);
        }
        
        let market_cap = self.market_cap()?;
        let threshold = crate::constants::GRADUATION_THRESHOLD_USD;
        
        Ok(market_cap >= threshold)
    }
}

// FIX: CurveState is an *embedded* struct inside TokenState, not a separate Anchor account.
// It must NOT use #[account] — that macro would derive AccountSerialize/AccountDeserialize/Owner
// which are only valid for top-level PDA accounts, causing all 40 E0277 trait errors.
// The correct derives for an embedded struct are AnchorSerialize + AnchorDeserialize + Clone + Copy.
// The discriminator field has also been removed — it is injected automatically by Anchor only for
// top-level accounts, and adding it manually to an embedded struct wastes 8 bytes and breaks sizing.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, Default)]
pub struct CurveState {
    /// Virtual SOL reserve (for pricing)
    pub virtual_sol_reserve: u64,
    
    /// Virtual token reserve (for pricing)
    pub virtual_token_reserve: u64,
    
    /// Real SOL in the curve
    pub real_sol_reserve: u64,
    
    /// Real tokens in the curve
    pub real_token_reserve: u64,
    
    /// Constant product k = x * y
    pub k: u128,
}

impl CurveState {
    // FIX: SIZE no longer includes 8 bytes for a discriminator — embedded structs have none.
    // Previous value was 8 + 8 + 8 + 8 + 8 + 16 = 56; correct value is 8 + 8 + 8 + 8 + 16 = 48.
    pub const SIZE: usize = 8 + 8 + 8 + 8 + 16; // virtual_sol + virtual_token + real_sol + real_token + k
    
    pub fn calculate_base_price(&self) -> Result<u64> {
        if self.virtual_token_reserve == 0 {
            return Err(LeveragedMemeError::MathOverflow.into());
        }
        
        let price = (self.virtual_sol_reserve as u128)
            .checked_mul(1_000_000) // 6 decimals
            .ok_or(LeveragedMemeError::MathOverflow)?
            .checked_div(self.virtual_token_reserve as u128)
            .ok_or(LeveragedMemeError::MathOverflow)? as u64;
        
        Ok(price)
    }
    
    /// Calculate tokens to receive for SOL input
    pub fn calculate_buy_output(&self, sol_input: u64) -> Result<u64> {
        let new_sol = self.virtual_sol_reserve
            .checked_add(sol_input)
            .ok_or(LeveragedMemeError::MathOverflow)?;
        
        let new_tokens = (self.k)
            .checked_div(new_sol as u128)
            .ok_or(LeveragedMemeError::MathOverflow)? as u64;
        
        let tokens_out = self.virtual_token_reserve
            .checked_sub(new_tokens)
            .ok_or(LeveragedMemeError::MathOverflow)?;
        
        Ok(tokens_out)
    }
    
    /// Calculate SOL to receive for token input
    pub fn calculate_sell_output(&self, token_input: u64) -> Result<u64> {
        let new_tokens = self.virtual_token_reserve
            .checked_add(token_input)
            .ok_or(LeveragedMemeError::MathOverflow)?;
        
        let new_sol = (self.k)
            .checked_div(new_tokens as u128)
            .ok_or(LeveragedMemeError::MathOverflow)? as u64;
        
        let sol_out = self.virtual_sol_reserve
            .checked_sub(new_sol)
            .ok_or(LeveragedMemeError::MathOverflow)?;
        
        Ok(sol_out)
    }
    
    /// Update reserves after buy
    pub fn update_after_buy(&mut self, sol_in: u64, tokens_out: u64) -> Result<()> {
        self.real_sol_reserve = self.real_sol_reserve
            .checked_add(sol_in)
            .ok_or(LeveragedMemeError::MathOverflow)?;
        
        self.real_token_reserve = self.real_token_reserve
            .checked_sub(tokens_out)
            .ok_or(LeveragedMemeError::MathOverflow)?;
        
        self.virtual_sol_reserve = self.virtual_sol_reserve
            .checked_add(sol_in)
            .ok_or(LeveragedMemeError::MathOverflow)?;
        
        self.virtual_token_reserve = self.virtual_token_reserve
            .checked_sub(tokens_out)
            .ok_or(LeveragedMemeError::MathOverflow)?;
        
        // Verify invariant
        let new_k = (self.virtual_sol_reserve as u128)
            .checked_mul(self.virtual_token_reserve as u128)
            .ok_or(LeveragedMemeError::MathOverflow)?;
        
        if new_k != self.k {
            return Err(LeveragedMemeError::InvariantViolation.into());
        }
        
        Ok(())
    }
    
    /// Update reserves after sell
    pub fn update_after_sell(&mut self, tokens_in: u64, sol_out: u64) -> Result<()> {
        self.real_sol_reserve = self.real_sol_reserve
            .checked_sub(sol_out)
            .ok_or(LeveragedMemeError::MathOverflow)?;
        
        self.real_token_reserve = self.real_token_reserve
            .checked_add(tokens_in)
            .ok_or(LeveragedMemeError::MathOverflow)?;
        
        self.virtual_sol_reserve = self.virtual_sol_reserve
            .checked_sub(sol_out)
            .ok_or(LeveragedMemeError::MathOverflow)?;
        
        self.virtual_token_reserve = self.virtual_token_reserve
            .checked_add(tokens_in)
            .ok_or(LeveragedMemeError::MathOverflow)?;
        
        // Verify invariant
        let new_k = (self.virtual_sol_reserve as u128)
            .checked_mul(self.virtual_token_reserve as u128)
            .ok_or(LeveragedMemeError::MathOverflow)?;
        
        if new_k != self.k {
            return Err(LeveragedMemeError::InvariantViolation.into());
        }
        
        Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub struct PerpPosition {
    /// Position size (base asset)
    pub base_asset_amount: i64,
    
    /// Entry price
    pub entry_price: u64,
    
    /// Last known mark price
    pub last_mark_price: u64,
    
    /// Unrealized PnL
    pub unrealized_pnl: i64,
    
    /// Margin deposited
    pub margin: u64,
    
    /// Leverage used
    pub leverage: u8,
    
    /// Direction
    pub direction: Direction,
    
    /// Last update timestamp
    pub last_update: i64,
}

impl PerpPosition {
    pub const SIZE: usize = 8 + 8 + 8 + 8 + 8 + 1 + 1 + 8;
    
    /// Calculate price multiplier based on perp PnL
    pub fn get_price_multiplier(&self) -> Result<u64> {
        if self.entry_price == 0 {
            return Ok(1_000_000); // No position yet, 1x multiplier
        }
        
        let price_change = if self.direction == Direction::Long {
            self.last_mark_price.saturating_sub(self.entry_price)
        } else {
            self.entry_price.saturating_sub(self.last_mark_price)
        };
        
        let leverage_factor = self.leverage as u64;
        
        // Multiplier = 1 + (price_change / entry_price) * leverage
        let pnl_ratio = (price_change as u128)
            .checked_mul(1_000_000)
            .ok_or(LeveragedMemeError::MathOverflow)?
            .checked_div(self.entry_price as u128)
            .ok_or(LeveragedMemeError::MathOverflow)?;
        
        let leveraged_pnl = pnl_ratio
            .checked_mul(leverage_factor as u128)
            .ok_or(LeveragedMemeError::MathOverflow)?;
        
        let multiplier = (1_000_000 as u128)
            .checked_add(leveraged_pnl)
            .ok_or(LeveragedMemeError::MathOverflow)? as u64;
        
        Ok(multiplier)
    }
    
    /// Check if position is at risk of liquidation
    pub fn is_at_liquidation_risk(&self, maintenance_margin_ratio: u64) -> bool {
        if self.margin == 0 {
            return false;
        }
        
        let position_value = (self.base_asset_amount.abs() as u128)
            .checked_mul(self.last_mark_price as u128)
            .unwrap_or(0) as u64;
        
        if position_value == 0 {
            return false;
        }
        
        let margin_ratio = (self.margin as u128)
            .checked_mul(1_000_000)
            .unwrap_or(0)
            .checked_div(position_value as u128)
            .unwrap_or(0) as u64;
        
        margin_ratio < maintenance_margin_ratio
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq)]
pub enum Direction {
    Long,
    Short,
}

#[account]
pub struct FeeVault {
    /// Token this vault belongs to
    pub token_mint: Pubkey,
    
    /// Total fees collected
    pub total_collected: u64,
    
    /// Fees claimed by creator
    pub creator_claimed: u64,
    
    /// Fees claimed by protocol
    pub protocol_claimed: u64,
    
    /// Creator fee share (basis points)
    pub creator_share_bps: u64,
}

impl FeeVault {
    pub const SIZE: usize = 32 + 8 + 8 + 8 + 8;
}

use crate::errors::LeveragedMemeError;
