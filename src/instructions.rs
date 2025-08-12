use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint, transfer, Transfer};
use drift::cpi::accounts::{PlaceOrder, CancelOrder};
use drift::program::Drift;
use drift::state::{UserStats, User, Order, OrderType, OrderStatus, MarketType, PositionDirection};

use crate::errors::*;
use crate::state::*;
use crate::events::*;
use crate::constants::*;
use crate::utils::*;

pub fn initialize_vault(
    ctx: Context<InitializeVault>,
    target_leverage: u8,
    rebalance_threshold: u16,
    max_slippage: u16,
) -> Result<()> {
    // Validate parameters
    require!(
        target_leverage > 0 && target_leverage <= MAX_LEVERAGE,
        DeltaNeutralVaultError::InvalidLeverage
    );
    
    require!(
        rebalance_threshold >= MIN_REBALANCE_THRESHOLD_BPS && 
        rebalance_threshold <= MAX_REBALANCE_THRESHOLD_BPS,
        DeltaNeutralVaultError::InvalidRebalanceThreshold
    );
    
    require!(
        max_slippage <= MAX_SLIPPAGE_BPS,
        DeltaNeutralVaultError::InvalidSlippage
    );

    let vault_state = &mut ctx.accounts.vault_state;
    let clock = Clock::get()?;

    // Initialize vault state
    vault_state.admin = ctx.accounts.admin.key();
    vault_state.bump = ctx.bumps.vault_state;
    vault_state.target_leverage = target_leverage;
    vault_state.rebalance_threshold = rebalance_threshold;
    vault_state.max_slippage = max_slippage;
    vault_state.total_assets = 0;
    vault_state.total_shares = 0;
    vault_state.long_position = 0;
    vault_state.short_position = 0;
    vault_state.total_fees_collected = 0;
    vault_state.last_rebalance_time = clock.unix_timestamp;
    vault_state.net_deposits = 0;
    vault_state.emergency_stop = false;
    vault_state.max_capacity = 1_000_000_000_000; // 1M tokens
    vault_state.drift_user_authority = Pubkey::default();
    vault_state.drift_user = Pubkey::default();
    vault_state.drift_user_stats = Pubkey::default();
    vault_state.management_fee = DEFAULT_MANAGEMENT_FEE_BPS;
    vault_state.performance_fee = DEFAULT_PERFORMANCE_FEE_BPS;
    vault_state.min_rebalance_interval = MIN_REBALANCE_INTERVAL;
    vault_state.delta_threshold = rebalance_threshold;

    // Emit event
    emit!(VaultInitialized {
        vault: vault_state.key(),
        admin: ctx.accounts.admin.key(),
        target_leverage,
        rebalance_threshold,
        max_slippage,
        timestamp: clock.unix_timestamp,
    });

    msg!("Vault initialized successfully");
    Ok(())
}

pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
    let vault_state = &mut ctx.accounts.vault_state;
    let clock = Clock::get()?;

    // Check if vault is in emergency stop
    require!(!vault_state.emergency_stop, DeltaNeutralVaultError::EmergencyStopActive);

    // Check vault capacity
    require!(
        vault_state.total_assets + amount <= vault_state.max_capacity,
        DeltaNeutralVaultError::VaultAtCapacity
    );

    // Calculate shares to mint
    let shares_to_mint = if vault_state.total_shares == 0 {
        amount
    } else {
        (amount as u128 * vault_state.total_shares as u128 / vault_state.total_assets as u128) as u64
    };

    // Transfer tokens from depositor to vault
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.depositor_token_account.to_account_info(),
            to: ctx.accounts.vault_token_account.to_account_info(),
            authority: ctx.accounts.depositor.to_account_info(),
        },
    );
    transfer(transfer_ctx, amount)?;

    // Update vault state
    vault_state.total_assets += amount;
    vault_state.total_shares += shares_to_mint;
    vault_state.net_deposits += amount as i64;

    // Calculate share price
    let share_price = vault_state.calculate_share_price()?;

    // Emit event
    emit!(DepositEvent {
        vault: vault_state.key(),
        user: ctx.accounts.depositor.key(),
        amount,
        shares_minted: shares_to_mint,
        share_price,
        timestamp: clock.unix_timestamp,
    });

    msg!("Deposit successful: {} tokens, {} shares", amount, shares_to_mint);
    Ok(())
}

pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
    let vault_state = &mut ctx.accounts.vault_state;
    let clock = Clock::get()?;

    // Check if vault is in emergency stop
    require!(!vault_state.emergency_stop, DeltaNeutralVaultError::EmergencyStopActive);

    // Check if vault has sufficient assets
    require!(
        vault_state.total_assets >= amount,
        DeltaNeutralVaultError::InsufficientFunds
    );

    // Calculate shares to burn
    let shares_to_burn = if vault_state.total_shares == 0 {
        0
    } else {
        (amount as u128 * vault_state.total_shares as u128 / vault_state.total_assets as u128) as u64
    };

    // Transfer tokens from vault to withdrawer
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.vault_token_account.to_account_info(),
            to: ctx.accounts.withdrawer_token_account.to_account_info(),
            authority: vault_state.to_account_info(),
        },
    );
    transfer(transfer_ctx, amount)?;

    // Update vault state
    vault_state.total_assets -= amount;
    vault_state.total_shares -= shares_to_burn;
    vault_state.net_deposits -= amount as i64;

    // Calculate share price
    let share_price = vault_state.calculate_share_price()?;

    // Emit event
    emit!(WithdrawEvent {
        vault: vault_state.key(),
        user: ctx.accounts.withdrawer.key(),
        amount,
        shares_burned: shares_to_burn,
        share_price,
        timestamp: clock.unix_timestamp,
    });

    msg!("Withdrawal successful: {} tokens, {} shares", amount, shares_to_burn);
    Ok(())
}

pub fn rebalance(ctx: Context<Rebalance>) -> Result<()> {
    let vault_state = &mut ctx.accounts.vault_state;
    let clock = Clock::get()?;

    // Check if vault is in emergency stop
    require!(!vault_state.emergency_stop, DeltaNeutralVaultError::EmergencyStopActive);

    // Check rebalance cooldown
    require!(
        vault_state.can_rebalance(clock.unix_timestamp),
        DeltaNeutralVaultError::RebalanceCooldown
    );

    // Check if rebalancing is needed
    require!(
        vault_state.needs_rebalance()?,
        DeltaNeutralVaultError::RebalanceNotNeeded
    );

    let delta_before = vault_state.calculate_delta()?;
    
    // Calculate required hedge
    let hedge_calc = vault_state.calculate_required_hedge()?;
    
    match hedge_calc.action {
        HedgeAction::IncreaseLong => {
            // Place long order on Drift
            place_drift_order(
                &ctx.accounts.drift_program,
                &ctx.accounts.drift_user,
                &ctx.accounts.drift_user_stats,
                &ctx.accounts.drift_state,
                hedge_calc.amount,
                PositionDirection::Long,
                vault_state.max_slippage,
            )?;
            vault_state.long_position += hedge_calc.amount as i64;
        },
        HedgeAction::IncreaseShort => {
            // Place short order on Drift
            place_drift_order(
                &ctx.accounts.drift_program,
                &ctx.accounts.drift_user,
                &ctx.accounts.drift_user_stats,
                &ctx.accounts.drift_state,
                hedge_calc.amount,
                PositionDirection::Short,
                vault_state.max_slippage,
            )?;
            vault_state.short_position += hedge_calc.amount as i64;
        },
        HedgeAction::ReduceLong => {
            // Close long position on Drift
            close_drift_position(
                &ctx.accounts.drift_program,
                &ctx.accounts.drift_user,
                &ctx.accounts.drift_user_stats,
                &ctx.accounts.drift_state,
                hedge_calc.amount,
                PositionDirection::Long,
                vault_state.max_slippage,
            )?;
            vault_state.long_position -= hedge_calc.amount as i64;
        },
        HedgeAction::ReduceShort => {
            // Close short position on Drift
            close_drift_position(
                &ctx.accounts.drift_program,
                &ctx.accounts.drift_user,
                &ctx.accounts.drift_user_stats,
                &ctx.accounts.drift_state,
                hedge_calc.amount,
                PositionDirection::Short,
                vault_state.max_slippage,
            )?;
            vault_state.short_position -= hedge_calc.amount as i64;
        },
        HedgeAction::None => {
            msg!("No rebalancing needed");
            return Ok(());
        }
    }

    let delta_after = vault_state.calculate_delta()?;
    vault_state.last_rebalance_time = clock.unix_timestamp;

    // Emit event
    emit!(RebalanceEvent {
        vault: vault_state.key(),
        delta_before,
        delta_after,
        long_position_change: vault_state.long_position,
        short_position_change: vault_state.short_position,
        timestamp: clock.unix_timestamp,
    });

    msg!("Rebalancing completed: delta {} -> {}", delta_before, delta_after);
    Ok(())
}

