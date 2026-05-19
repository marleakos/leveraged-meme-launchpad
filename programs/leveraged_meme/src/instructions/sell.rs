use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use crate::{
    constants::*,
    errors::LeveragedMemeError,
    state::*,
};

#[derive(Accounts)]
pub struct Sell<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,
    
    #[account(
        mut,
        seeds = [
            TOKEN_STATE_SEED,
            token_mint.key().as_ref(),
        ],
        bump
    )]
    pub token_state: Account<'info, TokenState>,
    
    // FIX: curve_state removed as a separate account — see buy.rs and state.rs for rationale.
    // All curve access now routes through token_state.curve_state.
    
    #[account(mut)]
    pub token_mint: Account<'info, Mint>,
    
    /// Seller's token account
    #[account(
        mut,
        token::mint = token_mint,
        token::authority = seller,
    )]
    pub seller_token_account: Account<'info, TokenAccount>,
    
    /// Curve's token account
    #[account(
        mut,
        token::mint = token_mint,
    )]
    pub curve_token_account: Account<'info, TokenAccount>,
    
    /// Fee vault
    #[account(
        mut,
        seeds = [
            FEE_VAULT_SEED,
            token_mint.key().as_ref(),
        ],
        bump
    )]
    pub fee_vault: Account<'info, FeeVault>,
    
    /// Protocol fee account
    /// CHECK: This is the protocol's fee wallet
    #[account(mut)]
    pub protocol_fee_account: AccountInfo<'info>,
    
    /// Creator fee account
    /// CHECK: This is the creator's fee wallet
    #[account(mut)]
    pub creator_fee_account: AccountInfo<'info>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub clock: Sysvar<'info, Clock>,
}

