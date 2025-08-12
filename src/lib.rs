use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint};
use drift::cpi::accounts::{PlaceOrder, CancelOrder};
use drift::program::Drift;
use drift::state::{UserStats, User, Order, OrderType, OrderStatus, MarketType, PositionDirection};

declare_id!("DNVt1111111111111111111111111111111111111111");

pub mod errors;
pub mod state;
pub mod instructions;
pub mod utils;
pub mod events;
pub mod constants;

use errors::*;
use state::*;
use instructions::*;
use events::*;
use constants::*;

#[program]
pub mod delta_neutral_vault {
    use super::*;

    /// Initialize a new delta-neutral vault
    /// 
    /// # Arguments
    /// * `target_leverage` - Target leverage ratio (1-10x)
    /// * `rebalance_threshold` - Threshold to trigger rebalancing (basis points)
    /// * `max_slippage` - Maximum allowed slippage (basis points)
    pub fn initialize_vault(
        ctx: Context<InitializeVault>,
        target_leverage: u8,
        rebalance_threshold: u16,
        max_slippage: u16,
    ) -> Result<()> {
        instructions::initialize_vault(ctx, target_leverage, rebalance_threshold, max_slippage)
    }

    /// Deposit funds into the vault
    /// 
    /// # Arguments
    /// * `amount` - Amount to deposit (in token units)
    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        instructions::deposit(ctx, amount)
    }

    /// Withdraw funds from the vault
    /// 
    /// # Arguments
    /// * `amount` - Amount to withdraw (in token units)
    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        instructions::withdraw(ctx, amount)
    }

    /// Rebalance the vault's delta-neutral positions
    pub fn rebalance(ctx: Context<Rebalance>) -> Result<()> {
        instructions::rebalance(ctx)
    }

    /// Emergency stop the vault (admin only)
    pub fn emergency_stop(ctx: Context<EmergencyStop>) -> Result<()> {
        instructions::emergency_stop(ctx)
    }

    /// Collect fees from the vault
    pub fn collect_fees(ctx: Context<CollectFees>) -> Result<()> {
        instructions::collect_fees(ctx)
    }

    /// Update vault parameters (admin only)
    /// 
    /// # Arguments
    /// * `target_leverage` - New target leverage ratio
    /// * `rebalance_threshold` - New rebalance threshold
    /// * `max_slippage` - New maximum slippage
    pub fn update_vault_params(
        ctx: Context<UpdateVaultParams>,
        target_leverage: Option<u8>,
        rebalance_threshold: Option<u16>,
        max_slippage: Option<u16>,
    ) -> Result<()> {
        instructions::update_vault_params(ctx, target_leverage, rebalance_threshold, max_slippage)
    }

    /// Open a new position on Drift Protocol
    /// 
    /// # Arguments
    /// * `market_index` - Market index for the position
    /// * `size` - Position size
    /// * `direction` - Position direction (0 = long, 1 = short)
    pub fn open_position(
        ctx: Context<OpenPosition>,
        market_index: u16,
        size: u64,
        direction: u8,
    ) -> Result<()> {
        instructions::open_position(ctx, market_index, size, direction)
    }

    /// Close an existing position on Drift Protocol
    /// 
    /// # Arguments
    /// * `market_index` - Market index for the position
    /// * `size` - Position size to close
    pub fn close_position(
        ctx: Context<ClosePosition>,
        market_index: u16,
        size: u64,
    ) -> Result<()> {
        instructions::close_position(ctx, market_index, size)
    }
}

#[derive(Accounts)]
pub struct InitializeVault<'info> {
    #[account(
        init,
        payer = admin,
        space = VaultState::LEN,
        seeds = [VAULT_SEED, admin.key().as_ref()],
        bump
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        init,
        payer = admin,
        token::mint = token_mint,
        token::authority = vault_state,
        seeds = [VAULT_TOKEN_ACCOUNT_SEED, vault_state.key().as_ref()],
        bump
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub token_mint: Account<'info, Mint>,
    pub drift_program: Program<'info, Drift>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(
        mut,
        seeds = [VAULT_SEED, vault_state.admin.as_ref()],
        bump = vault_state.bump
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        mut,
        seeds = [VAULT_TOKEN_ACCOUNT_SEED, vault_state.key().as_ref()],
        bump
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        token::mint = vault_token_account.mint,
        token::authority = depositor
    )]
    pub depositor_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub depositor: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(
        mut,
        seeds = [VAULT_SEED, vault_state.admin.as_ref()],
        bump = vault_state.bump
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        mut,
        seeds = [VAULT_TOKEN_ACCOUNT_SEED, vault_state.key().as_ref()],
        bump
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        token::mint = vault_token_account.mint,
        token::authority = withdrawer
    )]
    pub withdrawer_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub withdrawer: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Rebalance<'info> {
    #[account(
        mut,
        seeds = [VAULT_SEED, vault_state.admin.as_ref()],
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

    #[account(mut)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct EmergencyStop<'info> {
    #[account(
        mut,
        seeds = [VAULT_SEED, vault_state.admin.as_ref()],
        bump = vault_state.bump,
        has_one = admin
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(mut)]
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct CollectFees<'info> {
    #[account(
        mut,
        seeds = [VAULT_SEED, vault_state.admin.as_ref()],
        bump = vault_state.bump
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        mut,
        seeds = [VAULT_TOKEN_ACCOUNT_SEED, vault_state.key().as_ref()],
        bump
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        token::mint = vault_token_account.mint,
        token::authority = fee_collector
    )]
    pub fee_collector_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub fee_collector: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct UpdateVaultParams<'info> {
    #[account(
        mut,
        seeds = [VAULT_SEED, vault_state.admin.as_ref()],
        bump = vault_state.bump,
        has_one = admin
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(mut)]
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct OpenPosition<'info> {
    #[account(
        mut,
        seeds = [VAULT_SEED, vault_state.admin.as_ref()],
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

    #[account(mut)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct ClosePosition<'info> {
    #[account(
        mut,
        seeds = [VAULT_SEED, vault_state.admin.as_ref()],
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

    #[account(mut)]
    pub authority: Signer<'info>,
}