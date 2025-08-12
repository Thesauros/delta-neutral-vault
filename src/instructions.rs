use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer};
use drift::cpi::accounts::{PlaceOrder, CancelOrder};
use drift::state::{Order, OrderType, OrderStatus, MarketType, PositionDirection};

use crate::state::*;
use crate::errors::*;
use crate::utils::*;

pub fn initialize_vault(
    ctx: Context<super::InitializeVault>,
    target_leverage: u8,
    rebalance_threshold: u16,
    max_slippage: u16,
) -> Result<()> {
    let vault_state = &mut ctx.accounts.vault_state;
    
    require!(target_leverage > 0 && target_leverage <= 10, VaultError::InvalidLeverage);
    require!(rebalance_threshold <= 1000, VaultError::InvalidThreshold); // Max 10%
    require!(max_slippage <= 500, VaultError::InvalidSlippage); // Max 5%
    
    vault_state.admin = ctx.accounts.admin.key();
    vault_state.bump = *ctx.bumps.get("vault_state").unwrap();
    vault_state.target_leverage = target_leverage;
    vault_state.rebalance_threshold = rebalance_threshold;
    vault_state.max_slippage = max_slippage;
    
    vault_state.total_assets = 0;
    vault_state.total_shares = 0;
    vault_state.long_position = 0;
    vault_state.short_position = 0;
    vault_state.total_fees_collected = 0;
    vault_state.last_rebalance_time = Clock::get()?.unix_timestamp;
    vault_state.net_deposits = 0;
    vault_state.emergency_stop = false;
    vault_state.max_capacity = u64::MAX;
    
    // Initialize Drift-related fields (would be set during Drift user creation)
    vault_state.drift_user_authority = vault_state.key();
    vault_state.drift_user = Pubkey::default();
    vault_state.drift_user_stats = Pubkey::default();
    
    // Default fee structure
    vault_state.management_fee = 200; // 2% annual
    vault_state.performance_fee = 1000; // 10% of profits
    vault_state.min_rebalance_interval = 3600; // 1 hour
    vault_state.delta_threshold = 100; // 1%
    
    vault_state.reserved = [0; 32];
    
    msg!("Vault initialized with target leverage: {}x", target_leverage);
    
    Ok(())
}

pub fn deposit(ctx: Context<super::Deposit>, amount: u64) -> Result<()> {
    let vault_state = &mut ctx.accounts.vault_state;
    
    require!(!vault_state.emergency_stop, VaultError::EmergencyStop);
    require!(amount > 0, VaultError::InvalidAmount);
    require!(
        vault_state.total_assets + amount <= vault_state.max_capacity,
        VaultError::CapacityExceeded
    );
    
    // Calculate shares to mint
    let shares_to_mint = if vault_state.total_shares == 0 {
        amount // 1:1 for first deposit
    } else {
        let share_price = vault_state.calculate_share_price()?;
        (amount as u128 * 1_000_000 / share_price as u128) as u64
    };
    
    // Transfer tokens from user to vault
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.depositor_token_account.to_account_info(),
            to: ctx.accounts.vault_token_account.to_account_info(),
            authority: ctx.accounts.depositor.to_account_info(),
        },
    );
    token::transfer(transfer_ctx, amount)?;
    
    // Update vault state
    vault_state.total_assets += amount;
    vault_state.total_shares += shares_to_mint;
    vault_state.net_deposits += amount as i64;
    
    emit!(DepositEvent {
        user: ctx.accounts.depositor.key(),
        amount,
        shares_minted: shares_to_mint,
        total_assets: vault_state.total_assets,
        total_shares: vault_state.total_shares,
    });
    
    Ok(())
}

pub fn withdraw(ctx: Context<super::Withdraw>, shares: u64) -> Result<()> {
    let vault_state = &mut ctx.accounts.vault_state;
    
    require!(!vault_state.emergency_stop, VaultError::EmergencyStop);
    require!(shares > 0, VaultError::InvalidAmount);
    require!(shares <= vault_state.total_shares, VaultError::InsufficientShares);
    
    // Calculate tokens to return
    let share_price = vault_state.calculate_share_price()?;
    let tokens_to_return = (shares as u128 * share_price as u128 / 1_000_000) as u64;
    
    require!(
        tokens_to_return <= vault_state.total_assets,
        VaultError::InsufficientLiquidity
    );
    
    // Create PDA signer
    let admin_key = vault_state.admin;
    let bump = vault_state.bump;
    let signer_seeds = &[
        b"vault",
        admin_key.as_ref(),
        &[bump],
    ];
    let signer = &[&signer_seeds[..]];
    
    // Transfer tokens from vault to user
    let transfer_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.vault_token_account.to_account_info(),
            to: ctx.accounts.withdrawer_token_account.to_account_info(),
            authority: vault_state.to_account_info(),
        },
        signer,
    );
    token::transfer(transfer_ctx, tokens_to_return)?;
    
    // Update vault state
    vault_state.total_assets -= tokens_to_return;
    vault_state.total_shares -= shares;
    vault_state.net_deposits -= tokens_to_return as i64;
    
    emit!(WithdrawEvent {
        user: ctx.accounts.withdrawer.key(),
        shares_burned: shares,
        amount: tokens_to_return,
        total_assets: vault_state.total_assets,
        total_shares: vault_state.total_shares,
    });
    
    Ok(())
}

