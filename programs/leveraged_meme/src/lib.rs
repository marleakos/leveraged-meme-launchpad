use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};

// Pump.fun program ID for reference
// 6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P

declare_id!("9siEsegivtASLpuRHzMC9UEBcCuzeKe8iREadFEZqCAP");

pub mod constants;
pub mod errors;
pub mod state;
pub mod instructions;
pub mod drift_integration;
pub mod oracle_integration;
pub mod idl;

use constants::*;
use errors::*;
use state::*;
use instructions::*;

#[program]
pub mod leveraged_meme {
    use super::*;

    /// Initialize a new leveraged meme token
    /// Creates token mint, bonding curve, and opens perp position
    pub fn initialize_token(
        ctx: Context<InitializeToken>,
        name: String,
        symbol: String,
        uri: String,
        leverage: u8,
        direction: Direction,
        perp_market_index: u16,
    ) -> Result<()> {
        instructions::initialize_token::handler(
            ctx,
            name,
            symbol,
            uri,
            leverage,
            direction,
            perp_market_index,
        )
    }

    /// Buy tokens from the bonding curve
    /// Increases perp position and mints tokens
    pub fn buy(ctx: Context<Buy>, amount: u64) -> Result<()> {
        instructions::buy::handler(ctx, amount)
    }

    /// Sell tokens back to the bonding curve
    /// Reduces perp position and burns tokens
    pub fn sell(ctx: Context<Sell>, amount: u64) -> Result<()> {
        instructions::sell::handler(ctx, amount)
    }

    /// Graduate token to Raydium AMM
    /// Closes perp position and migrates liquidity
    pub fn graduate(ctx: Context<Graduate>) -> Result<()> {
        instructions::graduate::handler(ctx)
    }

    /// Emergency pause (authority only)
    pub fn set_pause(ctx: Context<SetPause>, paused: bool) -> Result<()> {
        instructions::set_pause::handler(ctx, paused)
    }

    /// Update perp position (keeper function)
    pub fn sync_perp_position(ctx: Context<SyncPerpPosition>) -> Result<()> {
        instructions::sync_perp_position::handler(ctx)
    }

    /// Buy with Drift integration (full perp support)
    pub fn buy_with_drift(ctx: Context<BuyWithDrift>, amount: u64) -> Result<()> {
        instructions::buy_with_drift::handler(ctx, amount)
    }
}

// Re-export for IDL generation
pub use state::*;
pub use errors::*;
