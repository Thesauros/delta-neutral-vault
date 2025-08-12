use anchor_lang::prelude::*;
use drift::cpi::accounts::*;
use drift::program::Drift;
use drift::state::*;
use drift::instructions::*;

use crate::state::*;
use crate::errors::*;

/// Initialize a new Drift user account for the vault
pub fn initialize_drift_user(
    ctx: Context<InitializeDriftUser>,
    sub_account_id: u16,
) -> Result<()> {
    let vault_state = &ctx.accounts.vault_state;
    
    // Create signer seeds for the vault PDA
    let admin_key = vault_state.admin;
    let bump = vault_state.bump;
    let signer_seeds = &[
        b"vault",
        admin_key.as_ref(),
        &[bump],
    ];
    let signer = &[&signer_seeds[..]];
    
    // Initialize Drift user via CPI
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.drift_program.to_account_info(),
        InitializeUser {
            user: ctx.accounts.drift_user.to_account_info(),
            user_stats: ctx.accounts.drift_user_stats.to_account_info(),
            state: ctx.accounts.drift_state.to_account_info(),
            authority: vault_state.to_account_info(),
            payer: ctx.accounts.payer.to_account_info(),
            rent: ctx.accounts.rent.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
        },
        signer,
    );
    
    drift::cpi::initialize_user(cpi_ctx, sub_account_id, None)?;
    
    Ok(())
}

/// Place a perp order on Drift to hedge positions
pub fn place_perp_order(
    ctx: Context<PlacePerpOrder>,
    order_params: OrderParams,
) -> Result<()> {
    let vault_state = &ctx.accounts.vault_state;
    
    // Validate order parameters
    require!(
        order_params.market_type == MarketType::Perp,
        VaultError::InvalidMarketState
    );
    
    // Create signer seeds for the vault PDA
    let admin_key = vault_state.admin;
    let bump = vault_state.bump;
    let signer_seeds = &[
        b"vault",
        admin_key.as_ref(),
        &[bump],
    ];
    let signer = &[&signer_seeds[..]];
    
    // Place order via CPI
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.drift_program.to_account_info(),
        PlaceOrder {
            state: ctx.accounts.drift_state.to_account_info(),
            user: ctx.accounts.drift_user.to_account_info(),
            user_stats: ctx.accounts.drift_user_stats.to_account_info(),
            authority: vault_state.to_account_info(),
        },
        signer,
    );
    
    drift::cpi::place_perp_order(cpi_ctx, order_params)?;
    
    emit!(OrderPlacedEvent {
        market_index: order_params.market_index,
        direction: order_params.direction,
        base_asset_amount: order_params.base_asset_amount,
        price: order_params.price.unwrap_or(0),
        order_type: order_params.order_type,
        timestamp: Clock::get()?.unix_timestamp,
    });
    
    Ok(())
}

/// Cancel an existing order on Drift
pub fn cancel_order(
    ctx: Context<CancelDriftOrder>,
    order_id: u32,
) -> Result<()> {
    let vault_state = &ctx.accounts.vault_state;
    
    // Create signer seeds for the vault PDA
    let admin_key = vault_state.admin;
    let bump = vault_state.bump;
    let signer_seeds = &[
        b"vault",
        admin_key.as_ref(),
        &[bump],
    ];
    let signer = &[&signer_seeds[..]];
    
    // Cancel order via CPI
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.drift_program.to_account_info(),
        CancelOrder {
            state: ctx.accounts.drift_state.to_account_info(),
            user: ctx.accounts.drift_user.to_account_info(),
            authority: vault_state.to_account_info(),
        },
        signer,
    );
    
    drift::cpi::cancel_order(cpi_ctx, order_id)?;
    
    emit!(OrderCancelledEvent {
        order_id,
        timestamp: Clock::get()?.unix_timestamp,
    });
    
    Ok(())
}

/// Calculate optimal hedge size based on current positions and target delta
pub fn calculate_hedge_order_params(
    vault_state: &VaultState,
    market_index: u16,
    current_position: i64,
    target_delta: i64,
    oracle_price: u64,
) -> Result<OrderParams> {
    let current_delta = vault_state.calculate_delta()?;
    let delta_diff = current_delta - target_delta;
    
    if delta_diff.abs() < 1000 {
        return Err(VaultError::RebalanceNotNeeded.into());
    }
    
    let (direction, base_asset_amount) = if delta_diff > 0 {
        // Too much long exposure, need to short
        (PositionDirection::Short, delta_diff.abs() as u64)
    } else {
        // Too much short exposure, need to long
        (PositionDirection::Long, delta_diff.abs() as u64)
    };
    
    // Calculate limit price with slippage protection
    let slippage_adjustment = oracle_price * vault_state.max_slippage as u64 / 10_000;
    let limit_price = match direction {
        PositionDirection::Long => oracle_price + slippage_adjustment,
        PositionDirection::Short => oracle_price - slippage_adjustment,
    };
    
    Ok(OrderParams {
        order_type: OrderType::Limit,
        market_type: MarketType::Perp,
        direction,
        user_order_id: 0,
        base_asset_amount,
        price: Some(limit_price),
        market_index,
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
    })
}