pub fn rebalance(ctx: Context<super::Rebalance>) -> Result<()> {
    let vault_state = &mut ctx.accounts.vault_state;
    let current_time = Clock::get()?.unix_timestamp;
    
    require!(!vault_state.emergency_stop, VaultError::EmergencyStop);
    require!(
        vault_state.can_rebalance(current_time),
        VaultError::RebalanceTooSoon
    );
    require!(
        vault_state.needs_rebalance()?,
        VaultError::RebalanceNotNeeded
    );
    
    let hedge_calculation = vault_state.calculate_required_hedge()?;
    
    match hedge_calculation.action {
        HedgeAction::None => {
            msg!("No hedge action required");
            return Ok(());
        }
        HedgeAction::IncreaseLong => {
            place_long_order(ctx, hedge_calculation.amount)?;
            vault_state.long_position += hedge_calculation.amount as i64;
        }
        HedgeAction::IncreaseShort => {
            place_short_order(ctx, hedge_calculation.amount)?;
            vault_state.short_position -= hedge_calculation.amount as i64;
        }
        HedgeAction::ReduceLong => {
            reduce_long_position(ctx, hedge_calculation.amount)?;
            vault_state.long_position -= hedge_calculation.amount as i64;
        }
        HedgeAction::ReduceShort => {
            reduce_short_position(ctx, hedge_calculation.amount)?;
            vault_state.short_position += hedge_calculation.amount as i64;
        }
    }
    
    vault_state.last_rebalance_time = current_time;
    
    emit!(RebalanceEvent {
        delta_before: vault_state.calculate_delta()?,
        action: hedge_calculation.action,
        amount: hedge_calculation.amount,
        timestamp: current_time,
    });
    
    Ok(())
}

pub fn emergency_stop(ctx: Context<super::EmergencyStop>) -> Result<()> {
    let vault_state = &mut ctx.accounts.vault_state;
    vault_state.emergency_stop = true;
    
    emit!(EmergencyStopEvent {
        admin: ctx.accounts.admin.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    
    Ok(())
}

pub fn collect_fees(ctx: Context<super::CollectFees>) -> Result<()> {
    let vault_state = &mut ctx.accounts.vault_state;
    
    // Calculate management fees
    let current_time = Clock::get()?.unix_timestamp;
    let time_elapsed = current_time - vault_state.last_rebalance_time;
    let annual_fee_rate = vault_state.management_fee as u64;
    
    let management_fees = vault_state.total_assets 
        * annual_fee_rate 
        * time_elapsed as u64 
        / (365 * 24 * 3600 * 10_000); // Convert to annual basis points
    
    require!(
        management_fees <= vault_state.total_assets,
        VaultError::InsufficientLiquidity
    );
    
    if management_fees > 0 {
        // Create PDA signer
        let admin_key = vault_state.admin;
        let bump = vault_state.bump;
        let signer_seeds = &[
            b"vault",
            admin_key.as_ref(),
            &[bump],
        ];
        let signer = &[&signer_seeds[..]];
        
        // Transfer fees
        let transfer_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault_token_account.to_account_info(),
                to: ctx.accounts.fee_collector_account.to_account_info(),
                authority: vault_state.to_account_info(),
            },
            signer,
        );
        token::transfer(transfer_ctx, management_fees)?;
        
        vault_state.total_assets -= management_fees;
        vault_state.total_fees_collected += management_fees;
    }
    
    emit!(FeesCollectedEvent {
        amount: management_fees,
        total_fees_collected: vault_state.total_fees_collected,
        timestamp: current_time,
    });
    
    Ok(())
}

pub fn update_vault_params(
    ctx: Context<super::UpdateVaultParams>,
    target_leverage: Option<u8>,
    rebalance_threshold: Option<u16>,
    max_slippage: Option<u16>,
) -> Result<()> {
    let vault_state = &mut ctx.accounts.vault_state;
    
    if let Some(leverage) = target_leverage {
        require!(leverage > 0 && leverage <= 10, VaultError::InvalidLeverage);
        vault_state.target_leverage = leverage;
    }
    
    if let Some(threshold) = rebalance_threshold {
        require!(threshold <= 1000, VaultError::InvalidThreshold);
        vault_state.rebalance_threshold = threshold;
    }
    
    if let Some(slippage) = max_slippage {
        require!(slippage <= 500, VaultError::InvalidSlippage);
        vault_state.max_slippage = slippage;
    }
    
    Ok(())
}

// Helper functions for Drift integration
fn place_long_order(ctx: Context<super::Rebalance>, amount: u64) -> Result<()> {
    // This would place a long order on Drift
    // Implementation depends on Drift's CPI interface
    msg!("Placing long order for amount: {}", amount);
    Ok(())
}

fn place_short_order(ctx: Context<super::Rebalance>, amount: u64) -> Result<()> {
    // This would place a short order on Drift
    // Implementation depends on Drift's CPI interface
    msg!("Placing short order for amount: {}", amount);
    Ok(())
}

fn reduce_long_position(ctx: Context<super::Rebalance>, amount: u64) -> Result<()> {
    // This would reduce long position on Drift
    msg!("Reducing long position by amount: {}", amount);
    Ok(())
}

fn reduce_short_position(ctx: Context<super::Rebalance>, amount: u64) -> Result<()> {
    // This would reduce short position on Drift
    msg!("Reducing short position by amount: {}", amount);
    Ok(())
}

// Events
#[event]
pub struct DepositEvent {
    pub user: Pubkey,
    pub amount: u64,
    pub shares_minted: u64,
    pub total_assets: u64,
    pub total_shares: u64,
}

#[event]
pub struct WithdrawEvent {
    pub user: Pubkey,
    pub shares_burned: u64,
    pub amount: u64,
    pub total_assets: u64,
    pub total_shares: u64,
}

#[event]
pub struct RebalanceEvent {
    pub delta_before: i64,
    pub action: HedgeAction,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct EmergencyStopEvent {
    pub admin: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct FeesCollectedEvent {
    pub amount: u64,
    pub total_fees_collected: u64,
    pub timestamp: i64,
}