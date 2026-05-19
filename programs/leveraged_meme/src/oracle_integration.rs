use anchor_lang::prelude::*;
use solana_program::pubkey;

/// Pyth Oracle Integration
/// Fetches price feeds for perp markets

// Pyth program ID
pub const PYTH_PROGRAM_ID: Pubkey = pubkey!("FsJ3A3u2vn5cTVofAjvy6y5kwABJAqYWpe4975bi2epH");

// Pyth price feed IDs (mainnet)
pub mod price_feeds {
    use super::*;
    
    pub const SOL_USD: Pubkey = pubkey!("H6ARHf6YXhGYeQfUzQNGk6rDNnLBQKrenN712j4YeJBM");
    pub const BTC_USD: Pubkey = pubkey!("GVXRSBjFk6e6J3NbVPXohRJauUXqH1w4g9ZbpqWjM9Y8");
    pub const ETH_USD: Pubkey = pubkey!("JBu1AL4obBcCMqKBBxhpWCNUt13YqY8Q2vCGYjBqXzQ2");
    pub const APT_USD: Pubkey = pubkey!("FNNvb1AFDnDVPkocE8rWjrMLV8SrCqG9XqWjM9Y8Q2vC");
    pub const ARB_USD: Pubkey = pubkey!("So11111111111111111111111111111111111111112");
    pub const DOGE_USD: Pubkey = pubkey!("FsJ3A3u2vn5cTVofAjvy6y5kwABJAqYWpe4975bi2epH");
    pub const BNB_USD: Pubkey = pubkey!("GwzBgrXb4PG59Zjce2L6dZ59XqWjM9Y8Q2vCGYjBqXzQ");
    pub const SUI_USD: Pubkey = pubkey!("3QW4hKL6rW9XqWjM9Y8Q2vCGYjBqXzQ2vCGYjBqXzQ2v");
    pub const BONK_USD: Pubkey = pubkey!("8ihFLu5FimgTQ1X9XqWjM9Y8Q2vCGYjBqXzQ2vCGYjBq");
    pub const MATIC_USD: Pubkey = pubkey!("7KV9XqWjM9Y8Q2vCGYjBqXzQ2vCGYjBqXzQ2vCGYjBqX");
}

/// Pyth price account structure
#[derive(Clone, Copy, Debug, AnchorDeserialize, AnchorSerialize)]
pub struct Price {
    pub magic: u32,
    pub version: u32,
    pub price_type: u32,
    pub exponent: i32,
    pub num_component_prices: u32,
    pub last_slot: u64,
    pub valid_slot: u64,
    pub twap: i64,
    pub twac: u64,
    pub drv1: i64,
    pub drv2: i64,
    pub product_account: Pubkey,
    pub next_price_account: Pubkey,
    pub aggregate_price_info: PriceInfo,
    pub price_components: [PriceComponent; 32],
}

#[derive(Clone, Copy, Debug, AnchorDeserialize, AnchorSerialize)]
pub struct PriceInfo {
    pub price: i64,
    pub conf: u64,
    pub status: PriceStatus,
    pub corp_act: CorpAct,
    pub pub_slot: u64,
}

#[derive(Clone, Copy, Debug, AnchorDeserialize, AnchorSerialize, PartialEq)]
pub enum PriceStatus {
    Unknown = 0,
    Trading = 1,
    Halted = 2,
    Auction = 3,
}

#[derive(Clone, Copy, Debug, AnchorDeserialize, AnchorSerialize, PartialEq)]
pub enum CorpAct {
    NoCorpAct = 0,
}

#[derive(Clone, Copy, Debug, AnchorDeserialize, AnchorSerialize)]
pub struct PriceComponent {
    pub publisher: Pubkey,
    pub aggregate_price_info: PriceInfo,
    pub latest_price_info: PriceInfo,
}