pub fn handler(ctx: Context<Sell>, token_amount: u64) -> Result<()> {
    // Save key before mutable borrows.
    let token_mint_key = ctx.accounts.token_mint.key();

    let token_state = &mut ctx.accounts.token_state;
    
    // Check if paused
    require!(!token_state.paused, LeveragedMemeError::ContractPaused);
    
    // Check seller has enough tokens
    require!(
        ctx.accounts.seller_token_account.amount >= token_amount,
        LeveragedMemeError::InsufficientLiquidity
    );
    
    // FIX: was `curve_state.calculate_sell_output(...)`.
    let gross_sol_out = token_state.curve_state.calculate_sell_output(token_amount)?;
    
    require!(gross_sol_out > 0, LeveragedMemeError::MathOverflow);
    
    // Calculate fees
    let trading_fee = gross_sol_out
        .checked_mul(TRADING_FEE_BPS)
        .ok_or(LeveragedMemeError::MathOverflow)?
        .checked_div(10000)
        .ok_or(LeveragedMemeError::MathOverflow)?;
    
    let leverage_fee = gross_sol_out
        .checked_mul(LEVERAGE_FEE_BPS)
        .ok_or(LeveragedMemeError::MathOverflow)?
        .checked_div(10000)
        .ok_or(LeveragedMemeError::MathOverflow)?;
    
    let total_fees = trading_fee
        .checked_add(leverage_fee)
        .ok_or(LeveragedMemeError::MathOverflow)?;
    
    let net_sol_out = gross_sol_out
        .checked_sub(total_fees)
        .ok_or(LeveragedMemeError::MathOverflow)?;
    
    // FIX: was `net_sol_out <= curve_state.real_sol_reserve`.
    require!(
        net_sol_out <= token_state.curve_state.real_sol_reserve,
        LeveragedMemeError::InsufficientLiquidity
    );
    
    // Transfer tokens from seller to curve
    let cpi_accounts_transfer = token::Transfer {
        from: ctx.accounts.seller_token_account.to_account_info(),
        to: ctx.accounts.curve_token_account.to_account_info(),
        authority: ctx.accounts.seller.to_account_info(),
    };
    
    let cpi_ctx_transfer = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts_transfer,
    );
    
    token::transfer(cpi_ctx_transfer, token_amount)?;
    
    // Burn the tokens
    let seeds = &[
        TOKEN_STATE_SEED,
        token_mint_key.as_ref(),
        &[ctx.bumps.token_state],
    ];
    
    let signer = &[&seeds[..]];
    
    let cpi_accounts_burn = token::Burn {
        mint: ctx.accounts.token_mint.to_account_info(),
        from: ctx.accounts.curve_token_account.to_account_info(),
        // FIX: use the local mutable reference (same reasoning as buy.rs).
        authority: token_state.to_account_info(),
    };
    
    let cpi_ctx_burn = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts_burn,
        signer,
    );
    
    token::burn(cpi_ctx_burn, token_amount)?;
    
    // FIX: was `curve_state.update_after_sell(...)` — now updates the embedded field in-place.
    token_state.curve_state.update_after_sell(token_amount, gross_sol_out)?;
    
    // Transfer SOL from curve to seller
    let cpi_accounts_sol = anchor_lang::system_program::Transfer {
        from: ctx.accounts.curve_token_account.to_account_info(),
        to: ctx.accounts.seller.to_account_info(),
    };
    
    let cpi_ctx_sol = CpiContext::new_with_signer(
        ctx.accounts.system_program.to_account_info(),
        cpi_accounts_sol,
        signer,
    );
    
    anchor_lang::system_program::transfer(cpi_ctx_sol, net_sol_out)?;
    
    // Calculate and distribute fees
    let protocol_fee = trading_fee
        .checked_mul(PROTOCOL_FEE_SHARE)
        .ok_or(LeveragedMemeError::MathOverflow)?
        .checked_div(100)
        .ok_or(LeveragedMemeError::MathOverflow)?;
    
    let creator_fee = trading_fee
        .checked_sub(protocol_fee)
        .ok_or(LeveragedMemeError::MathOverflow)?;
    
    // Transfer protocol fee from curve
    let cpi_accounts_protocol = anchor_lang::system_program::Transfer {
        from: ctx.accounts.curve_token_account.to_account_info(),
        to: ctx.accounts.protocol_fee_account.to_account_info(),
    };
    
    let cpi_ctx_protocol = CpiContext::new_with_signer(
        ctx.accounts.system_program.to_account_info(),
        cpi_accounts_protocol,
        signer,
    );
    
    anchor_lang::system_program::transfer(cpi_ctx_protocol, protocol_fee)?;
    
    // Transfer creator fee from curve
    let cpi_accounts_creator = anchor_lang::system_program::Transfer {
        from: ctx.accounts.curve_token_account.to_account_info(),
        to: ctx.accounts.creator_fee_account.to_account_info(),
    };
    
    let cpi_ctx_creator = CpiContext::new_with_signer(
        ctx.accounts.system_program.to_account_info(),
        cpi_accounts_creator,
        signer,
    );
    
    anchor_lang::system_program::transfer(cpi_ctx_creator, creator_fee)?;
    
    // Transfer leverage fee from curve to protocol
    let cpi_accounts_leverage = anchor_lang::system_program::Transfer {
        from: ctx.accounts.curve_token_account.to_account_info(),
        to: ctx.accounts.protocol_fee_account.to_account_info(),
    };
    
    let cpi_ctx_leverage = CpiContext::new_with_signer(
        ctx.accounts.system_program.to_account_info(),
        cpi_accounts_leverage,
        signer,
    );
    
    anchor_lang::system_program::transfer(cpi_ctx_leverage, leverage_fee)?;
    
    // Update fee vault
    let fee_vault = &mut ctx.accounts.fee_vault;
    fee_vault.total_collected = fee_vault.total_collected
        .checked_add(total_fees)
        .ok_or(LeveragedMemeError::MathOverflow)?;
    
    // Update token state
    // FIX: removed `token_state.curve_state = **curve_state;` — already updated in-place above.
    token_state.total_fees_collected = token_state.total_fees_collected
        .checked_add(total_fees)
        .ok_or(LeveragedMemeError::MathOverflow)?;
    token_state.updated_at = ctx.accounts.clock.unix_timestamp;
    
    // TODO: Reduce perp position on Drift
    let perp_decrease = gross_sol_out
        .checked_mul(token_state.leverage as u64)
        .ok_or(LeveragedMemeError::MathOverflow)?;
    
    msg!("Sold {} tokens for {} SOL", token_amount, net_sol_out);
    msg!("Fees: {} SOL", total_fees);
    msg!("Perp position decreased by: {}", perp_decrease);
    
    Ok(())
}
