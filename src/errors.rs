use anchor_lang::prelude::*;

#[error_code]
pub enum VaultError {
    #[msg("Invalid leverage parameter")]
    InvalidLeverage,
    
    #[msg("Invalid rebalance threshold")]
    InvalidThreshold,
    
    #[msg("Invalid slippage parameter")]
    InvalidSlippage,
    
    #[msg("Invalid amount")]
    InvalidAmount,
    
    #[msg("Vault is in emergency stop mode")]
    EmergencyStop,
    
    #[msg("Vault capacity exceeded")]
    CapacityExceeded,
    
    #[msg("Insufficient shares")]
    InsufficientShares,
    
    #[msg("Insufficient liquidity")]
    InsufficientLiquidity,
    
    #[msg("Rebalance attempted too soon")]
    RebalanceTooSoon,
    
    #[msg("Rebalance not needed")]
    RebalanceNotNeeded,
    
    #[msg("Math overflow")]
    MathOverflow,
    
    #[msg("Delta calculation failed")]
    DeltaCalculationFailed,
    
    #[msg("Price calculation failed")]
    PriceCalculationFailed,
    
    #[msg("Drift operation failed")]
    DriftOperationFailed,
    
    #[msg("Unauthorized access")]
    Unauthorized,
    
    #[msg("Invalid market state")]
    InvalidMarketState,
    
    #[msg("Order placement failed")]
    OrderPlacementFailed,
    
    #[msg("Position size mismatch")]
    PositionSizeMismatch,
    
    #[msg("Risk limits exceeded")]
    RiskLimitsExceeded,
    
    #[msg("Invalid oracle price")]
    InvalidOraclePrice,
}