use anchor_lang::prelude::*;
use drift::state::{OracleSource, PerpMarket, SpotMarket};
use crate::errors::VaultError;

pub const PRICE_PRECISION: u64 = 1_000_000;
pub const BASIS_POINTS_DIVISOR: u64 = 10_000;
pub const SECONDS_PER_YEAR: i64 = 365 * 24 * 3600;

/// Calculate the required position size to maintain delta neutrality
pub fn calculate_hedge_amount(
    long_position: i64,
    short_position: i64,
    target_delta: i64,
) -> Result<(i64, bool)> {
    let current_delta = long_position + short_position;
    let delta_diff = current_delta - target_delta;
    
    // If delta_diff > 0, we need to reduce long or increase short
    // If delta_diff < 0, we need to increase long or reduce short
    let amount = delta_diff.abs();
    let should_short = delta_diff > 0;
    
    Ok((amount, should_short))
}

/// Calculate the optimal order size considering slippage and market depth
pub fn calculate_optimal_order_size(
    target_amount: u64,
    market_depth: u64,
    max_slippage_bps: u16,
) -> Result<u64> {
    // Limit order size to a percentage of market depth to minimize slippage
    let max_order_size = market_depth * max_slippage_bps as u64 / BASIS_POINTS_DIVISOR;
    Ok(target_amount.min(max_order_size))
}

/// Calculate the expected slippage for a given order size
pub fn calculate_expected_slippage(
    order_size: u64,
    market_depth: u64,
    base_slippage_bps: u16,
) -> Result<u16> {
    if market_depth == 0 {
        return Err(VaultError::InvalidMarketState.into());
    }
    
    // Simple linear slippage model
    let size_ratio = order_size * BASIS_POINTS_DIVISOR / market_depth;
    let slippage = base_slippage_bps as u64 * size_ratio / BASIS_POINTS_DIVISOR;
    
    Ok(slippage.min(u16::MAX as u64) as u16)
}

/// Calculate management fees accrued over time
pub fn calculate_management_fees(
    total_assets: u64,
    fee_rate_bps: u16,
    time_elapsed_seconds: i64,
) -> Result<u64> {
    if time_elapsed_seconds <= 0 {
        return Ok(0);
    }
    
    let annual_fee = total_assets as u128 * fee_rate_bps as u128 / BASIS_POINTS_DIVISOR as u128;
    let fees = annual_fee * time_elapsed_seconds as u128 / SECONDS_PER_YEAR as u128;
    
    Ok(fees.min(u64::MAX as u128) as u64)
}

/// Calculate performance fees based on profits
pub fn calculate_performance_fees(
    total_value: u64,
    net_deposits: i64,
    fee_rate_bps: u16,
) -> Result<u64> {
    if net_deposits < 0 || total_value <= net_deposits as u64 {
        return Ok(0); // No profit, no performance fee
    }
    
    let profit = total_value - net_deposits as u64;
    let performance_fee = profit as u128 * fee_rate_bps as u128 / BASIS_POINTS_DIVISOR as u128;
    
    Ok(performance_fee.min(u64::MAX as u128) as u64)
}

/// Calculate share price with proper precision handling
pub fn calculate_share_price(total_assets: u64, total_shares: u64) -> Result<u64> {
    if total_shares == 0 {
        return Ok(PRICE_PRECISION); // Initial price
    }
    
    let price = total_assets as u128 * PRICE_PRECISION as u128 / total_shares as u128;
    Ok(price.min(u64::MAX as u128) as u64)
}

/// Calculate shares to mint for a given deposit amount
pub fn calculate_shares_to_mint(
    deposit_amount: u64,
    total_assets: u64,
    total_shares: u64,
) -> Result<u64> {
    if total_shares == 0 {
        return Ok(deposit_amount); // 1:1 for first deposit
    }
    
    let share_price = calculate_share_price(total_assets, total_shares)?;
    let shares = deposit_amount as u128 * PRICE_PRECISION as u128 / share_price as u128;
    
    Ok(shares.min(u64::MAX as u128) as u64)
}

/// Calculate withdrawal amount for a given number of shares
pub fn calculate_withdrawal_amount(
    shares_to_burn: u64,
    total_assets: u64,
    total_shares: u64,
) -> Result<u64> {
    if total_shares == 0 {
        return Err(VaultError::InsufficientShares.into());
    }
    
    let withdrawal_amount = shares_to_burn as u128 * total_assets as u128 / total_shares as u128;
    Ok(withdrawal_amount.min(u64::MAX as u128) as u64)
}