/// Get current position information from Drift
pub fn get_position_info(
    drift_user: &Account<User>,
    market_index: u16,
) -> Result<(i64, u64)> {
    let position = drift_user
        .perp_positions
        .iter()
        .find(|p| p.market_index == market_index)
        .ok_or(VaultError::InvalidMarketState)?;
    
    let base_asset_amount = position.base_asset_amount;
    let quote_asset_amount = position.quote_asset_amount.abs() as u64;
    
    Ok((base_asset_amount, quote_asset_amount))
}

/// Calculate unrealized PnL for current positions
pub fn calculate_unrealized_pnl(
    drift_user: &Account<User>,
    perp_market: &Account<PerpMarket>,
    oracle_price: u64,
) -> Result<i64> {
    let position = drift_user
        .perp_positions
        .iter()
        .find(|p| p.market_index == perp_market.market_index)
        .ok_or(VaultError::InvalidMarketState)?;
    
    if position.base_asset_amount == 0 {
        return Ok(0);
    }
    
    let entry_price = position.quote_asset_amount.abs() as u64 / position.base_asset_amount.abs() as u64;
    let price_diff = if position.base_asset_amount > 0 {
        // Long position
        oracle_price as i64 - entry_price as i64
    } else {
        // Short position
        entry_price as i64 - oracle_price as i64
    };
    
    let unrealized_pnl = price_diff * position.base_asset_amount.abs() / 1_000_000; // Adjust for precision
    
    Ok(unrealized_pnl)
}

/// Update vault positions based on Drift user account
pub fn sync_vault_positions(
    vault_state: &mut VaultState,
    drift_user: &Account<User>,
    market_indices: &[u16],
) -> Result<()> {
    let mut total_long = 0i64;
    let mut total_short = 0i64;
    
    for &market_index in market_indices {
        if let Some(position) = drift_user
            .perp_positions
            .iter()
            .find(|p| p.market_index == market_index)
        {
            if position.base_asset_amount > 0 {
                total_long += position.base_asset_amount;
            } else {
                total_short += position.base_asset_amount;
            }
        }
    }
    
    vault_state.long_position = total_long;
    vault_state.short_position = total_short;
    
    Ok(())
}

// Account contexts for Drift integration
#[derive(Accounts)]
pub struct InitializeDriftUser<'info> {
    #[account(mut)]
    pub vault_state: Account<'info, VaultState>,
    
    /// CHECK: Drift user account
    #[account(mut)]
    pub drift_user: UncheckedAccount<'info>,
    
    /// CHECK: Drift user stats
    #[account(mut)]
    pub drift_user_stats: UncheckedAccount<'info>,
    
    /// CHECK: Drift state
    #[account(mut)]
    pub drift_state: UncheckedAccount<'info>,
    
    #[account(mut)]
    pub payer: Signer<'info>,
    
    pub drift_program: Program<'info, Drift>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct PlacePerpOrder<'info> {
    #[account(
        seeds = [b"vault", vault_state.admin.as_ref()],
        bump = vault_state.bump
    )]
    pub vault_state: Account<'info, VaultState>,
    
    /// CHECK: Drift user account
    #[account(mut)]
    pub drift_user: UncheckedAccount<'info>,
    
    /// CHECK: Drift user stats
    #[account(mut)]
    pub drift_user_stats: UncheckedAccount<'info>,
    
    /// CHECK: Drift state
    #[account(mut)]
    pub drift_state: UncheckedAccount<'info>,
    
    pub drift_program: Program<'info, Drift>,
}

#[derive(Accounts)]
pub struct CancelDriftOrder<'info> {
    #[account(
        seeds = [b"vault", vault_state.admin.as_ref()],
        bump = vault_state.bump
    )]
    pub vault_state: Account<'info, VaultState>,
    
    /// CHECK: Drift user account
    #[account(mut)]
    pub drift_user: UncheckedAccount<'info>,
    
    /// CHECK: Drift state
    #[account(mut)]
    pub drift_state: UncheckedAccount<'info>,
    
    pub drift_program: Program<'info, Drift>,
}

// Events for Drift operations
#[event]
pub struct OrderPlacedEvent {
    pub market_index: u16,
    pub direction: PositionDirection,
    pub base_asset_amount: u64,
    pub price: u64,
    pub order_type: OrderType,
    pub timestamp: i64,
}

#[event]
pub struct OrderCancelledEvent {
    pub order_id: u32,
    pub timestamp: i64,
}

#[event]
pub struct PositionUpdatedEvent {
    pub market_index: u16,
    pub old_position: i64,
    pub new_position: i64,
    pub pnl: i64,
    pub timestamp: i64,
}