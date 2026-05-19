use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use crate::{
    // FIX (E0659): `DRIFT_PROGRAM_ID` exists in both `constants` (as `&str`) and
    // `drift_integration` (as `Pubkey`).  The `#[account(address = ...)]` constraint needs a
    // `Pubkey`, so we must use the one from `drift_integration`.  Importing `constants::*` would
    // bring in the `&str` version and cause an "ambiguous" error.  Solution: import constants
    // items explicitly (excluding DRIFT_PROGRAM_ID) so there is no name collision.
    constants::{
        TOKEN_STATE_SEED,
        FEE_VAULT_SEED,
        TRADING_FEE_BPS,
        LEVERAGE_FEE_BPS,
        PROTOCOL_FEE_SHARE,
    },
    errors::LeveragedMemeError,
    state::*,
    drift_integration::*,   // brings in DRIFT_PROGRAM_ID as Pubkey — no ambiguity now
    oracle_integration::*,
};

/// Buy tokens with Drift perp integration
/// Opens/increases perp position on Drift
#[derive(Accounts)]
pub struct BuyWithDrift<'info> {
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
    
    // FIX: curve_state removed as a separate account — see buy.rs and state.rs for rationale.
    
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
    /// CHECK: Protocol's fee wallet
    #[account(mut)]
    pub protocol_fee_account: AccountInfo<'info>,
    
    /// Creator fee account
    /// CHECK: Creator's fee wallet
    #[account(mut)]
    pub creator_fee_account: AccountInfo<'info>,
    
    /// Drift program
    /// CHECK: Drift program ID
    // FIX: DRIFT_PROGRAM_ID now unambiguously refers to the Pubkey from drift_integration.
    #[account(address = DRIFT_PROGRAM_ID)]
    pub drift_program: AccountInfo<'info>,
    
    /// Drift state
    /// CHECK: Drift state account
    #[account(mut)]
    pub drift_state: AccountInfo<'info>,
    
    /// Drift user account (for this token)
    /// CHECK: Drift user PDA
    #[account(mut)]
    pub drift_user: AccountInfo<'info>,
    
    /// Drift user stats
    /// CHECK: Drift user stats
    #[account(mut)]
    pub drift_user_stats: AccountInfo<'info>,
    
    /// Pyth price feed
    /// CHECK: Pyth price account
    pub pyth_price_feed: AccountInfo<'info>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub clock: Sysvar<'info, Clock>,
}

pub fn handler(ctx: Context<BuyWithDrift>, sol_amount: u64) -> Result<()> {
    // Save key before mutable borrows.
    let token_mint_key = ctx.accounts.token_mint.key();

    let token_state = &mut ctx.accounts.token_state;
    
    // Check if paused
    require!(!token_state.paused, LeveragedMemeError::ContractPaused);
    
    // Check if graduated
    require!(!token_state.graduated, LeveragedMemeError::AlreadyGraduated);
    
    // FIX: was `curve_state.real_token_reserve`.
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
    
    // FIX: was `curve_state.calculate_buy_output(...)`.
    let tokens_out = token_state.curve_state.calculate_buy_output(sol_for_curve)?;
    
    require!(tokens_out > 0, LeveragedMemeError::MathOverflow);
    
    // FIX: was `tokens_out <= curve_state.real_token_reserve`.
    require!(
        tokens_out <= token_state.curve_state.real_token_reserve,
        LeveragedMemeError::InsufficientLiquidity
    );
    
    // Get current price from Pyth
    let current_price = fetch_price(
        &ctx.accounts.pyth_price_feed,
        ctx.accounts.clock.slot,
    )?;
    
    // Calculate perp position size
    let perp_increase = calculate_perp_position_size(
        sol_for_curve,
        token_state.leverage,
        current_price,
    )?;
    
    // Open/increase perp position on Drift
    let order_params = OrderParams {
        order_type: OrderType::Market,
        market_type: MarketType::Perp,
        direction: if token_state.direction == Direction::Long {
            PositionDirection::Long
        } else {
            PositionDirection::Short
        },
        base_asset_amount: perp_increase,
        market_index: token_state.perp_market_index,
        ..Default::default()
    };
    
    let seeds = &[
        TOKEN_STATE_SEED,
        token_mint_key.as_ref(),
        &[ctx.bumps.token_state],
    ];
    let signer = &[&seeds[..]];
    
    place_drift_perp_order(
        ctx.accounts.drift_program.to_account_info(),
        ctx.accounts.drift_state.to_account_info(),
        ctx.accounts.drift_user.to_account_info(),
        // FIX: use local mutable reference instead of re-borrowing ctx.accounts.token_state.
        token_state.to_account_info(),
        order_params,
        signer,
    )?;
    
    // Update perp position in state.
    // `perp_position` borrows a sub-field of token_state; Rust field-level splitting allows
    // accessing token_state.curve_state separately after perp_position's last use (NLL).
    let perp_position = &mut token_state.perp_position;
    
    if perp_position.entry_price == 0 {
        // First position
        perp_position.entry_price = current_price;
        perp_position.last_mark_price = current_price;
    }
    
    perp_position.base_asset_amount = perp_position.base_asset_amount
        .checked_add(perp_increase as i64)
        .ok_or(LeveragedMemeError::MathOverflow)?;
    
    perp_position.margin = perp_position.margin
        .checked_add(sol_for_curve)
        .ok_or(LeveragedMemeError::MathOverflow)?;
    
    perp_position.last_update = ctx.accounts.clock.unix_timestamp;
    // perp_position borrow ends here (last use) — token_state.curve_state is now accessible.
    
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
    
    // Transfer leverage fee
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
    
    // FIX: was `curve_state.update_after_buy(...)` — updates embedded field in-place.
    token_state.curve_state.update_after_buy(sol_for_curve, tokens_out)?;
    
    // Transfer tokens from curve to buyer
    let cpi_accounts = token::Transfer {
        from: ctx.accounts.curve_token_account.to_account_info(),
        to: ctx.accounts.buyer_token_account.to_account_info(),
        authority: token_state.to_account_info(),
    };
    
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts,
        signer,
    );
    
    token::transfer(cpi_ctx, tokens_out)?;
    
    // Update token state
    // FIX: removed `token_state.curve_state = **curve_state;` — no longer needed.
    token_state.total_fees_collected = token_state.total_fees_collected
        .checked_add(total_fees)
        .ok_or(LeveragedMemeError::MathOverflow)?;
    token_state.updated_at = ctx.accounts.clock.unix_timestamp;
    
    msg!("✅ Buy with Drift integration successful!");
    msg!("   Tokens received: {}", tokens_out);
    msg!("   Perp position increased: {}", perp_increase);
    msg!("   Current mark price: ${}", current_price / 1_000_000);
    
    // Check if ready for graduation
    if token_state.can_graduate()? {
        msg!("🎓 Token ready for graduation!");
    }
    
    Ok(())
}