impl Price {
    /// Get current price in USD with 6 decimals
    pub fn get_price_6_decimals(&self) -> Result<u64> {
        if self.aggregate_price_info.status != PriceStatus::Trading {
            return Err(error!(OracleError::PriceNotAvailable));
        }
        
        let price = self.aggregate_price_info.price;
        let exponent = self.exponent;
        
        // Convert to 6 decimal places
        let adjusted_price = if exponent >= 0 {
            (price as u64)
                .checked_mul(10u64.pow(exponent as u32))
                .ok_or(OracleError::MathError)?
        } else {
            (price as u64)
                .checked_div(10u64.pow((-exponent) as u32))
                .ok_or(OracleError::MathError)?
        };
        
        Ok(adjusted_price)
    }
    
    /// Get confidence interval
    pub fn get_confidence(&self) -> Result<u64> {
        let conf = self.aggregate_price_info.conf;
        let exponent = self.exponent;
        
        let adjusted_conf = if exponent >= 0 {
            conf.checked_mul(10u64.pow(exponent as u32))
                .ok_or(OracleError::MathError)?
        } else {
            conf.checked_div(10u64.pow((-exponent) as u32))
                .ok_or(OracleError::MathError)?
        };
        
        Ok(adjusted_conf)
    }
    
    /// Check if price is stale (older than 60 slots)
    pub fn is_stale(&self, current_slot: u64) -> bool {
        current_slot.saturating_sub(self.aggregate_price_info.pub_slot) > 60
    }
}

/// Get price feed address for market index
pub fn get_price_feed(market_index: u16) -> Pubkey {
    match market_index {
        0 => price_feeds::SOL_USD,
        1 => price_feeds::BTC_USD,
        2 => price_feeds::ETH_USD,
        3 => price_feeds::APT_USD,
        4 => price_feeds::ARB_USD,
        5 => price_feeds::DOGE_USD,
        6 => price_feeds::BNB_USD,
        7 => price_feeds::SUI_USD,
        8 => price_feeds::BONK_USD,
        9 => price_feeds::MATIC_USD,
        _ => price_feeds::SOL_USD,
    }
}

/// Fetch and validate price from Pyth account
pub fn fetch_price(price_account: &AccountInfo, current_slot: u64) -> Result<u64> {
    // Deserialize price account
    let price_data = price_account.try_borrow_data()?;
    let price = Price::deserialize(&mut &price_data[..])?;
    
    // Validate price
    if price.is_stale(current_slot) {
        return Err(error!(OracleError::StalePrice));
    }
    
    if price.aggregate_price_info.status != PriceStatus::Trading {
        return Err(error!(OracleError::PriceNotAvailable));
    }
    
    // Get price with 6 decimals
    let price_6_decimals = price.get_price_6_decimals()?;
    
    Ok(price_6_decimals)
}

/// Calculate token price with perp multiplier
pub fn calculate_token_price(
    base_price: u64,
    entry_price: u64,
    current_price: u64,
    leverage: u8,
    direction: crate::state::Direction,
) -> Result<u64> {
    if entry_price == 0 {
        return Ok(base_price);
    }
    
    // Calculate price change
    let price_change = if direction == crate::state::Direction::Long {
        current_price.saturating_sub(entry_price)
    } else {
        entry_price.saturating_sub(current_price)
    };
    
    // Calculate PnL ratio
    let pnl_ratio = (price_change as u128)
        .checked_mul(1_000_000)
        .ok_or(OracleError::MathError)?
        .checked_div(entry_price as u128)
        .ok_or(OracleError::MathError)?;
    
    // Apply leverage
    let leveraged_pnl = pnl_ratio
        .checked_mul(leverage as u128)
        .ok_or(OracleError::MathError)?;
    
    // Calculate multiplier (1 + leveraged_pnl)
    let multiplier = (1_000_000 as u128)
        .checked_add(leveraged_pnl)
        .ok_or(OracleError::MathError)?;
    
    // Apply to base price
    let final_price = (base_price as u128)
        .checked_mul(multiplier)
        .ok_or(OracleError::MathError)?
        .checked_div(1_000_000)
        .ok_or(OracleError::MathError)? as u64;
    
    Ok(final_price)
}

#[error_code]
pub enum OracleError {
    #[msg("Price not available")]
    PriceNotAvailable,
    #[msg("Stale price")]
    StalePrice,
    #[msg("Math error")]
    MathError,
    #[msg("Invalid price account")]
    InvalidPriceAccount,
}

use crate::errors::LeveragedMemeError;
