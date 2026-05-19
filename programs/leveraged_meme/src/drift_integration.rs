use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::solana_program::program::invoke_signed;

/// Drift Protocol Integration
/// This module handles all interactions with Drift's perpetual exchange

// Drift program ID
pub const DRIFT_PROGRAM_ID: Pubkey = pubkey!("dRiftyHA39MWEi3m9aunc5MzRF1JYuBsbn6VPcn33UH");

// Drift state account
pub const DRIFT_STATE: Pubkey = pubkey!("5zpq7DvB6UdFFvpmjPjPEJhtk1bUQpQ8m9J3sL1Y9Xz");

/// Drift instruction discriminators
#[derive(Clone, Copy, Debug, AnchorSerialize, AnchorDeserialize)]
pub enum DriftInstruction {
    InitializeUser = 0,
    Deposit = 1,
    Withdraw = 2,
    PlacePerpOrder = 3,
    CancelPerpOrder = 4,
    CancelPerpOrderByUserId = 5,
    CancelAllPerpOrders = 6,
    UpdateUserIdle = 7,
    UpdateUserName = 8,
    UpdateUserCustomMarginRatio = 9,
    UpdateUserMarginTradingEnabled = 10,
}

/// Perp order parameters
#[derive(Clone, Copy, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct OrderParams {
    pub order_type: OrderType,
    pub market_type: MarketType,
    pub direction: PositionDirection,
    pub user_order_id: u8,
    pub base_asset_amount: u64,
    pub price: u64,
    pub market_index: u16,
    pub reduce_only: bool,
    pub post_only: PostOnlyParam,
    pub immediate_or_cancel: bool,
    pub max_ts: Option<i64>,
    pub trigger_price: Option<u64>,
    pub trigger_condition: OrderTriggerCondition,
    pub oracle_price_offset: Option<i32>,
    pub auction_duration: Option<u16>,
    pub auction_start_price: Option<i64>,
    pub auction_end_price: Option<i64>,
}

impl Default for OrderParams {
    fn default() -> Self {
        Self {
            order_type: OrderType::Market,
            market_type: MarketType::Perp,
            direction: PositionDirection::Long,
            user_order_id: 0,
            base_asset_amount: 0,
            price: 0,
            market_index: 0,
            reduce_only: false,
            post_only: PostOnlyParam::None,
            immediate_or_cancel: false,
            max_ts: None,
            trigger_price: None,
            trigger_condition: OrderTriggerCondition::Above,
            oracle_price_offset: None,
            auction_duration: None,
            auction_start_price: None,
            auction_end_price: None,
        }
    }
}

#[derive(Clone, Copy, AnchorSerialize, AnchorDeserialize, Debug, PartialEq)]
pub enum OrderType {
    Market = 0,
    Limit = 1,
    TriggerMarket = 2,
    TriggerLimit = 3,
    MarketIfTouched = 4,
    LimitIfTouched = 5,
    Oracle = 6,
}

#[derive(Clone, Copy, AnchorSerialize, AnchorDeserialize, Debug, PartialEq)]
pub enum MarketType {
    Spot = 0,
    Perp = 1,
}

#[derive(Clone, Copy, AnchorSerialize, AnchorDeserialize, Debug, PartialEq)]
pub enum PositionDirection {
    Long = 0,
    Short = 1,
}

#[derive(Clone, Copy, AnchorSerialize, AnchorDeserialize, Debug, PartialEq)]
pub enum PostOnlyParam {
    None = 0,
    MustPostOnly = 1,
    TryPostOnly = 2,
    Slide = 3,
}

#[derive(Clone, Copy, AnchorSerialize, AnchorDeserialize, Debug, PartialEq)]
pub enum OrderTriggerCondition {
    Above = 0,
    Below = 1,
    TriggeredAbove = 2,
    TriggeredBelow = 3,
}

