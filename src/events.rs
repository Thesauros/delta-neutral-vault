use anchor_lang::prelude::*;

#[event]
pub struct VaultInitialized {
    pub vault: Pubkey,
    pub admin: Pubkey,
    pub target_leverage: u8,
    pub rebalance_threshold: u16,
    pub max_slippage: u16,
    pub timestamp: i64,
}

#[event]
pub struct DepositEvent {
    pub vault: Pubkey,
    pub user: Pubkey,
    pub amount: u64,
    pub shares_minted: u64,
    pub share_price: u64,
    pub timestamp: i64,
}

#[event]
pub struct WithdrawEvent {
    pub vault: Pubkey,
    pub user: Pubkey,
    pub amount: u64,
    pub shares_burned: u64,
    pub share_price: u64,
    pub timestamp: i64,
}

#[event]
pub struct RebalanceEvent {
    pub vault: Pubkey,
    pub delta_before: i64,
    pub delta_after: i64,
    pub long_position_change: i64,
    pub short_position_change: i64,
    pub timestamp: i64,
}

#[event]
pub struct EmergencyStopEvent {
    pub vault: Pubkey,
    pub admin: Pubkey,
    pub reason: String,
    pub timestamp: i64,
}

#[event]
pub struct FeeCollectionEvent {
    pub vault: Pubkey,
    pub fee_collector: Pubkey,
    pub management_fees: u64,
    pub performance_fees: u64,
    pub total_fees: u64,
    pub timestamp: i64,
}

#[event]
pub struct VaultParamsUpdated {
    pub vault: Pubkey,
    pub admin: Pubkey,
    pub target_leverage: Option<u8>,
    pub rebalance_threshold: Option<u16>,
    pub max_slippage: Option<u16>,
    pub timestamp: i64,
}

#[event]
pub struct PositionOpened {
    pub vault: Pubkey,
    pub market: Pubkey,
    pub direction: String,
    pub size: u64,
    pub price: u64,
    pub timestamp: i64,
}

#[event]
pub struct PositionClosed {
    pub vault: Pubkey,
    pub market: Pubkey,
    pub direction: String,
    pub size: u64,
    pub price: u64,
    pub pnl: i64,
    pub timestamp: i64,
}

#[event]
pub struct SlippageExceeded {
    pub vault: Pubkey,
    pub expected_price: u64,
    pub actual_price: u64,
    pub slippage_bps: u16,
    pub max_slippage_bps: u16,
    pub timestamp: i64,
}
