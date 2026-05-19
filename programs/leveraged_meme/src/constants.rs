use anchor_lang::prelude::*;

/// Program constants

// Token decimals
pub const TOKEN_DECIMALS: u8 = 6;

// Total supply: 1 billion tokens
pub const TOTAL_SUPPLY: u64 = 1_000_000_000_000_000; // 1B with 6 decimals

// Curve reserve: 75% of supply
pub const CURVE_RESERVE_AMOUNT: u64 = TOTAL_SUPPLY * 75 / 100; // 750M

// LP reserve: 25% of supply (held for graduation)
pub const LP_RESERVE_AMOUNT: u64 = TOTAL_SUPPLY * 25 / 100; // 250M

// Virtual SOL seed: $4,000 worth at launch
pub const VIRTUAL_SOL_SEED: u64 = 4_000_000_000; // 4,000 SOL with 6 decimals

// Graduation threshold: $69,000 market cap
pub const GRADUATION_THRESHOLD_USD: u64 = 69_000_000_000; // $69k with 6 decimals

// Fee configuration
pub const DEPLOY_FEE: u64 = 100_000_000; // 0.1 SOL
pub const TRADING_FEE_BPS: u64 = 50; // 0.5%
pub const LEVERAGE_FEE_BPS: u64 = 10; // 0.1%
pub const PROTOCOL_FEE_SHARE: u64 = 50; // 50% of trading fees
pub const CREATOR_FEE_SHARE: u64 = 50; // 50% to creator

// Leverage limits
pub const MIN_LEVERAGE: u8 = 2;
pub const MAX_LEVERAGE: u8 = 5;

// Perp market indices
pub const SOL_PERP_MARKET: u16 = 0;
pub const BTC_PERP_MARKET: u16 = 1;
pub const ETH_PERP_MARKET: u16 = 2;

// Drift program ID
pub const DRIFT_PROGRAM_ID: &str = "dRiftyHA39MWEi3m9aunc5MzRF1JYuBsbn6VPcn33UH";

// Pyth price feed IDs
pub const PYTH_SOL_FEED: &str = "H6ARHf6YXhGYeQfUzQNGk6rDNnLBQKrenN712j4YeJBM";
pub const PYTH_BTC_FEED: &str = "GVXRSBjFk6e6J3NbVPXohRJauUXqH1w4g9";
pub const PYTH_ETH_FEED: &str = "JBu1AL4obBcCMqKBBxhpWCNUt13";

// Seeds for PDA derivation
pub const TOKEN_STATE_SEED: &[u8] = b"token_state";
pub const CURVE_STATE_SEED: &[u8] = b"curve_state";
pub const PERP_POSITION_SEED: &[u8] = b"perp_position";
pub const FEE_VAULT_SEED: &[u8] = b"fee_vault";