pub fn emergency_stop(ctx: Context<EmergencyStop>) -> Result<()> {
    let vault_state = &mut ctx.accounts.vault_state;
    let clock = Clock::get()?;

    vault_state.emergency_stop = true;

    // Emit event
    emit!(EmergencyStopEvent {
        vault: vault_state.key(),
        admin: ctx.accounts.admin.key(),
        reason: "Emergency stop triggered by admin".to_string(),
        timestamp: clock.unix_timestamp,
    });

    msg!("Emergency stop activated");
    Ok(())
}

pub fn collect_fees(ctx: Context<CollectFees>) -> Result<()> {
    let vault_state = &mut ctx.accounts.vault_state;
    let clock = Clock::get()?;

    // Calculate management fees
    let time_elapsed = clock.unix_timestamp - vault_state.last_rebalance_time;
    let management_fees = calculate_management_fees(
        vault_state.total_assets,
        vault_state.management_fee,
        time_elapsed,
    )?;

    // Calculate performance fees
    let performance_fees = calculate_performance_fees(
        vault_state.total_assets,
        vault_state.performance_fee,
        vault_state.net_deposits,
    )?;

    let total_fees = management_fees + performance_fees;

    if total_fees > 0 {
        // Transfer fees to collector
        let transfer_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault_token_account.to_account_info(),
                to: ctx.accounts.fee_collector_account.to_account_info(),
                authority: vault_state.to_account_info(),
            },
        );
        transfer(transfer_ctx, total_fees)?;

        // Update vault state
        vault_state.total_assets -= total_fees;
        vault_state.total_fees_collected += total_fees;

        // Emit event
        emit!(FeeCollectionEvent {
            vault: vault_state.key(),
            fee_collector: ctx.accounts.fee_collector.key(),
            management_fees,
            performance_fees,
            total_fees,
            timestamp: clock.unix_timestamp,
        });

        msg!("Fees collected: {} (management: {}, performance: {})", 
             total_fees, management_fees, performance_fees);
    }

    Ok(())
}

pub fn update_vault_params(
    ctx: Context<UpdateVaultParams>,
    target_leverage: Option<u8>,
    rebalance_threshold: Option<u16>,
    max_slippage: Option<u16>,
) -> Result<()> {
    let vault_state = &mut ctx.accounts.vault_state;
    let clock = Clock::get()?;

    // Update parameters if provided
    if let Some(leverage) = target_leverage {
        require!(
            leverage > 0 && leverage <= MAX_LEVERAGE,
            DeltaNeutralVaultError::InvalidLeverage
        );
        vault_state.target_leverage = leverage;
    }

    if let Some(threshold) = rebalance_threshold {
        require!(
            threshold >= MIN_REBALANCE_THRESHOLD_BPS && 
            threshold <= MAX_REBALANCE_THRESHOLD_BPS,
            DeltaNeutralVaultError::InvalidRebalanceThreshold
        );
        vault_state.rebalance_threshold = threshold;
        vault_state.delta_threshold = threshold;
    }

    if let Some(slippage) = max_slippage {
        require!(
            slippage <= MAX_SLIPPAGE_BPS,
            DeltaNeutralVaultError::InvalidSlippage
        );
        vault_state.max_slippage = slippage;
    }

    // Emit event
    emit!(VaultParamsUpdated {
        vault: vault_state.key(),
        admin: ctx.accounts.admin.key(),
        target_leverage,
        rebalance_threshold,
        max_slippage,
        timestamp: clock.unix_timestamp,
    });

    msg!("Vault parameters updated successfully");
    Ok(())
}

