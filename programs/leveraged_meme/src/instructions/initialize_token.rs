use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use crate::{
    constants::*,
    errors::LeveragedMemeError,
    state::*,
};

#[derive(Accounts)]
pub struct InitializeToken<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,
    
    /// Token mint (created by program)
    #[account(
        init,
        payer = creator,
        mint::decimals = TOKEN_DECIMALS,
        mint::authority = token_state,
    )]
    pub token_mint: Account<'info, Mint>,

    /// Token state PDA
    #[account(
        init,
        payer = creator,
        space = 8 + TokenState::SIZE,
        seeds = [
            TOKEN_STATE_SEED,
            token_mint.key().as_ref(),
        ],
        bump
    )]
    pub token_state: Account<'info, TokenState>,

    // FIX: curve_state is NOT a separate on-chain account.
    // CurveState is an embedded struct inside TokenState (already serialised as part of it).
    // Having a separate Account<'info, CurveState> requires CurveState to implement
    // AccountSerialize + AccountDeserialize + Owner — traits that only #[account] provides.
    // CurveState uses plain #[derive(AnchorSerialize, AnchorDeserialize)] so those traits are
    // absent, producing 40 E0277 errors. Solution: remove this field entirely and initialise
    // token_state.curve_state directly in the handler below.

    /// Fee vault
    #[account(
        init,
        payer = creator,
        space = 8 + FeeVault::SIZE,
        seeds = [
            FEE_VAULT_SEED,
            token_mint.key().as_ref(),
        ],
        bump
    )]
    pub fee_vault: Account<'info, FeeVault>,
    
    /// Token account for curve reserve
    #[account(
        init,
        payer = creator,
        token::mint = token_mint,
        token::authority = token_state,
    )]
    pub curve_token_account: Account<'info, TokenAccount>,
    
    /// Token account for LP reserve (graduation)
    #[account(
        init,
        payer = creator,
        token::mint = token_mint,
        token::authority = token_state,
    )]
    pub lp_token_account: Account<'info, TokenAccount>,
    
    /// System programs
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
    
    /// Clock for timestamps
    pub clock: Sysvar<'info, Clock>,
}

pub fn handler(
    ctx: Context<InitializeToken>,
    name: String,
    symbol: String,
    uri: String,
    leverage: u8,
    direction: Direction,
    perp_market_index: u16,
) -> Result<()> {
    // Validate inputs
    require!(name.len() <= 32, LeveragedMemeError::NameTooLong);
    require!(symbol.len() <= 10, LeveragedMemeError::SymbolTooLong);
    require!(
        leverage >= MIN_LEVERAGE && leverage <= MAX_LEVERAGE,
        LeveragedMemeError::LeverageTooLow
    );
    require!(
        perp_market_index <= 2,
        LeveragedMemeError::InvalidPerpMarket
    );
    
    let clock = &ctx.accounts.clock;
    let token_mint = ctx.accounts.token_mint.key();
    
    // FIX: Build CurveState as a plain local struct value (no account needed).
    // We assign it into token_state.curve_state below, which serialises it as
    // part of the TokenState account automatically.
    let curve_state = CurveState {
        virtual_sol_reserve: VIRTUAL_SOL_SEED,
        virtual_token_reserve: CURVE_RESERVE_AMOUNT,
        real_sol_reserve: 0,
        real_token_reserve: CURVE_RESERVE_AMOUNT,
        k: (VIRTUAL_SOL_SEED as u128)
            .checked_mul(CURVE_RESERVE_AMOUNT as u128)
            .ok_or(LeveragedMemeError::MathOverflow)?,
    };
    
    // Mint initial tokens to curve reserve
    let cpi_accounts = token::MintTo {
        mint: ctx.accounts.token_mint.to_account_info(),
        to: ctx.accounts.curve_token_account.to_account_info(),
        authority: ctx.accounts.token_state.to_account_info(),
    };
    
    let seeds = &[
        TOKEN_STATE_SEED,
        token_mint.as_ref(),
        &[ctx.bumps.token_state],
    ];
    
    let signer = &[&seeds[..]];
    
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts,
        signer,
    );
    
    token::mint_to(cpi_ctx, CURVE_RESERVE_AMOUNT)?;
    
    // Mint LP reserve tokens
    let cpi_accounts_lp = token::MintTo {
        mint: ctx.accounts.token_mint.to_account_info(),
        to: ctx.accounts.lp_token_account.to_account_info(),
        authority: ctx.accounts.token_state.to_account_info(),
    };
    
    let cpi_ctx_lp = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts_lp,
        signer,
    );
    
    token::mint_to(cpi_ctx_lp, LP_RESERVE_AMOUNT)?;
    
    // Initialize perp position (placeholder - will integrate Drift)
    let perp_position = PerpPosition {
        base_asset_amount: 0,
        entry_price: 0,
        last_mark_price: 0,
        unrealized_pnl: 0,
        margin: 0,
        leverage,
        direction,
        last_update: clock.unix_timestamp,
    };
    
    // Initialize token state — this also persists curve_state via the embedded field.
    let token_state = &mut ctx.accounts.token_state;
    token_state.creator = ctx.accounts.creator.key();
    token_state.token_mint = token_mint;
    token_state.name = name;
    token_state.symbol = symbol;
    token_state.uri = uri;
    token_state.leverage = leverage;
    token_state.direction = direction;
    token_state.perp_market_index = perp_market_index;
    // FIX: plain assignment of the local CurveState value (was `**curve_state` which tried to
    // double-deref an Account<'_, CurveState>, causing E0614 "cannot be dereferenced").
    token_state.curve_state = curve_state;
    token_state.perp_position = perp_position;
    token_state.fee_vault = ctx.accounts.fee_vault.key();
    token_state.graduated = false;
    token_state.amm_pool = None;
    token_state.created_at = clock.unix_timestamp;
    token_state.updated_at = clock.unix_timestamp;
    token_state.paused = false;
    token_state.total_fees_collected = 0;
    
    // Initialize fee vault
    let fee_vault = &mut ctx.accounts.fee_vault;
    fee_vault.token_mint = token_mint;
    fee_vault.total_collected = 0;
    fee_vault.creator_claimed = 0;
    fee_vault.protocol_claimed = 0;
    fee_vault.creator_share_bps = (CREATOR_FEE_SHARE * 100) as u64;
    
    msg!("Token initialized: {}", token_state.name);
    msg!("Symbol: {}", token_state.symbol);
    msg!("Leverage: {}x {:?}", leverage, direction);
    
    Ok(())
}
