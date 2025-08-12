use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use solana_program_test::*;
use solana_sdk::{
    account::Account,
    signature::{Keypair, Signer},
    transaction::Transaction,
};

use delta_neutral_vault::{
    instruction::*,
    state::*,
    error::VaultError,
};

pub struct VaultTestFixture {
    pub vault_state: Keypair,
    pub admin: Keypair,
    pub token_mint: Keypair,
    pub vault_token_account: Pubkey,
    pub user1: Keypair,
    pub user1_token_account: Pubkey,
    pub user2: Keypair,
    pub user2_token_account: Pubkey,
}

impl VaultTestFixture {
    pub async fn new(context: &mut ProgramTestContext) -> Self {
        let admin = Keypair::new();
        let vault_state = Keypair::new();
        let token_mint = Keypair::new();
        let user1 = Keypair::new();
        let user2 = Keypair::new();

        // Airdrop SOL to accounts
        let rent = context.banks_client.get_rent().await.unwrap();
        let lamports = rent.minimum_balance(Mint::LEN);
        
        let mut transaction = Transaction::new_with_payer(
            &[
                solana_sdk::system_instruction::transfer(
                    &context.payer.pubkey(),
                    &admin.pubkey(),
                    1_000_000_000,
                ),
                solana_sdk::system_instruction::transfer(
                    &context.payer.pubkey(),
                    &user1.pubkey(),
                    1_000_000_000,
                ),
                solana_sdk::system_instruction::transfer(
                    &context.payer.pubkey(),
                    &user2.pubkey(),
                    1_000_000_000,
                ),
            ],
            Some(&context.payer.pubkey()),
        );
        
        transaction.sign(
            &[&context.payer, &admin, &user1, &user2],
            context.last_blockhash,
        );
        
        context.banks_client.process_transaction(transaction).await.unwrap();

        // Create token mint
        let create_mint_ix = anchor_spl::token::instruction::initialize_mint(
            &anchor_spl::token::ID,
            &token_mint.pubkey(),
            &admin.pubkey(),
            None,
            6,
        ).unwrap();

        // Create token accounts
        let vault_token_account = anchor_spl::associated_token::get_associated_token_address(
            &vault_state.pubkey(),
            &token_mint.pubkey(),
        );

        let user1_token_account = anchor_spl::associated_token::get_associated_token_address(
            &user1.pubkey(),
            &token_mint.pubkey(),
        );

        let user2_token_account = anchor_spl::associated_token::get_associated_token_address(
            &user2.pubkey(),
            &token_mint.pubkey(),
        );

        Self {
            vault_state,
            admin,
            token_mint,
            vault_token_account,
            user1,
            user1_token_account,
            user2,
            user2_token_account,
        }
    }

    pub async fn initialize_vault(
        &self,
        context: &mut ProgramTestContext,
        target_leverage: u8,
        rebalance_threshold: u16,
        max_slippage: u16,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let ix = initialize_vault(
            &delta_neutral_vault::ID,
            &self.admin.pubkey(),
            &self.token_mint.pubkey(),
            target_leverage,
            rebalance_threshold,
            max_slippage,
        );

        let mut transaction = Transaction::new_with_payer(&[ix], Some(&self.admin.pubkey()));
        transaction.sign(&[&self.admin], context.last_blockhash);
        context.banks_client.process_transaction(transaction).await?;

        Ok(())
    }

    pub async fn deposit(
        &self,
        context: &mut ProgramTestContext,
        user: &Keypair,
        user_token_account: &Pubkey,
        amount: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let ix = deposit(
            &delta_neutral_vault::ID,
            &self.vault_state.pubkey(),
            &user.pubkey(),
            user_token_account,
            &self.vault_token_account,
            amount,
        );

        let mut transaction = Transaction::new_with_payer(&[ix], Some(&user.pubkey()));
        transaction.sign(&[user], context.last_blockhash);
        context.banks_client.process_transaction(transaction).await?;

        Ok(())
    }
}

