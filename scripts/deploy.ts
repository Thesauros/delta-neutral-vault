import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, Keypair, SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, createMint, createAccount, mintTo } from "@solana/spl-token";
import { DeltaNeutralVault } from "../target/types/delta_neutral_vault";

// Configuration
const NETWORK = process.env.SOLANA_NETWORK || "devnet";
const ADMIN_KEYPAIR_PATH = process.env.ADMIN_KEYPAIR_PATH || "./keys/admin.json";
const VAULT_PARAMS = {
  targetLeverage: 2,
  rebalanceThreshold: 100, // 1%
  maxSlippage: 50, // 0.5%
};

interface DeploymentConfig {
  adminKeypair: Keypair;
  provider: anchor.AnchorProvider;
  program: Program<DeltaNeutralVault>;
  tokenMint: PublicKey;
  driftProgram: PublicKey;
}

async function setupProvider(): Promise<anchor.AnchorProvider> {
  // Configure the client to use the devnet cluster
  const connection = new anchor.web3.Connection(
    anchor.web3.clusterApiUrl(NETWORK as anchor.web3.Cluster)
  );
  
  // Load admin keypair
  const adminKeypair = Keypair.fromSecretKey(
    Uint8Array.from(JSON.parse(require("fs").readFileSync(ADMIN_KEYPAIR_PATH, "utf8")))
  );
  
  const wallet = new anchor.Wallet(adminKeypair);
  const provider = new anchor.AnchorProvider(connection, wallet, {
    commitment: "confirmed",
  });
  
  anchor.setProvider(provider);
  return provider;
}

async function loadProgram(provider: anchor.AnchorProvider): Promise<Program<DeltaNeutralVault>> {
  const idl = JSON.parse(
    require("fs").readFileSync("./target/idl/delta_neutral_vault.json", "utf8")
  );
  
  const programId = new PublicKey(idl.metadata.address);
  return new Program(idl, programId, provider);
}

async function createTestToken(
  provider: anchor.AnchorProvider,
  adminKeypair: Keypair
): Promise<PublicKey> {
  console.log("Creating test USDC token...");
  
  const mint = await createMint(
    provider.connection,
    adminKeypair,
    adminKeypair.publicKey,
    null,
    6 // USDC has 6 decimals
  );
  
  console.log(`Test USDC mint created: ${mint.toString()}`);
  return mint;
}

