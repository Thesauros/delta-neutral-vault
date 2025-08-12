use anchor_lang::prelude::*;

pub mod constants;
pub mod errors;
pub mod instructions;
pub mod state;
pub mod utils;

use instructions::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod delta_neutral_vault {
    use super::*;

    /// Initialize the delta neutral vault
    pub fn initialize_vault(
        ctx: Context<InitializeVault>,
        params: InitializeVaultParams,
    ) -> Result<()> {
        instructions::initialize_vault::handler(ctx, params)
    }

    /// Deposit funds into the vault
    pub fn deposit(
        ctx: Context<Deposit>,
        amount: u64,
    ) -> Result<()> {
        instructions::deposit::handler(ctx, amount)
    }

    /// Withdraw funds from the vault
    pub fn withdraw(
        ctx: Context<Withdraw>,
        amount: u64,
    ) -> Result<()> {
        instructions::withdraw::handler(ctx, amount)
    }

    /// Execute delta neutral strategy
    pub fn execute_strategy(
        ctx: Context<ExecuteStrategy>,
        strategy_params: StrategyParams,
    ) -> Result<()> {
        instructions::execute_strategy::handler(ctx, strategy_params)
    }

    /// Update vault parameters
    pub fn update_vault_params(
        ctx: Context<UpdateVaultParams>,
        params: UpdateVaultParamsParams,
    ) -> Result<()> {
        instructions::update_vault_params::handler(ctx, params)
    }

    /// Emergency pause the vault
    pub fn emergency_pause(
        ctx: Context<EmergencyPause>,
    ) -> Result<()> {
        instructions::emergency_pause::handler(ctx)
    }

    /// Resume vault operations
    pub fn resume_vault(
        ctx: Context<ResumeVault>,
    ) -> Result<()> {
        instructions::resume_vault::handler(ctx)
    }

    /// Collect fees from the vault
    pub fn collect_fees(
        ctx: Context<CollectFees>,
    ) -> Result<()> {
        instructions::collect_fees::handler(ctx)
    }

    /// Rebalance the vault positions
    pub fn rebalance(
        ctx: Context<Rebalance>,
        rebalance_params: RebalanceParams,
    ) -> Result<()> {
        instructions::rebalance::handler(ctx, rebalance_params)
    }
}