pub fn open_position(
    ctx: Context<OpenPosition>,
    market_index: u16,
    size: u64,
    direction: u8,
) -> Result<()> {
    let vault_state = &mut ctx.accounts.vault_state;
    let clock = Clock::get()?;

    // Check if vault is in emergency stop
    require!(!vault_state.emergency_stop, DeltaNeutralVaultError::EmergencyStopActive);

    // Validate direction
    let position_direction = match direction {
        0 => PositionDirection::Long,
        1 => PositionDirection::Short,
        _ => return err!(DeltaNeutralVaultError::InvalidPositionDirection),
    };

    // Place order on Drift
    place_drift_order(
        &ctx.accounts.drift_program,
        &ctx.accounts.drift_user,
        &ctx.accounts.drift_user_stats,
        &ctx.accounts.drift_state,
        size,
        position_direction,
        vault_state.max_slippage,
    )?;

    // Update vault state
    match position_direction {
        PositionDirection::Long => {
            vault_state.long_position += size as i64;
        },
        PositionDirection::Short => {
            vault_state.short_position += size as i64;
        }
    }

    // Emit event
    emit!(PositionOpened {
        vault: vault_state.key(),
        market: Pubkey::default(), // Would need market address
        direction: format!("{:?}", position_direction),
        size,
        price: 0, // Would need to get from Drift
        timestamp: clock.unix_timestamp,
    });

    msg!("Position opened: {} {} on market {}", size, format!("{:?}", position_direction), market_index);
    Ok(())
}

pub fn close_position(
    ctx: Context<ClosePosition>,
    market_index: u16,
    size: u64,
) -> Result<()> {
    let vault_state = &mut ctx.accounts.vault_state;
    let clock = Clock::get()?;

    // Check if vault is in emergency stop
    require!(!vault_state.emergency_stop, DeltaNeutralVaultError::EmergencyStopActive);

    // Determine position direction based on current positions
    let position_direction = if vault_state.long_position > 0 {
        PositionDirection::Long
    } else if vault_state.short_position > 0 {
        PositionDirection::Short
    } else {
        return err!(DeltaNeutralVaultError::NoPositionToClose);
    };

    // Close position on Drift
    close_drift_position(
        &ctx.accounts.drift_program,
        &ctx.accounts.drift_user,
        &ctx.accounts.drift_user_stats,
        &ctx.accounts.drift_state,
        size,
        position_direction,
        vault_state.max_slippage,
    )?;

    // Update vault state
    match position_direction {
        PositionDirection::Long => {
            vault_state.long_position -= size as i64;
        },
        PositionDirection::Short => {
            vault_state.short_position -= size as i64;
        }
    }

    // Emit event
    emit!(PositionClosed {
        vault: vault_state.key(),
        market: Pubkey::default(), // Would need market address
        direction: format!("{:?}", position_direction),
        size,
        price: 0, // Would need to get from Drift
        pnl: 0, // Would need to calculate from Drift
        timestamp: clock.unix_timestamp,
    });

    msg!("Position closed: {} {} on market {}", size, format!("{:?}", position_direction), market_index);
    Ok(())
}

// Helper functions for Drift integration
fn place_drift_order(
    drift_program: &Account<Drift>,
    drift_user: &AccountInfo,
    drift_user_stats: &AccountInfo,
    drift_state: &AccountInfo,
    size: u64,
    direction: PositionDirection,
    max_slippage: u16,
) -> Result<()> {
    // This would implement the actual Drift order placement
    // For now, just log the action
    msg!("Placing Drift order: {} {:?} with max slippage {}", size, direction, max_slippage);
    Ok(())
}

fn close_drift_position(
    drift_program: &Account<Drift>,
    drift_user: &AccountInfo,
    drift_user_stats: &AccountInfo,
    drift_state: &AccountInfo,
    size: u64,
    direction: PositionDirection,
    max_slippage: u16,
) -> Result<()> {
    // This would implement the actual Drift position closing
    // For now, just log the action
    msg!("Closing Drift position: {} {:?} with max slippage {}", size, direction, max_slippage);
    Ok(())
}

fn calculate_management_fees(
    total_assets: u64,
    management_fee_bps: u16,
    time_elapsed: i64,
) -> Result<u64> {
    let annual_fee_rate = management_fee_bps as u128 * 1_000_000 / BASIS_POINTS_DIVISOR as u128;
    let time_fraction = time_elapsed as u128 * 1_000_000 / SECONDS_PER_YEAR as u128;
    let fee = (total_assets as u128 * annual_fee_rate * time_fraction) / 1_000_000_000_000;
    Ok(fee as u64)
}

fn calculate_performance_fees(
    total_assets: u64,
    performance_fee_bps: u16,
    net_deposits: i64,
) -> Result<u64> {
    if net_deposits <= 0 {
        return Ok(0);
    }
    
    let performance = total_assets as i64 - net_deposits;
    if performance <= 0 {
        return Ok(0);
    }
    
    let fee = (performance as u128 * performance_fee_bps as u128) / BASIS_POINTS_DIVISOR as u128;
    Ok(fee as u64)
}