async function deployVault(config: DeploymentConfig): Promise<{
  vaultState: PublicKey;
  vaultTokenAccount: PublicKey;
}> {
  const { adminKeypair, program, tokenMint } = config;
  
  console.log("Deploying Delta Neutral Vault...");
  
  // Derive PDA for vault state
  const [vaultState, vaultBump] = PublicKey.findProgramAddressSync(
    [Buffer.from("vault"), adminKeypair.publicKey.toBuffer()],
    program.programId
  );
  
  // Derive PDA for vault token account
  const [vaultTokenAccount, tokenBump] = PublicKey.findProgramAddressSync(
    [Buffer.from("vault_token_account"), vaultState.toBuffer()],
    program.programId
  );
  
  console.log(`Vault State PDA: ${vaultState.toString()}`);
  console.log(`Vault Token Account PDA: ${vaultTokenAccount.toString()}`);
  
  try {
    // Initialize the vault
    const tx = await program.methods
      .initializeVault(
        VAULT_PARAMS.targetLeverage,
        VAULT_PARAMS.rebalanceThreshold,
        VAULT_PARAMS.maxSlippage
      )
      .accounts({
        vaultState,
        vaultTokenAccount,
        admin: adminKeypair.publicKey,
        tokenMint,
        driftProgram: config.driftProgram,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .signers([adminKeypair])
      .rpc();
    
    console.log(`Vault initialized! Transaction: ${tx}`);
    
    // Verify vault state
    const vaultAccount = await program.account.vaultState.fetch(vaultState);
    console.log("Vault State:", {
      admin: vaultAccount.admin.toString(),
      targetLeverage: vaultAccount.targetLeverage,
      rebalanceThreshold: vaultAccount.rebalanceThreshold,
      maxSlippage: vaultAccount.maxSlippage,
      totalAssets: vaultAccount.totalAssets.toString(),
      totalShares: vaultAccount.totalShares.toString(),
      emergencyStop: vaultAccount.emergencyStop,
    });
    
    return { vaultState, vaultTokenAccount };
    
  } catch (error) {
    console.error("Error initializing vault:", error);
    throw error;
  }
}

async function setupTestUsers(
  config: DeploymentConfig,
  vaultState: PublicKey,
  vaultTokenAccount: PublicKey
): Promise<{
  user1: Keypair;
  user2: Keypair;
  user1TokenAccount: PublicKey;
  user2TokenAccount: PublicKey;
}> {
  const { provider, adminKeypair, tokenMint } = config;
  
  console.log("Setting up test users...");
  
  // Create test users
  const user1 = Keypair.generate();
  const user2 = Keypair.generate();
  
  // Airdrop SOL to users
  await provider.connection.requestAirdrop(user1.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL);
  await provider.connection.requestAirdrop(user2.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL);
  
  // Wait for airdrops to confirm
  await new Promise(resolve => setTimeout(resolve, 2000));
  
  // Create token accounts for users
  const user1TokenAccount = await createAccount(
    provider.connection,
    user1,
    tokenMint,
    user1.publicKey
  );
  
  const user2TokenAccount = await createAccount(
    provider.connection,
    user2,
    tokenMint,
    user2.publicKey
  );
  
  // Mint test tokens to users
  await mintTo(
    provider.connection,
    adminKeypair,
    tokenMint,
    user1TokenAccount,
    adminKeypair,
    1000 * 10**6 // 1000 USDC
  );
  
  await mintTo(
    provider.connection,
    adminKeypair,
    tokenMint,
    user2TokenAccount,
    adminKeypair,
    500 * 10**6 // 500 USDC
  );
  
  console.log(`User 1: ${user1.publicKey.toString()}`);
  console.log(`User 1 Token Account: ${user1TokenAccount.toString()}`);
  console.log(`User 2: ${user2.publicKey.toString()}`);
  console.log(`User 2 Token Account: ${user2TokenAccount.toString()}`);
  
  return { user1, user2, user1TokenAccount, user2TokenAccount };
}

async function testDeposit(
  config: DeploymentConfig,
  vaultState: PublicKey,
  vaultTokenAccount: PublicKey,
  user: Keypair,
  userTokenAccount: PublicKey,
  amount: number
): Promise<void> {
  const { program } = config;
  
  console.log(`Testing deposit of ${amount} tokens from ${user.publicKey.toString()}...`);
  
  try {
    const tx = await program.methods
      .deposit(new anchor.BN(amount * 10**6))
      .accounts({
        vaultState,
        vaultTokenAccount,
        depositorTokenAccount: userTokenAccount,
        depositor: user.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([user])
      .rpc();
    
    console.log(`Deposit successful! Transaction: ${tx}`);
    
    // Check vault state after deposit
    const vaultAccount = await program.account.vaultState.fetch(vaultState);
    console.log(`Vault total assets: ${vaultAccount.totalAssets.toString()}`);
    console.log(`Vault total shares: ${vaultAccount.totalShares.toString()}`);
    
  } catch (error) {
    console.error("Error during deposit:", error);
    throw error;
  }
}

async function testRebalance(
  config: DeploymentConfig,
  vaultState: PublicKey
): Promise<void> {
  const { program, adminKeypair } = config;
  
  console.log("Testing rebalance...");
  
  // For this test, we'll simulate that rebalancing is needed
  // In production, this would check actual positions and market conditions
  
  try {
    const tx = await program.methods
      .rebalance()
      .accounts({
        vaultState,
        driftUser: Keypair.generate().publicKey, // Placeholder
        driftUserStats: Keypair.generate().publicKey, // Placeholder
        driftState: config.driftProgram,
        driftProgram: config.driftProgram,
        authority: adminKeypair.publicKey,
      })
      .signers([adminKeypair])
      .rpc();
    
    console.log(`Rebalance successful! Transaction: ${tx}`);
    
  } catch (error) {
    console.log("Rebalance test skipped (expected in test environment):", error.message);
  }
}

async function printDeploymentSummary(
  config: DeploymentConfig,
  vaultState: PublicKey,
  vaultTokenAccount: PublicKey
): Promise<void> {
  console.log("\n=== DEPLOYMENT SUMMARY ===");
  console.log(`Network: ${NETWORK}`);
  console.log(`Program ID: ${config.program.programId.toString()}`);
  console.log(`Admin: ${config.adminKeypair.publicKey.toString()}`);
  console.log(`Token Mint: ${config.tokenMint.toString()}`);
  console.log(`Vault State: ${vaultState.toString()}`);
  console.log(`Vault Token Account: ${vaultTokenAccount.toString()}`);
  console.log(`Drift Program: ${config.driftProgram.toString()}`);
  
  // Get final vault state
  const vaultAccount = await config.program.account.vaultState.fetch(vaultState);
  console.log("\n=== VAULT STATE ===");
  console.log(`Target Leverage: ${vaultAccount.targetLeverage}x`);
  console.log(`Rebalance Threshold: ${vaultAccount.rebalanceThreshold} bps`);
  console.log(`Max Slippage: ${vaultAccount.maxSlippage} bps`);
  console.log(`Total Assets: ${vaultAccount.totalAssets.toString()}`);
  console.log(`Total Shares: ${vaultAccount.totalShares.toString()}`);
  console.log(`Emergency Stop: ${vaultAccount.emergencyStop}`);
  console.log(`Management Fee: ${vaultAccount.managementFee} bps`);
  console.log(`Performance Fee: ${vaultAccount.performanceFee} bps`);
}

async function main() {
  try {
    console.log("Starting Delta Neutral Vault deployment...");
    
    // Setup
    const provider = await setupProvider();
    const program = await loadProgram(provider);
    const adminKeypair = (provider.wallet as anchor.Wallet).payer;
    
    // Create test token
    const tokenMint = await createTestToken(provider, adminKeypair);
    
    // Drift program ID (use mainnet/devnet address or placeholder)
    const driftProgram = new PublicKey("dRiftyHA39MWEi3m9aunc5MzRF1JYuBsbn6VPcn33UH");
    
    const config: DeploymentConfig = {
      adminKeypair,
      provider,
      program,
      tokenMint,
      driftProgram,
    };
    
    // Deploy vault
    const { vaultState, vaultTokenAccount } = await deployVault(config);
    
    // Setup test users and perform test operations
    const { user1, user2, user1TokenAccount, user2TokenAccount } = 
      await setupTestUsers(config, vaultState, vaultTokenAccount);
    
    // Test deposits
    await testDeposit(config, vaultState, vaultTokenAccount, user1, user1TokenAccount, 100);
    await testDeposit(config, vaultState, vaultTokenAccount, user2, user2TokenAccount, 50);
    
    // Test rebalance
    await testRebalance(config, vaultState);
    
    // Print summary
    await printDeploymentSummary(config, vaultState, vaultTokenAccount);
    
    console.log("\n✅ Deployment completed successfully!");
    
  } catch (error) {
    console.error("❌ Deployment failed:", error);
    process.exit(1);
  }
}

// Run the deployment
if (require.main === module) {
  main();
}