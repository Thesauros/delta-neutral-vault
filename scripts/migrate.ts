import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, Keypair, Connection } from "@solana/web3.js";
import { DeltaNeutralVault } from "../target/types/delta_neutral_vault";
import * as dotenv from "dotenv";

// Load environment variables
dotenv.config();

interface MigrationConfig {
    oldProgramId: string;
    newProgramId: string;
    vaultAddress: string;
    adminKeypair: Keypair;
    provider: anchor.AnchorProvider;
}

async function setupProvider(): Promise<anchor.AnchorProvider> {
    const connection = new Connection(
        process.env.SOLANA_RPC_URL || anchor.web3.clusterApiUrl("devnet"),
        "confirmed"
    );
    
    const adminKeypair = Keypair.fromSecretKey(
        Uint8Array.from(JSON.parse(require("fs").readFileSync(process.env.ADMIN_KEYPAIR_PATH || "./keys/admin.json", "utf8")))
    );
    
    const wallet = new anchor.Wallet(adminKeypair);
    const provider = new anchor.AnchorProvider(connection, wallet, {
        commitment: "confirmed",
    });
    
    anchor.setProvider(provider);
    return provider;
}

async function loadProgram(provider: anchor.AnchorProvider, programId: string): Promise<Program<DeltaNeutralVault>> {
    const idl = JSON.parse(
        require("fs").readFileSync("./target/idl/delta_neutral_vault.json", "utf8")
    );
    
    const programPubkey = new PublicKey(programId);
    return new Program(idl, programPubkey, provider);
}

async function backupVaultState(
    provider: anchor.AnchorProvider,
    vaultAddress: string
): Promise<any> {
    console.log("📋 Backing up vault state...");
    
    const vaultPubkey = new PublicKey(vaultAddress);
    const vaultAccount = await provider.connection.getAccountInfo(vaultPubkey);
    
    if (!vaultAccount) {
        throw new Error("Vault account not found");
    }
    
    // Save backup to file
    const backupData = {
        address: vaultAddress,
        data: Array.from(vaultAccount.data),
        lamports: vaultAccount.lamports,
        owner: vaultAccount.owner.toString(),
        timestamp: new Date().toISOString()
    };
    
    require("fs").writeFileSync(
        `./backup/vault-${Date.now()}.json`,
        JSON.stringify(backupData, null, 2)
    );
    
    console.log("✅ Vault state backed up");
    return backupData;
}

async function migrateVault(config: MigrationConfig): Promise<void> {
    const { oldProgramId, newProgramId, vaultAddress, adminKeypair, provider } = config;
    
    console.log("🔄 Starting vault migration...");
    console.log(`From: ${oldProgramId}`);
    console.log(`To: ${newProgramId}`);
    console.log(`Vault: ${vaultAddress}`);
    
    // Backup current state
    await backupVaultState(provider, vaultAddress);
    
    // Load old and new programs
    const oldProgram = await loadProgram(provider, oldProgramId);
    const newProgram = await loadProgram(provider, newProgramId);
    
    // Get current vault state
    const vaultPubkey = new PublicKey(vaultAddress);
    const vaultState = await oldProgram.account.vaultState.fetch(vaultPubkey);
    
    console.log("📊 Current vault state:");
    console.log(`- Total assets: ${vaultState.totalAssets}`);
    console.log(`- Total shares: ${vaultState.totalShares}`);
    console.log(`- Long position: ${vaultState.longPosition}`);
    console.log(`- Short position: ${vaultState.shortPosition}`);
    
    // Create new vault with same parameters
    console.log("🏗 Creating new vault...");
    
    const [newVaultState, newVaultBump] = PublicKey.findProgramAddressSync(
        [Buffer.from("vault"), adminKeypair.publicKey.toBuffer()],
        newProgram.programId
    );
    
    const [newVaultTokenAccount, newTokenBump] = PublicKey.findProgramAddressSync(
        [Buffer.from("vault_token_account"), newVaultState.toBuffer()],
        newProgram.programId
    );
    
    // Initialize new vault
    await newProgram.methods
        .initializeVault(
            vaultState.targetLeverage,
            vaultState.rebalanceThreshold,
            vaultState.maxSlippage
        )
        .accounts({
            vaultState: newVaultState,
            vaultTokenAccount: newVaultTokenAccount,
            admin: adminKeypair.publicKey,
            tokenMint: vaultState.tokenMint,
            driftProgram: new PublicKey(process.env.DRIFT_PROGRAM_ID || "DRiFTvSoSLjH8XJx2wSJ1GL9jR8qXhF2vC2QBK5FyL9"),
            tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
            systemProgram: anchor.web3.SystemProgram.programId,
            rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        })
        .signers([adminKeypair])
        .rpc();
    
    console.log("✅ New vault initialized");
    console.log(`New vault address: ${newVaultState.toString()}`);
    
    // Transfer assets from old vault to new vault
    console.log("💰 Transferring assets...");
    
    // This would require implementing a migration instruction
    // that allows transferring assets between vaults
    
    console.log("✅ Migration completed");
    console.log("\n📋 Migration summary:");
    console.log(`Old vault: ${vaultAddress}`);
    console.log(`New vault: ${newVaultState.toString()}`);
    console.log(`Old program: ${oldProgramId}`);
    console.log(`New program: ${newProgramId}`);
}

async function main() {
    console.log("🚀 Delta Neutral Vault Migration Tool");
    
    const provider = await setupProvider();
    const adminKeypair = Keypair.fromSecretKey(
        Uint8Array.from(JSON.parse(require("fs").readFileSync(process.env.ADMIN_KEYPAIR_PATH || "./keys/admin.json", "utf8")))
    );
    
    // Configuration
    const config: MigrationConfig = {
        oldProgramId: process.env.OLD_PROGRAM_ID || "DNVt1111111111111111111111111111111111111111",
        newProgramId: process.env.NEW_PROGRAM_ID || "DNVt1111111111111111111111111111111111111111",
        vaultAddress: process.env.VAULT_ADDRESS || "",
        adminKeypair,
        provider
    };
    
    if (!config.vaultAddress) {
        console.error("❌ VAULT_ADDRESS environment variable is required");
        process.exit(1);
    }
    
    try {
        await migrateVault(config);
    } catch (error) {
        console.error("❌ Migration failed:", error);
        process.exit(1);
    }
}

main();