#[tokio::test]
async fn test_initialize_vault() {
    let program = ProgramTest::new(
        "delta_neutral_vault",
        delta_neutral_vault::ID,
        processor!(delta_neutral_vault::entry),
    );
    let (mut banks_client, payer, recent_blockhash) = program.start().await;
    let mut context = ProgramTestContext {
        banks_client,
        payer,
        last_blockhash: recent_blockhash,
    };

    let fixture = VaultTestFixture::new(&mut context).await;

    // Test successful initialization
    fixture
        .initialize_vault(&mut context, 2, 100, 50)
        .await
        .expect("Failed to initialize vault");

    // Verify vault state
    let vault_account = context
        .banks_client
        .get_account(fixture.vault_state.pubkey())
        .await
        .expect("Failed to get vault account")
        .expect("Vault account not found");

    let vault_state: VaultState = VaultState::try_deserialize(&mut vault_account.data.as_slice())
        .expect("Failed to deserialize vault state");

    assert_eq!(vault_state.admin, fixture.admin.pubkey());
    assert_eq!(vault_state.target_leverage, 2);
    assert_eq!(vault_state.rebalance_threshold, 100);
    assert_eq!(vault_state.max_slippage, 50);
    assert_eq!(vault_state.total_assets, 0);
    assert_eq!(vault_state.total_shares, 0);
    assert_eq!(vault_state.emergency_stop, false);
}

#[tokio::test]
async fn test_deposit_withdraw() {
    let program = ProgramTest::new(
        "delta_neutral_vault",
        delta_neutral_vault::ID,
        processor!(delta_neutral_vault::entry),
    );
    let (mut banks_client, payer, recent_blockhash) = program.start().await;
    let mut context = ProgramTestContext {
        banks_client,
        payer,
        last_blockhash: recent_blockhash,
    };

    let fixture = VaultTestFixture::new(&mut context).await;

    // Initialize vault
    fixture
        .initialize_vault(&mut context, 2, 100, 50)
        .await
        .expect("Failed to initialize vault");

    // Mint tokens to user1
    let mint_to_user_ix = anchor_spl::token::instruction::mint_to(
        &anchor_spl::token::ID,
        &fixture.token_mint.pubkey(),
        &fixture.user1_token_account,
        &fixture.admin.pubkey(),
        &[],
        1_000_000_000,
    ).unwrap();

    let mut transaction = Transaction::new_with_payer(
        &[mint_to_user_ix],
        Some(&fixture.admin.pubkey()),
    );
    transaction.sign(&[&fixture.admin], context.last_blockhash);
    context.banks_client.process_transaction(transaction).await.unwrap();

    // Test deposit
    let deposit_amount = 100_000_000; // 100 tokens
    fixture
        .deposit(&mut context, &fixture.user1, &fixture.user1_token_account, deposit_amount)
        .await
        .expect("Failed to deposit");

    // Verify vault state after deposit
    let vault_account = context
        .banks_client
        .get_account(fixture.vault_state.pubkey())
        .await
        .expect("Failed to get vault account")
        .expect("Vault account not found");

    let vault_state: VaultState = VaultState::try_deserialize(&mut vault_account.data.as_slice())
        .expect("Failed to deserialize vault state");

    assert_eq!(vault_state.total_assets, deposit_amount);
    assert_eq!(vault_state.total_shares, deposit_amount); // 1:1 for first deposit
    assert_eq!(vault_state.net_deposits, deposit_amount as i64);

    // Test second deposit with different user
    let mint_to_user2_ix = anchor_spl::token::instruction::mint_to(
        &anchor_spl::token::ID,
        &fixture.token_mint.pubkey(),
        &fixture.user2_token_account,
        &fixture.admin.pubkey(),
        &[],
        500_000_000,
    ).unwrap();

    let mut transaction = Transaction::new_with_payer(
        &[mint_to_user2_ix],
        Some(&fixture.admin.pubkey()),
    );
    transaction.sign(&[&fixture.admin], context.last_blockhash);
    context.banks_client.process_transaction(transaction).await.unwrap();

    let second_deposit = 200_000_000; // 200 tokens
    fixture
        .deposit(&mut context, &fixture.user2, &fixture.user2_token_account, second_deposit)
        .await
        .expect("Failed to make second deposit");

    // Verify vault state after second deposit
    let vault_account = context
        .banks_client
        .get_account(fixture.vault_state.pubkey())
        .await
        .expect("Failed to get vault account")
        .expect("Vault account not found");

    let vault_state: VaultState = VaultState::try_deserialize(&mut vault_account.data.as_slice())
        .expect("Failed to deserialize vault state");

    assert_eq!(vault_state.total_assets, deposit_amount + second_deposit);
    assert_eq!(vault_state.total_shares, deposit_amount + second_deposit); // Still 1:1 since no profits
}

