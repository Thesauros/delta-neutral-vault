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

use errors::*;
use state::*;
use instructions::*;

#[program]
pub mod delta_neutral_vault {
    use super::*;

    pub fn initialize_vault(
        ctx: Context<InitializeVault>,
        target_leverage: u8,
        rebalance_threshold: u16, // basis points
        max_slippage: u16,        // basis points
    ) -> Result<()> {
        instructions::initialize_vault(ctx, target_leverage, rebalance_threshold, max_slippage)
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        instructions::deposit(ctx, amount)
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        instructions::withdraw(ctx, amount)
    }

    pub fn rebalance(ctx: Context<Rebalance>) -> Result<()> {
        instructions::rebalance(ctx)
    }

    pub fn emergency_stop(ctx: Context<EmergencyStop>) -> Result<()> {
        instructions::emergency_stop(ctx)
    }

    pub fn collect_fees(ctx: Context<CollectFees>) -> Result<()> {
        instructions::collect_fees(ctx)
    }

    pub fn update_vault_params(
        ctx: Context<UpdateVaultParams>,
        target_leverage: Option<u8>,
        rebalance_threshold: Option<u16>,
        max_slippage: Option<u16>,
    ) -> Result<()> {
        instructions::update_vault_params(ctx, target_leverage, rebalance_threshold, max_slippage)
    }
}

#[derive(Accounts)]
pub struct InitializeVault<'info> {
    #[account(
        init,
        payer = admin,
        space = VaultState::LEN,
        seeds = [b"vault", admin.key().as_ref()],
        bump
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        init,
        payer = admin,
        token::mint = token_mint,
        token::authority = vault_state,
        seeds = [b"vault_token_account", vault_state.key().as_ref()],
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
        seeds = [b"vault", vault_state.admin.as_ref()],
        bump = vault_state.bump
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        mut,
        seeds = [b"vault_token_account", vault_state.key().as_ref()],
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
        seeds = [b"vault", vault_state.admin.as_ref()],
        bump = vault_state.bump
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        mut,
        seeds = [b"vault_token_account", vault_state.key().as_ref()],
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

    #[account(mut)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct EmergencyStop<'info> {
    #[account(
        mut,
        seeds = [b"vault", vault_state.admin.as_ref()],
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
        seeds = [b"vault", vault_state.admin.as_ref()],
        bump = vault_state.bump
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        mut,
        seeds = [b"vault_token_account", vault_state.key().as_ref()],
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
        seeds = [b"vault", vault_state.admin.as_ref()],
        bump = vault_state.bump,
        has_one = admin
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(mut)]
    pub admin: Signer<'info>,
}