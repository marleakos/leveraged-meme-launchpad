use anchor_lang::prelude::*;
// FIX (E0425): same missing Mint import as set_pause.rs.
use anchor_spl::token::Mint;
use crate::{
    constants::*,
    errors::LeveragedMemeError,
    state::*,
};

#[derive(Accounts)]
pub struct SyncPerpPosition<'info> {
    /// Keeper or anyone can call this
    pub caller: Signer<'info>,
    
    #[account(
        mut,
        seeds = [
            TOKEN_STATE_SEED,
            token_mint.key().as_ref(),
        ],
        bump
    )]
    pub token_state: Account<'info, TokenState>,
    
    pub token_mint: Account<'info, Mint>,
    
    /// Pyth price feed for the perp market
    /// CHECK: Pyth oracle account
    pub pyth_price_feed: AccountInfo<'info>,
    
    pub clock: Sysvar<'info, Clock>,
}

pub fn handler(ctx: Context<SyncPerpPosition>) -> Result<()> {
    let token_state = &mut ctx.accounts.token_state;
    let clock = ctx.accounts.clock.unix_timestamp;
    
    // TODO: Fetch actual price from Pyth oracle
    // For now, placeholder
    let current_price: u64 = 100_000_000; // $100 with 6 decimals
    
    // Update last mark price
    token_state.perp_position.last_mark_price = current_price;
    
    // Calculate unrealized PnL
    if token_state.perp_position.base_asset_amount != 0 && token_state.perp_position.entry_price != 0 {
        let price_diff = if token_state.perp_position.direction == Direction::Long {
            current_price.saturating_sub(token_state.perp_position.entry_price) as i64
        } else {
            token_state.perp_position.entry_price.saturating_sub(current_price) as i64
        };
        
        let pnl = (price_diff as i128)
            .checked_mul(token_state.perp_position.base_asset_amount.abs() as i128)
            .ok_or(LeveragedMemeError::MathOverflow)?
            .checked_div(1_000_000) // Adjust for decimals
            .ok_or(LeveragedMemeError::MathOverflow)? as i64;
        
        token_state.perp_position.unrealized_pnl = pnl;
    }
    
    token_state.perp_position.last_update = clock;
    
    // Check liquidation risk
    let maintenance_margin = 625_000; // 6.25% in basis points
    if token_state.perp_position.is_at_liquidation_risk(maintenance_margin) {
        msg!("⚠️ WARNING: Perp position at liquidation risk!");
        // TODO: Trigger deleveraging
    }
    
    let unrealized_pnl = token_state.perp_position.unrealized_pnl;
    token_state.updated_at = clock;
    
    msg!("Perp position synced");
    msg!("  Current price: ${}", current_price / 1_000_000);
    msg!("  Unrealized PnL: ${}", unrealized_pnl);
    
    Ok(())
}
