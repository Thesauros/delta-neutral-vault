use anchor_lang::prelude::*;

/// Maximum leverage allowed for the vault
pub const MAX_LEVERAGE: u64 = 10;

/// Minimum deposit amount in lamports
pub const MIN_DEPOSIT: u64 = 1_000_000; // 1 USDC

/// Maximum deposit amount in lamports
pub const MAX_DEPOSIT: u64 = 1_000_000_000_000; // 1M USDC

/// Performance fee percentage (basis points)
pub const PERFORMANCE_FEE_BPS: u64 = 200; // 2%

/// Management fee percentage (basis points)
pub const MANAGEMENT_FEE_BPS: u64 = 50; // 0.5%

/// Maximum slippage tolerance (basis points)
pub const MAX_SLIPPAGE_BPS: u64 = 100; // 1%

/// Rebalance threshold (basis points)
pub const REBALANCE_THRESHOLD_BPS: u64 = 500; // 5%

/// Emergency pause cooldown period (slots)
pub const EMERGENCY_PAUSE_COOLDOWN: u64 = 100;

/// Maximum number of positions per vault
pub const MAX_POSITIONS: usize = 10;

/// Drift Protocol program ID
pub const DRIFT_PROGRAM_ID: &str = "dRiftyHA39MWEi3m9aunc5MzRF1JYuBsbn6VPcn33UH";

/// USDC mint address
pub const USDC_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";

/// SOL mint address
pub const SOL_MINT: &str = "So11111111111111111111111111111111111111112";

/// Oracle price precision
pub const PRICE_PRECISION: u64 = 1_000_000_000; // 9 decimals

/// Vault token decimals
pub const VAULT_TOKEN_DECIMALS: u8 = 6;

/// Minimum time between rebalances (slots)
pub const MIN_REBALANCE_INTERVAL: u64 = 1000;

/// Maximum position size as percentage of total vault value
pub const MAX_POSITION_SIZE_PCT: u64 = 2000; // 20%

/// Liquidation threshold (basis points)
pub const LIQUIDATION_THRESHOLD_BPS: u64 = 8000; // 80%

/// Health factor minimum
pub const MIN_HEALTH_FACTOR: u64 = 110; // 1.1

/// Fee collection interval (slots)
pub const FEE_COLLECTION_INTERVAL: u64 = 10000;

/// Strategy execution timeout (slots)
pub const STRATEGY_EXECUTION_TIMEOUT: u64 = 100;

/// Maximum gas price for transactions
pub const MAX_GAS_PRICE: u64 = 5000;

/// Minimum vault TVL for active trading
pub const MIN_VAULT_TVL: u64 = 10_000_000; // 10 USDC

/// Maximum number of concurrent strategies
pub const MAX_CONCURRENT_STRATEGIES: usize = 5;

/// Risk management parameters
pub mod risk {
    use super::*;

    /// Maximum drawdown allowed (basis points)
    pub const MAX_DRAWDOWN_BPS: u64 = 1000; // 10%

    /// Stop loss threshold (basis points)
    pub const STOP_LOSS_BPS: u64 = 500; // 5%

    /// Take profit threshold (basis points)
    pub const TAKE_PROFIT_BPS: u64 = 1000; // 10%

    /// Correlation threshold for position sizing
    pub const CORRELATION_THRESHOLD: u64 = 7000; // 70%

    /// Volatility threshold for position adjustment
    pub const VOLATILITY_THRESHOLD: u64 = 5000; // 50%
}

/// Time constants
pub mod time {
    use super::*;

    /// Slots per second
    pub const SLOTS_PER_SECOND: u64 = 2;

    /// Seconds per day
    pub const SECONDS_PER_DAY: u64 = 86400;

    /// Slots per day
    pub const SLOTS_PER_DAY: u64 = SLOTS_PER_SECOND * SECONDS_PER_DAY;

    /// Minimum lock period (slots)
    pub const MIN_LOCK_PERIOD: u64 = SLOTS_PER_DAY * 7; // 7 days

    /// Maximum lock period (slots)
    pub const MAX_LOCK_PERIOD: u64 = SLOTS_PER_DAY * 365; // 1 year
}
