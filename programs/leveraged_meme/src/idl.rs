//! IDL Types for Frontend Integration
//! 
//! This module contains TypeScript-compatible type definitions
//! for generating the Anchor IDL.

use anchor_lang::prelude::*;
use crate::state::*;

/// Export IDL-compatible types
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct TokenStateIDL {
    pub creator: Pubkey,
    pub token_mint: Pubkey,
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub leverage: u8,
    pub direction: Direction,
    pub perp_market_index: u16,
    pub graduated: bool,
    pub created_at: i64,
}

impl From<TokenState> for TokenStateIDL {
    fn from(state: TokenState) -> Self {
        Self {
            creator: state.creator,
            token_mint: state.token_mint,
            name: state.name,
            symbol: state.symbol,
            uri: state.uri,
            leverage: state.leverage,
            direction: state.direction,
            perp_market_index: state.perp_market_index,
            graduated: state.graduated,
            created_at: state.created_at,
        }
    }
}

/// Events emitted by the program
#[event]
pub struct TokenCreated {
    pub creator: Pubkey,
    pub token_mint: Pubkey,
    pub name: String,
    pub symbol: String,
    pub leverage: u8,
    pub direction: Direction,
    pub timestamp: i64,
}

#[event]
pub struct TokenBought {
    pub buyer: Pubkey,
    pub token_mint: Pubkey,
    pub sol_amount: u64,
    pub token_amount: u64,
    pub perp_position_size: u64,
    pub timestamp: i64,
}

#[event]
pub struct TokenSold {
    pub seller: Pubkey,
    pub token_mint: Pubkey,
    pub token_amount: u64,
    pub sol_amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct TokenGraduated {
    pub token_mint: Pubkey,
    pub amm_pool: Pubkey,
    pub final_market_cap: u64,
    pub timestamp: i64,
}

#[event]
pub struct PerpPositionUpdated {
    pub token_mint: Pubkey,
    pub base_asset_amount: i64,
    pub entry_price: u64,
    pub last_mark_price: u64,
    pub unrealized_pnl: i64,
    pub timestamp: i64,
}

#[event]
pub struct LiquidationWarning {
    pub token_mint: Pubkey,
    pub margin_ratio: u64,
    pub threshold: u64,
    pub timestamp: i64,
}
