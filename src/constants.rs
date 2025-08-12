use anchor_lang::prelude::*;

// Program ID
pub const PROGRAM_ID: &str = "DNVt1111111111111111111111111111111111111111";

// Seeds for PDAs
pub const VAULT_SEED: &[u8] = b"vault";
pub const VAULT_TOKEN_ACCOUNT_SEED: &[u8] = b"vault_token_account";
pub const USER_SEED: &[u8] = b"user";

// Fee constants (in basis points)
pub const MAX_MANAGEMENT_FEE_BPS: u16 = 500; // 5%
pub const MAX_PERFORMANCE_FEE_BPS: u16 = 5000; // 50%
pub const DEFAULT_MANAGEMENT_FEE_BPS: u16 = 200; // 2%
pub const DEFAULT_PERFORMANCE_FEE_BPS: u16 = 2000; // 20%

// Risk management constants
pub const MAX_LEVERAGE: u8 = 10;
pub const MIN_REBALANCE_THRESHOLD_BPS: u16 = 10; // 0.1%
pub const MAX_REBALANCE_THRESHOLD_BPS: u16 = 1000; // 10%
pub const MAX_SLIPPAGE_BPS: u16 = 1000; // 10%
pub const MIN_REBALANCE_INTERVAL: i64 = 300; // 5 minutes

// Precision constants
pub const BASIS_POINTS_DIVISOR: u64 = 10_000;
pub const SHARE_PRICE_PRECISION: u64 = 1_000_000; // 6 decimals
pub const PRICE_PRECISION: u64 = 1_000_000; // 6 decimals

// Drift Protocol constants
pub const DRIFT_PROGRAM_ID: &str = "DRiFTvSoSLjH8XJx2wSJ1GL9jR8qXhF2vC2QBK5FyL9";

// Market constants
pub const SOL_MARKET_INDEX: u16 = 0;
pub const BTC_MARKET_INDEX: u16 = 1;
pub const ETH_MARKET_INDEX: u16 = 2;

// Time constants
pub const SECONDS_PER_DAY: i64 = 86400;
pub const SECONDS_PER_YEAR: i64 = 31536000;

// Error messages
pub const ERROR_VAULT_FULL: &str = "Vault is at maximum capacity";
pub const ERROR_INSUFFICIENT_FUNDS: &str = "Insufficient funds for operation";
pub const ERROR_EMERGENCY_STOP: &str = "Vault is in emergency stop mode";
pub const ERROR_REBALANCE_COOLDOWN: &str = "Rebalance cooldown not met";
pub const ERROR_SLIPPAGE_EXCEEDED: &str = "Slippage exceeds maximum allowed";
pub const ERROR_INVALID_LEVERAGE: &str = "Invalid leverage ratio";
pub const ERROR_INVALID_FEES: &str = "Invalid fee structure";

// Event constants
pub const MAX_EVENT_STRING_LENGTH: usize = 100;

// Account size constants
pub const VAULT_STATE_SIZE: usize = 8 + 32 + 1 + 1 + 2 + 2 + 8 + 8 + 8 + 8 + 8 + 8 + 8 + 1 + 8 + 32 + 32 + 32 + 2 + 2 + 8 + 2 + (32 * 8);
pub const USER_STATE_SIZE: usize = 8 + 32 + 32 + 8 + 8 + 8 + 8 + 1;