/// Deposit collateral to Drift
pub fn deposit_to_drift<'info>(
    drift_program: AccountInfo<'info>,
    state: AccountInfo<'info>,
    user: AccountInfo<'info>,
    user_stats: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    spot_market_vault: AccountInfo<'info>,
    user_token_account: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
    amount: u64,
    market_index: u16,
    signer_seeds: &[&[&[u8]]],
) -> Result<()> {
    let data = DriftInstruction::Deposit.try_to_vec()?;
    
    let accounts = vec![
        AccountMeta::new(state.key(), false),
        AccountMeta::new(user.key(), false),
        AccountMeta::new(user_stats.key(), false),
        AccountMeta::new_readonly(authority.key(), true),
        AccountMeta::new(spot_market_vault.key(), false),
        AccountMeta::new(user_token_account.key(), false),
        AccountMeta::new_readonly(token_program.key(), false),
    ];
    
    let instruction = Instruction {
        program_id: DRIFT_PROGRAM_ID,
        accounts,
        data,
    };
    
    invoke_signed(
        &instruction,
        &[
            drift_program,
            state,
            user,
            user_stats,
            authority,
            spot_market_vault,
            user_token_account,
            token_program,
        ],
        signer_seeds,
    )?;
    
    Ok(())
}

/// Place perp order on Drift
pub fn place_drift_perp_order<'info>(
    drift_program: AccountInfo<'info>,
    state: AccountInfo<'info>,
    user: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    order_params: OrderParams,
    signer_seeds: &[&[&[u8]]],
) -> Result<()> {
    let mut data = DriftInstruction::PlacePerpOrder.try_to_vec()?;
    data.extend_from_slice(&order_params.try_to_vec()?);
    
    let accounts = vec![
        AccountMeta::new_readonly(state.key(), false),
        AccountMeta::new(user.key(), false),
        AccountMeta::new_readonly(authority.key(), true),
    ];
    
    let instruction = Instruction {
        program_id: DRIFT_PROGRAM_ID,
        accounts,
        data,
    };
    
    invoke_signed(
        &instruction,
        &[
            drift_program,
            state,
            user,
            authority,
        ],
        signer_seeds,
    )?;
    
    Ok(())
}

/// Cancel perp order on Drift
pub fn cancel_drift_perp_order<'info>(
    drift_program: AccountInfo<'info>,
    state: AccountInfo<'info>,
    user: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    order_id: Option<u32>,
    signer_seeds: &[&[&[u8]]],
) -> Result<()> {
    let data = if let Some(id) = order_id {
        let mut d = DriftInstruction::CancelPerpOrder.try_to_vec()?;
        d.extend_from_slice(&id.to_le_bytes());
        d
    } else {
        DriftInstruction::CancelAllPerpOrders.try_to_vec()?
    };
    
    let accounts = vec![
        AccountMeta::new_readonly(state.key(), false),
        AccountMeta::new(user.key(), false),
        AccountMeta::new_readonly(authority.key(), true),
    ];
    
    let instruction = Instruction {
        program_id: DRIFT_PROGRAM_ID,
        accounts,
        data,
    };
    
    invoke_signed(
        &instruction,
        &[
            drift_program,
            state,
            user,
            authority,
        ],
        signer_seeds,
    )?;
    
    Ok(())
}

/// Calculate position size based on leverage
pub fn calculate_perp_position_size(
    collateral: u64,
    leverage: u8,
    price: u64,
) -> Result<u64> {
    let notional = (collateral as u128)
        .checked_mul(leverage as u128)
        .ok_or(ErrorCode::MathError)?;
    
    let base_amount = notional
        .checked_mul(1_000_000_000) // Adjust for decimals
        .ok_or(ErrorCode::MathError)?
        .checked_div(price as u128)
        .ok_or(ErrorCode::MathError)? as u64;
    
    Ok(base_amount)
}

/// Get perp market index from token symbol
pub fn get_perp_market_index(symbol: &str) -> u16 {
    match symbol {
        "SOL" => 0,
        "BTC" => 1,
        "ETH" => 2,
        "APT" => 3,
        "ARB" => 4,
        "DOGE" => 5,
        "BNB" => 6,
        "SUI" => 7,
        "1MBONK" => 8,
        "MATIC" => 9,
        _ => 0, // Default to SOL
    }
}

#[error_code]
pub enum ErrorCode {
    #[msg("Math error")]
    MathError,
    #[msg("Drift CPI error")]
    DriftCpiError,
}

use crate::errors::LeveragedMemeError;