#[tokio::test]
async fn test_rebalance_logic() {
    let program = ProgramTest::new(
        "delta_neutral_vault",
        delta_neutral_vault::ID,
        processor!(delta_neutral_vault::entry),
    );
    let (mut banks_client, payer, recent_blockhash) = program.start().await;
    let mut context = ProgramTestContext {
        banks_client,
        payer,
        last_blockhash: recent_blockhash,
    };

    let fixture = VaultTestFixture::new(&mut context).await;

    // Initialize vault with tight rebalance threshold
    fixture
        .initialize_vault(&mut context, 2, 50, 25) // 0.5% threshold
        .await
        .expect("Failed to initialize vault");

    // Simulate positions that require rebalancing
    let vault_account = context
        .banks_client
        .get_account(fixture.vault_state.pubkey())
        .await
        .expect("Failed to get vault account")
        .expect("Vault account not found");

    let mut vault_state: VaultState = VaultState::try_deserialize(&mut vault_account.data.as_slice())
        .expect("Failed to deserialize vault state");

    // Set up test scenario with imbalanced positions
    vault_state.total_assets = 1_000_000_000; // 1000 tokens
    vault_state.long_position = 500_000; // Long position
    vault_state.short_position = -300_000; // Short position
    // Net delta = 200_000 (0.02% of total assets, should trigger rebalance)

    // Test rebalance calculation
    let needs_rebalance = vault_state.needs_rebalance().expect("Failed to check rebalance");
    assert!(needs_rebalance, "Vault should need rebalancing");

    let hedge_calc = vault_state.calculate_required_hedge().expect("Failed to calculate hedge");
    assert_eq!(hedge_calc.action, HedgeAction::IncreaseShort);
    assert_eq!(hedge_calc.amount, 200_000);
    assert_eq!(hedge_calc.direction, PositionDirection::Short);
}

#[tokio::test]
async fn test_emergency_stop() {
    let program = ProgramTest::new(
        "delta_neutral_vault",
        delta_neutral_vault::ID,
        processor!(delta_neutral_vault::entry),
    );
    let (mut banks_client, payer, recent_blockhash) = program.start().await;
    let mut context = ProgramTestContext {
        banks_client,
        payer,
        last_blockhash: recent_blockhash,
    };

    let fixture = VaultTestFixture::new(&mut context).await;

    // Initialize vault
    fixture
        .initialize_vault(&mut context, 2, 100, 50)
        .await
        .expect("Failed to initialize vault");

    // Test emergency stop
    let emergency_stop_ix = emergency_stop(
        &delta_neutral_vault::ID,
        &fixture.vault_state.pubkey(),
        &fixture.admin.pubkey(),
    );

    let mut transaction = Transaction::new_with_payer(
        &[emergency_stop_ix],
        Some(&fixture.admin.pubkey()),
    );
    transaction.sign(&[&fixture.admin], context.last_blockhash);
    context.banks_client.process_transaction(transaction).await.unwrap();

    // Verify emergency stop is active
    let vault_account = context
        .banks_client
        .get_account(fixture.vault_state.pubkey())
        .await
        .expect("Failed to get vault account")
        .expect("Vault account not found");

    let vault_state: VaultState = VaultState::try_deserialize(&mut vault_account.data.as_slice())
        .expect("Failed to deserialize vault state");

    assert!(vault_state.emergency_stop, "Emergency stop should be active");

    // Test that deposits are blocked during emergency stop
    let mint_to_user_ix = anchor_spl::token::instruction::mint_to(
        &anchor_spl::token::ID,
        &fixture.token_mint.pubkey(),
        &fixture.user1_token_account,
        &fixture.admin.pubkey(),
        &[],
        1_000_000_000,
    ).unwrap();

    let mut transaction = Transaction::new_with_payer(
        &[mint_to_user_ix],
        Some(&fixture.admin.pubkey()),
    );
    transaction.sign(&[&fixture.admin], context.last_blockhash);
    context.banks_client.process_transaction(transaction).await.unwrap();

    // This should fail due to emergency stop
    let result = fixture
        .deposit(&mut context, &fixture.user1, &fixture.user1_token_account, 100_000_000)
        .await;

    assert!(result.is_err(), "Deposit should fail during emergency stop");
}

