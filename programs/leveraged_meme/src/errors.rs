use anchor_lang::prelude::*;

#[error_code]
pub enum LeveragedMemeError {
    #[msg("Name too long (max 32 characters)")]
    NameTooLong,
    
    #[msg("Symbol too long (max 10 characters)")]
    SymbolTooLong,
    
    #[msg("Leverage too high (max 5x)")]
    LeverageTooHigh,
    
    #[msg("Leverage too low (min 2x)")]
    LeverageTooLow,
    
    #[msg("Invalid perp market index")]
    InvalidPerpMarket,
    
    #[msg("Token already graduated")]
    AlreadyGraduated,
    
    #[msg("Token not yet graduated")]
    NotGraduated,
    
    #[msg("Insufficient liquidity in curve")]
    InsufficientLiquidity,
    
    #[msg("Slippage tolerance exceeded")]
    SlippageExceeded,
    
    #[msg("Perp position would be liquidated")]
    LiquidationRisk,
    
    #[msg("Graduation threshold not reached")]
    GraduationThresholdNotMet,
    
    #[msg("Contract is paused")]
    ContractPaused,
    
    #[msg("Unauthorized")]
    Unauthorized,
    
    #[msg("Math overflow")]
    MathOverflow,
    
    #[msg("Invalid fee configuration")]
    InvalidFeeConfig,
    
    #[msg("Drift integration error")]
    DriftError,
    
    #[msg("Price oracle error")]
    OracleError,
    
    #[msg("Bonding curve invariant violated")]
    InvariantViolation,
}