/// Calculate the current delta as a percentage of total value
pub fn calculate_delta_percentage(
    long_position: i64,
    short_position: i64,
    total_value: u64,
) -> Result<u16> {
    if total_value == 0 {
        return Ok(0);
    }
    
    let delta = long_position + short_position;
    let delta_percentage = delta.abs() as u128 * BASIS_POINTS_DIVISOR as u128 / total_value as u128;
    
    Ok(delta_percentage.min(u16::MAX as u128) as u16)
}

/// Check if a rebalance is needed based on delta threshold
pub fn should_rebalance(
    long_position: i64,
    short_position: i64,
    total_value: u64,
    delta_threshold_bps: u16,
) -> Result<bool> {
    let delta_percentage = calculate_delta_percentage(long_position, short_position, total_value)?;
    Ok(delta_percentage > delta_threshold_bps)
}

/// Validate that vault parameters are within acceptable ranges
pub fn validate_vault_params(
    target_leverage: u8,
    rebalance_threshold: u16,
    max_slippage: u16,
    management_fee: u16,
    performance_fee: u16,
) -> Result<()> {
    require!(target_leverage > 0 && target_leverage <= 10, VaultError::InvalidLeverage);
    require!(rebalance_threshold <= 1000, VaultError::InvalidThreshold); // Max 10%
    require!(max_slippage <= 500, VaultError::InvalidSlippage); // Max 5%
    require!(management_fee <= 500, VaultError::InvalidAmount); // Max 5% annual
    require!(performance_fee <= 2000, VaultError::InvalidAmount); // Max 20%
    
    Ok(())
}

/// Calculate the maximum position size based on available capital and leverage
pub fn calculate_max_position_size(
    available_capital: u64,
    leverage: u8,
    safety_margin_bps: u16,
) -> Result<u64> {
    let max_theoretical = available_capital * leverage as u64;
    let safety_adjustment = max_theoretical * safety_margin_bps as u64 / BASIS_POINTS_DIVISOR;
    
    Ok(max_theoretical - safety_adjustment)
}

/// Format a price for display (assuming 6 decimal places)
pub fn format_price(price: u64) -> String {
    let integer_part = price / PRICE_PRECISION;
    let fractional_part = price % PRICE_PRECISION;
    format!("{}.{:06}", integer_part, fractional_part)
}

/// Parse a price string into internal representation
pub fn parse_price(price_str: &str) -> Result<u64> {
    let parts: Vec<&str> = price_str.split('.').collect();
    if parts.len() > 2 {
        return Err(VaultError::PriceCalculationFailed.into());
    }
    
    let integer_part: u64 = parts[0].parse()
        .map_err(|_| VaultError::PriceCalculationFailed)?;
    
    let fractional_part = if parts.len() == 2 {
        let frac_str = format!("{:0<6}", parts[1]); // Pad with zeros
        let frac_str = &frac_str[..6.min(frac_str.len())]; // Take max 6 digits
        frac_str.parse::<u64>()
            .map_err(|_| VaultError::PriceCalculationFailed)?
    } else {
        0
    };
    
    Ok(integer_part * PRICE_PRECISION + fractional_part)
}

/// Check if two prices are within acceptable slippage tolerance
pub fn is_within_slippage_tolerance(
    expected_price: u64,
    actual_price: u64,
    max_slippage_bps: u16,
) -> bool {
    let price_diff = if actual_price > expected_price {
        actual_price - expected_price
    } else {
        expected_price - actual_price
    };
    
    let max_allowed_diff = expected_price * max_slippage_bps as u64 / BASIS_POINTS_DIVISOR;
    price_diff <= max_allowed_diff
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_hedge_amount() {
        let (amount, should_short) = calculate_hedge_amount(1000, -500, 0).unwrap();
        assert_eq!(amount, 500);
        assert_eq!(should_short, true);
        
        let (amount, should_short) = calculate_hedge_amount(500, -1000, 0).unwrap();
        assert_eq!(amount, 500);
        assert_eq!(should_short, false);
    }

    #[test]
    fn test_calculate_shares_to_mint() {
        // First deposit
        let shares = calculate_shares_to_mint(1000, 0, 0).unwrap();
        assert_eq!(shares, 1000);
        
        // Subsequent deposit
        let shares = calculate_shares_to_mint(1000, 1000, 1000).unwrap();
        assert_eq!(shares, 1000);
    }

    #[test]
    fn test_format_parse_price() {
        let price = 1_500_000; // 1.5
        let formatted = format_price(price);
        assert_eq!(formatted, "1.500000");
        
        let parsed = parse_price("1.5").unwrap();
        assert_eq!(parsed, 1_500_000);
    }
}