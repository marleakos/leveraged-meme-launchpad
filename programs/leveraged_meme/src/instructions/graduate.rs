use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use crate::{
    constants::*,
    errors::LeveragedMemeError,
    state::*,
};

#[derive(Accounts)]
pub struct Graduate<'info> {
    #[account(mut)]
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
    
    // FIX: curve_state removed as a separate account — see buy.rs and state.rs for rationale.
    // token_state.curve_state already holds the live curve data.
    // Note: the two lines at the bottom of the original handler that wrote to
    // `token_state.curve_state.real_sol_reserve` / `real_token_reserve` directly were already
    // correct in intent; they were only broken because `curve_state` (the account) existed and
    // shadowed them.  With the account gone those lines compile cleanly.
    
    #[account(mut)]
    pub token_mint: Account<'info, Mint>,
    
    /// Curve's token account (will be emptied)
    #[account(
        mut,
        token::mint = token_mint,
    )]
    pub curve_token_account: Account<'info, TokenAccount>,
    
    /// LP's token account (reserve for AMM)
    #[account(
        mut,
        token::mint = token_mint,
    )]
    pub lp_token_account: Account<'info, TokenAccount>,
    
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
    
    /// Protocol fee account for graduation fee
    /// CHECK: Protocol's fee wallet
    #[account(mut)]
    pub protocol_fee_account: AccountInfo<'info>,
    
    /// Creator account
    /// CHECK: Token creator
    #[account(mut)]
    pub creator: AccountInfo<'info>,
    
    /// Raydium AMM pool (to be created)
    /// CHECK: Will be initialized during graduation
    #[account(mut)]
    pub amm_pool: AccountInfo<'info>,
    
    /// Token program
    pub token_program: Program<'info, Token>,
    
    /// System program
    pub system_program: Program<'info, System>,
    
    /// Clock
    pub clock: Sysvar<'info, Clock>,
}

pub fn handler(ctx: Context<Graduate>) -> Result<()> {
    // Save key before mutable borrow.
    let token_mint_key = ctx.accounts.token_mint.key();

    let token_state = &mut ctx.accounts.token_state;
    
    // Check if already graduated
    require!(!token_state.graduated, LeveragedMemeError::AlreadyGraduated);
    
    // Check graduation threshold
    require!(
        token_state.can_graduate()?,
        LeveragedMemeError::GraduationThresholdNotMet
    );
    
    // FIX: was `curve_state.real_sol_reserve` / `curve_state.real_token_reserve` via a
    // separate account. Now reads from the embedded field — no behaviour change.
    let sol_in_curve = token_state.curve_state.real_sol_reserve;
    let tokens_in_curve = token_state.curve_state.real_token_reserve;
    let tokens_in_lp = LP_RESERVE_AMOUNT;
    
    // Calculate graduation fee (1% of SOL in curve)
    let graduation_fee = sol_in_curve
        .checked_mul(100)
        .ok_or(LeveragedMemeError::MathOverflow)?
        .checked_div(10000)
        .ok_or(LeveragedMemeError::MathOverflow)?;
    
    let sol_for_amm = sol_in_curve
        .checked_sub(graduation_fee)
        .ok_or(LeveragedMemeError::MathOverflow)?;
    
    // Total tokens for AMM = curve tokens + LP reserve
    let total_tokens_for_amm = tokens_in_curve
        .checked_add(tokens_in_lp)
        .ok_or(LeveragedMemeError::MathOverflow)?;
    
    let seeds = &[
        TOKEN_STATE_SEED,
        token_mint_key.as_ref(),
        &[ctx.bumps.token_state],
    ];
    let signer = &[&seeds[..]];
    
    // Transfer graduation fee to protocol
    let cpi_accounts_fee = anchor_lang::system_program::Transfer {
        from: ctx.accounts.curve_token_account.to_account_info(),
        to: ctx.accounts.protocol_fee_account.to_account_info(),
    };
    
    let cpi_ctx_fee = CpiContext::new_with_signer(
        ctx.accounts.system_program.to_account_info(),
        cpi_accounts_fee,
        signer,
    );
    
    anchor_lang::system_program::transfer(cpi_ctx_fee, graduation_fee)?;
    
    // TODO: Close perp position on Drift
    msg!("Closing perp position on Drift...");
    
    // TODO: Create Raydium AMM pool
    msg!("Creating Raydium AMM pool...");
    msg!("  SOL: {}", sol_for_amm);
    msg!("  Tokens: {}", total_tokens_for_amm);
    
    // Transfer all tokens from curve to AMM
    let cpi_accounts_curve_tokens = token::Transfer {
        from: ctx.accounts.curve_token_account.to_account_info(),
        to: ctx.accounts.amm_pool.to_account_info(),
        // FIX: use local reference instead of re-borrowing ctx.accounts.token_state.
        authority: token_state.to_account_info(),
    };
    
    let cpi_ctx_curve_tokens = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts_curve_tokens,
        signer,
    );
    
    token::transfer(cpi_ctx_curve_tokens, tokens_in_curve)?;
    
    // Transfer LP reserve tokens to AMM
    let cpi_accounts_lp_tokens = token::Transfer {
        from: ctx.accounts.lp_token_account.to_account_info(),
        to: ctx.accounts.amm_pool.to_account_info(),
        authority: token_state.to_account_info(),
    };
    
    let cpi_ctx_lp_tokens = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts_lp_tokens,
        signer,
    );
    
    token::transfer(cpi_ctx_lp_tokens, tokens_in_lp)?;
    
    // Transfer SOL to AMM
    let cpi_accounts_sol = anchor_lang::system_program::Transfer {
        from: ctx.accounts.curve_token_account.to_account_info(),
        to: ctx.accounts.amm_pool.to_account_info(),
    };
    
    let cpi_ctx_sol = CpiContext::new_with_signer(
        ctx.accounts.system_program.to_account_info(),
        cpi_accounts_sol,
        signer,
    );
    
    anchor_lang::system_program::transfer(cpi_ctx_sol, sol_for_amm)?;
    
    // Capture market cap before mutating state (borrows token_state immutably).
    let final_market_cap = token_state.market_cap()?;
    let amm_pool_key = ctx.accounts.amm_pool.key();
    let clock_ts = ctx.accounts.clock.unix_timestamp;

    // Update token state
    token_state.graduated = true;
    token_state.amm_pool = Some(amm_pool_key);
    token_state.updated_at = clock_ts;
    
    // Zero out the embedded curve reserves — these two lines were already syntactically correct
    // in the original file (they addressed token_state.curve_state directly), so no change
    // needed here other than the removal of the now-gone `curve_state` variable above.
    token_state.curve_state.real_sol_reserve = 0;
    token_state.curve_state.real_token_reserve = 0;
    
    // Update fee vault
    let fee_vault = &mut ctx.accounts.fee_vault;
    fee_vault.total_collected = fee_vault.total_collected
        .checked_add(graduation_fee)
        .ok_or(LeveragedMemeError::MathOverflow)?;
    
    msg!("🎓 Token graduated to Raydium!");
    msg!("  Market cap at graduation: ${}", final_market_cap);
    msg!("  Graduation fee: {} SOL", graduation_fee);
    msg!("  AMM Pool: {}", amm_pool_key);
    
    Ok(())
}
