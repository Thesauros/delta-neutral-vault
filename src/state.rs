use anchor_lang::prelude::*;

#[account]
pub struct VaultState {
    pub admin: Pubkey,
    pub bump: u8,
    
    // Vault parameters
    pub target_leverage: u8,          // Target leverage (e.g., 2x = 2)
    pub rebalance_threshold: u16,     // Threshold to trigger rebalance (basis points)
    pub max_slippage: u16,           // Maximum allowed slippage (basis points)
    
    // Current positions
    pub total_assets: u64,           // Total assets under management
    pub total_shares: u64,           // Total shares issued
    pub long_position: i64,          // Current long position size
    pub short_position: i64,         // Current short position size
    
    // Performance tracking
    pub total_fees_collected: u64,   // Cumulative fees collected
    pub last_rebalance_time: i64,    // Timestamp of last rebalance
    pub net_deposits: i64,           // Net deposits (deposits - withdrawals)
    
    // Risk management
    pub emergency_stop: bool,        // Emergency stop flag
    pub max_capacity: u64,          // Maximum vault capacity
    
    // Drift integration
    pub drift_user_authority: Pubkey, // Authority for Drift user account
    pub drift_user: Pubkey,          // Drift user account
    pub drift_user_stats: Pubkey,    // Drift user stats account
    
    // Fees
    pub management_fee: u16,         // Annual management fee (basis points)
    pub performance_fee: u16,        // Performance fee (basis points)
    
    // Rebalancing parameters
    pub min_rebalance_interval: i64, // Minimum time between rebalances
    pub delta_threshold: u16,        // Delta threshold for rebalancing (basis points)
    
    // Reserved for future use
    pub reserved: [u64; 32],
}

impl VaultState {
    pub const LEN: usize = 8 +        // discriminator
        32 +                          // admin
        1 +                           // bump
        1 +                           // target_leverage
        2 +                           // rebalance_threshold
        2 +                           // max_slippage
        8 +                           // total_assets
        8 +                           // total_shares
        8 +                           // long_position
        8 +                           // short_position
        8 +                           // total_fees_collected
        8 +                           // last_rebalance_time
        8 +                           // net_deposits
        1 +                           // emergency_stop
        8 +                           // max_capacity
        32 +                          // drift_user_authority
        32 +                          // drift_user
        32 +                          // drift_user_stats
        2 +                           // management_fee
        2 +                           // performance_fee
        8 +                           // min_rebalance_interval
        2 +                           // delta_threshold
        (32 * 8);                     // reserved

    pub fn calculate_delta(&self) -> Result<i64> {
        // Calculate current delta of the vault
        // Delta = long_position + short_position
        // For delta neutral, this should be close to 0
        Ok(self.long_position + self.short_position)
    }

    pub fn calculate_total_value(&self) -> Result<u64> {
        // Calculate total value including positions and cash
        // This would need to fetch current market prices
        // For now, return total_assets as placeholder
        Ok(self.total_assets)
    }

    pub fn calculate_share_price(&self) -> Result<u64> {
        if self.total_shares == 0 {
            return Ok(1_000_000); // Initial share price (6 decimals)
        }
        
        let total_value = self.calculate_total_value()?;
        Ok((total_value as u128 * 1_000_000 / self.total_shares as u128) as u64)
    }

    pub fn needs_rebalance(&self) -> Result<bool> {
        let delta = self.calculate_delta()?;
        let total_value = self.calculate_total_value()?;
        
        if total_value == 0 {
            return Ok(false);
        }
        
        // Calculate delta as percentage of total value
        let delta_percentage = (delta.abs() as u128 * 10_000 / total_value as u128) as u16;
        
        Ok(delta_percentage > self.delta_threshold)
    }

    pub fn can_rebalance(&self, current_time: i64) -> bool {
        if self.emergency_stop {
            return false;
        }
        
        current_time - self.last_rebalance_time >= self.min_rebalance_interval
    }

    pub fn calculate_required_hedge(&self) -> Result<HedgeCalculation> {
        let delta = self.calculate_delta()?;
        
        if delta.abs() < 1000 { // Small delta threshold
            return Ok(HedgeCalculation {
                action: HedgeAction::None,
                amount: 0,
                direction: PositionDirection::Long,
            });
        }
        
        let (action, amount, direction) = if delta > 0 {
            // Too much long exposure, need to short
            (HedgeAction::IncreaseShort, delta.abs() as u64, PositionDirection::Short)
        } else {
            // Too much short exposure, need to long
            (HedgeAction::IncreaseLong, delta.abs() as u64, PositionDirection::Long)
        };
        
        Ok(HedgeCalculation {
            action,
            amount,
            direction,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HedgeCalculation {
    pub action: HedgeAction,
    pub amount: u64,
    pub direction: PositionDirection,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HedgeAction {
    None,
    IncreaseLong,
    IncreaseShort,
    ReduceLong,
    ReduceShort,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PositionDirection {
    Long,
    Short,
}

#[account]
pub struct VaultUser {
    pub user: Pubkey,
    pub vault: Pubkey,
    pub shares: u64,
    pub last_deposit_time: i64,
    pub total_deposits: u64,
    pub total_withdrawals: u64,
    pub bump: u8,
}

impl VaultUser {
    pub const LEN: usize = 8 +        // discriminator
        32 +                          // user
        32 +                          // vault
        8 +                           // shares
        8 +                           // last_deposit_time
        8 +                           // total_deposits
        8 +                           // total_withdrawals
        1;                            // bump
}