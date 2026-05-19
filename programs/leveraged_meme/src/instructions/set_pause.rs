use anchor_lang::prelude::*;
// FIX (E0425): `Mint` is referenced in the Accounts struct below but was not imported.
// anchor_lang::prelude::* does not re-export SPL token types; they must be imported explicitly.
use anchor_spl::token::Mint;
use crate::{
    constants::*,
    errors::LeveragedMemeError,
    state::*,
};

#[derive(Accounts)]
pub struct SetPause<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
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
    
    pub clock: Sysvar<'info, Clock>,
}

pub fn handler(ctx: Context<SetPause>, paused: bool) -> Result<()> {
    let token_state = &mut ctx.accounts.token_state;
    
    // Only creator or authority can pause
    require!(
        ctx.accounts.authority.key() == token_state.creator,
        LeveragedMemeError::Unauthorized
    );
    
    token_state.paused = paused;
    token_state.updated_at = ctx.accounts.clock.unix_timestamp;
    
    if paused {
        msg!("⏸️ Token trading PAUSED");
    } else {
        msg!("▶️ Token trading RESUMED");
    }
    
    Ok(())
}