#[tokio::test]
async fn test_fee_calculation() {
    use delta_neutral_vault::utils::*;

    // Test management fee calculation
    let total_assets = 1_000_000_000; // 1000 tokens
    let fee_rate_bps = 200; // 2% annual
    let time_elapsed = 365 * 24 * 3600; // 1 year

    let fees = calculate_management_fees(total_assets, fee_rate_bps, time_elapsed).unwrap();
    assert_eq!(fees, 20_000_000); // 2% of 1000 = 20 tokens

    // Test performance fee calculation
    let total_value = 1_200_000_000; // 1200 tokens
    let net_deposits = 1_000_000_000i64; // 1000 tokens deposited
    let performance_fee_bps = 1000; // 10%

    let performance_fees = calculate_performance_fees(total_value, net_deposits, performance_fee_bps).unwrap();
    assert_eq!(performance_fees, 20_000_000); // 10% of 200 profit = 20 tokens

    // Test share price calculation
    let share_price = calculate_share_price(total_value, 1_000_000_000).unwrap();
    assert_eq!(share_price, 1_200_000); // 1.2 tokens per share
}

#[tokio::test]
async fn test_delta_calculation() {
    use delta_neutral_vault::utils::*;

    // Test delta percentage calculation
    let long_position = 1_000_000i64;
    let short_position = -800_000i64;
    let total_value = 10_000_000u64;

    let delta_percentage = calculate_delta_percentage(long_position, short_position, total_value).unwrap();
    assert_eq!(delta_percentage, 200); // 2% delta

    // Test rebalance decision
    let should_rebalance = should_rebalance(long_position, short_position, total_value, 100).unwrap();
    assert!(should_rebalance, "Should trigger rebalance with 2% delta and 1% threshold");

    let should_not_rebalance = should_rebalance(long_position, short_position, total_value, 300).unwrap();
    assert!(!should_not_rebalance, "Should not trigger rebalance with 2% delta and 3% threshold");
}

// Helper functions for creating instructions
fn initialize_vault(
    program_id: &Pubkey,
    admin: &Pubkey,
    token_mint: &Pubkey,
    target_leverage: u8,
    rebalance_threshold: u16,
    max_slippage: u16,
) -> solana_sdk::instruction::Instruction {
    // This would be the actual instruction creation
    // For now, returning a placeholder
    solana_sdk::instruction::Instruction {
        program_id: *program_id,
        accounts: vec![],
        data: vec![],
    }
}

fn deposit(
    program_id: &Pubkey,
    vault_state: &Pubkey,
    user: &Pubkey,
    user_token_account: &Pubkey,
    vault_token_account: &Pubkey,
    amount: u64,
) -> solana_sdk::instruction::Instruction {
    // This would be the actual instruction creation
    // For now, returning a placeholder
    solana_sdk::instruction::Instruction {
        program_id: *program_id,
        accounts: vec![],
        data: vec![],
    }
}

fn emergency_stop(
    program_id: &Pubkey,
    vault_state: &Pubkey,
    admin: &Pubkey,
) -> solana_sdk::instruction::Instruction {
    // This would be the actual instruction creation
    // For now, returning a placeholder
    solana_sdk::instruction::Instruction {
        program_id: *program_id,
        accounts: vec![],
        data: vec![],
    }
}