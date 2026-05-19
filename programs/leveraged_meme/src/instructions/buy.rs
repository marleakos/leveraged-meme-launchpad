use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use crate::{
    constants::*,
    errors::LeveragedMemeError,
    state::*,
};

#[derive(Accounts)]
pub struct Buy<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,
    
    #[account(
        mut,
        seeds = [
            TOKEN_STATE_SEED,
            token_mint.key().as_ref(),
        ],
        bump
    )]
    pub token_state: Account<'info, TokenState>,
    
    // FIX: curve_state removed as a separate account — see state.rs for the full explanation.
    // All curve operations now go through token_state.curve_state (the embedded field).
    // Removing this account also eliminates the 11 E0609 "no field on Account<'_, CurveState>"
    // errors, the E0614 dereference errors, and the E0599 method-not-found errors in this file.
    
    #[account(mut)]
    pub token_mint: Account<'info, Mint>,
    
    /// Buyer's token account
    #[account(
        mut,
        token::mint = token_mint,
        token::authority = buyer,
    )]
    pub buyer_token_account: Account<'info, TokenAccount>,
    
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

pub fn handler(ctx: Context<Buy>, sol_amount: u64) -> Result<()> {
    // Save the mint key before taking mutable borrows so we can use it in CPI seeds later.
    let token_mint_key = ctx.accounts.token_mint.key();

    let token_state = &mut ctx.accounts.token_state;
    
    // Check if paused
    require!(!token_state.paused, LeveragedMemeError::ContractPaused);
    
    // Check if graduated
    require!(!token_state.graduated, LeveragedMemeError::AlreadyGraduated);
    
    // FIX: was `curve_state.real_token_reserve` — now accessed through the embedded field.
    require!(
        token_state.curve_state.real_token_reserve > 0,
        LeveragedMemeError::InsufficientLiquidity
    );
    
    // Calculate fees
    let trading_fee = sol_amount
        .checked_mul(TRADING_FEE_BPS)
        .ok_or(LeveragedMemeError::MathOverflow)?
        .checked_div(10000)
        .ok_or(LeveragedMemeError::MathOverflow)?;
    
    let leverage_fee = sol_amount
        .checked_mul(LEVERAGE_FEE_BPS)
        .ok_or(LeveragedMemeError::MathOverflow)?
        .checked_div(10000)
        .ok_or(LeveragedMemeError::MathOverflow)?;
    
    let total_fees = trading_fee
        .checked_add(leverage_fee)
        .ok_or(LeveragedMemeError::MathOverflow)?;
    
    let sol_for_curve = sol_amount
        .checked_sub(total_fees)
        .ok_or(LeveragedMemeError::MathOverflow)?;
    
    // FIX: was `curve_state.calculate_buy_output(...)` — now called on the embedded struct.
    let tokens_out = token_state.curve_state.calculate_buy_output(sol_for_curve)?;
    
    require!(tokens_out > 0, LeveragedMemeError::MathOverflow);
    
    // FIX: was `tokens_out <= curve_state.real_token_reserve`.
    require!(
        tokens_out <= token_state.curve_state.real_token_reserve,
        LeveragedMemeError::InsufficientLiquidity
    );
    
    // Transfer SOL from buyer to curve
    let cpi_context = CpiContext::new(
        ctx.accounts.system_program.to_account_info(),
        anchor_lang::system_program::Transfer {
            from: ctx.accounts.buyer.to_account_info(),
            to: ctx.accounts.curve_token_account.to_account_info(),
        },
    );
    
    anchor_lang::system_program::transfer(cpi_context, sol_for_curve)?;
    
    // Transfer fees
    let protocol_fee = trading_fee
        .checked_mul(PROTOCOL_FEE_SHARE)
        .ok_or(LeveragedMemeError::MathOverflow)?
        .checked_div(100)
        .ok_or(LeveragedMemeError::MathOverflow)?;
    
    let creator_fee = trading_fee
        .checked_sub(protocol_fee)
        .ok_or(LeveragedMemeError::MathOverflow)?;
    
    // Transfer protocol fee
    let cpi_context = CpiContext::new(
        ctx.accounts.system_program.to_account_info(),
        anchor_lang::system_program::Transfer {
            from: ctx.accounts.buyer.to_account_info(),
            to: ctx.accounts.protocol_fee_account.to_account_info(),
        },
    );
    anchor_lang::system_program::transfer(cpi_context, protocol_fee)?;
    
    // Transfer creator fee
    let cpi_context = CpiContext::new(
        ctx.accounts.system_program.to_account_info(),
        anchor_lang::system_program::Transfer {
            from: ctx.accounts.buyer.to_account_info(),
            to: ctx.accounts.creator_fee_account.to_account_info(),
        },
    );
    anchor_lang::system_program::transfer(cpi_context, creator_fee)?;
    
    // Transfer leverage fee to protocol
    let cpi_context = CpiContext::new(
        ctx.accounts.system_program.to_account_info(),
        anchor_lang::system_program::Transfer {
            from: ctx.accounts.buyer.to_account_info(),
            to: ctx.accounts.protocol_fee_account.to_account_info(),
        },
    );
    anchor_lang::system_program::transfer(cpi_context, leverage_fee)?;
    
    // Update fee vault
    let fee_vault = &mut ctx.accounts.fee_vault;
    fee_vault.total_collected = fee_vault.total_collected
        .checked_add(total_fees)
        .ok_or(LeveragedMemeError::MathOverflow)?;
    
    // FIX: was `curve_state.update_after_buy(...)` on a separate Account<> — now updates the
    // embedded field in-place, so token_state is dirty and will be persisted by Anchor.
    token_state.curve_state.update_after_buy(sol_for_curve, tokens_out)?;
    
    // Transfer tokens from curve to buyer
    let seeds = &[
        TOKEN_STATE_SEED,
        token_mint_key.as_ref(),
        &[ctx.bumps.token_state],
    ];
    
    let signer = &[&seeds[..]];
    
    let cpi_accounts = token::Transfer {
        from: ctx.accounts.curve_token_account.to_account_info(),
        to: ctx.accounts.buyer_token_account.to_account_info(),
        // FIX: use the local mutable reference instead of re-borrowing ctx.accounts.token_state
        // (which is already mutably borrowed above). Reborrowing as &self via deref is valid.
        authority: token_state.to_account_info(),
    };
    
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts,
        signer,
    );
    
    token::transfer(cpi_ctx, tokens_out)?;
    
    // Update token state
    // FIX: removed `token_state.curve_state = **curve_state;` — the embedded curve_state was
    // already mutated in-place above (update_after_buy), so no copy-back is needed.
    token_state.total_fees_collected = token_state.total_fees_collected
        .checked_add(total_fees)
        .ok_or(LeveragedMemeError::MathOverflow)?;
    token_state.updated_at = ctx.accounts.clock.unix_timestamp;
    
    // TODO: Open/increase perp position on Drift
    let perp_increase = sol_for_curve
        .checked_mul(token_state.leverage as u64)
        .ok_or(LeveragedMemeError::MathOverflow)?;
    
    msg!("Bought {} tokens for {} SOL", tokens_out, sol_amount);
    msg!("Fees: {} SOL", total_fees);
    msg!("Perp position increased by: {}", perp_increase);
    
    // Check if ready for graduation
    if token_state.can_graduate()? {
        msg!("Token ready for graduation!");
    }
    
    Ok(())
}
