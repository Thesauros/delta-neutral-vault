import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, Keypair, SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, createMint, createAccount, mintTo, getAccount } from "@solana/spl-token";
import { DeltaNeutralVault } from "../target/types/delta_neutral_vault";
import { assert } from "chai";

describe("Delta Neutral Vault Integration Tests", () => {
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);

    const program = anchor.workspace.DeltaNeutralVault as Program<DeltaNeutralVault>;
    
    let adminKeypair: Keypair;
    let userKeypair: Keypair;
    let tokenMint: PublicKey;
    let vaultState: PublicKey;
    let vaultTokenAccount: PublicKey;
    let userTokenAccount: PublicKey;

    const VAULT_PARAMS = {
        targetLeverage: 2,
        rebalanceThreshold: 100, // 1%
        maxSlippage: 50, // 0.5%
    };

    before(async () => {
        console.log("🚀 Setting up integration tests...");

        // Generate keypairs
        adminKeypair = Keypair.generate();
        userKeypair = Keypair.generate();

        // Airdrop SOL to admin
        const signature = await provider.connection.requestAirdrop(
            adminKeypair.publicKey,
            10 * anchor.web3.LAMPORTS_PER_SOL
        );
        await provider.connection.confirmTransaction(signature);

        // Airdrop SOL to user
        const userSignature = await provider.connection.requestAirdrop(
            userKeypair.publicKey,
            5 * anchor.web3.LAMPORTS_PER_SOL
        );
        await provider.connection.confirmTransaction(userSignature);

        // Create test token mint
        tokenMint = await createMint(
            provider.connection,
            adminKeypair,
            adminKeypair.publicKey,
            null,
            6 // 6 decimals like USDC
        );

        console.log(`✅ Test token mint created: ${tokenMint.toString()}`);

        // Create user token account
        userTokenAccount = await createAccount(
            provider.connection,
            userKeypair,
            tokenMint,
            userKeypair.publicKey
        );

        // Mint tokens to user
        await mintTo(
            provider.connection,
            adminKeypair,
            tokenMint,
            userTokenAccount,
            adminKeypair,
            1000000000 // 1000 tokens
        );

        console.log("✅ Test setup completed");
    });

    it("Should initialize vault successfully", async () => {
        console.log("🧪 Testing vault initialization...");

        // Derive PDA for vault state
        const [vaultStatePda, vaultBump] = PublicKey.findProgramAddressSync(
            [Buffer.from("vault"), adminKeypair.publicKey.toBuffer()],
            program.programId
        );
        vaultState = vaultStatePda;

        // Derive PDA for vault token account
        const [vaultTokenAccountPda, tokenBump] = PublicKey.findProgramAddressSync(
            [Buffer.from("vault_token_account"), vaultState.toBuffer()],
            program.programId
        );
        vaultTokenAccount = vaultTokenAccountPda;

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
                    vaultState: vaultState,
                    vaultTokenAccount: vaultTokenAccount,
                    admin: adminKeypair.publicKey,
                    tokenMint: tokenMint,
                    driftProgram: new PublicKey("DRiFTvSoSLjH8XJx2wSJ1GL9jR8qXhF2vC2QBK5FyL9"),
                    tokenProgram: TOKEN_PROGRAM_ID,
                    systemProgram: SystemProgram.programId,
                    rent: SYSVAR_RENT_PUBKEY,
                })
                .signers([adminKeypair])
                .rpc();

            console.log(`✅ Vault initialized successfully. Tx: ${tx}`);

            // Verify vault state
            const vaultAccount = await program.account.vaultState.fetch(vaultState);
            assert.equal(vaultAccount.admin.toString(), adminKeypair.publicKey.toString());
            assert.equal(vaultAccount.targetLeverage, VAULT_PARAMS.targetLeverage);
            assert.equal(vaultAccount.rebalanceThreshold, VAULT_PARAMS.rebalanceThreshold);
            assert.equal(vaultAccount.maxSlippage, VAULT_PARAMS.maxSlippage);
            assert.equal(vaultAccount.totalAssets, 0);
            assert.equal(vaultAccount.totalShares, 0);

            console.log("✅ Vault state verified");

        } catch (error) {
            console.error("❌ Vault initialization failed:", error);
            throw error;
        }
    });

    it("Should allow deposits", async () => {
        console.log("🧪 Testing deposits...");

        const depositAmount = 100000; // 0.1 tokens

        try {
            // Get initial balances
            const initialVaultBalance = await getAccount(provider.connection, vaultTokenAccount);
            const initialUserBalance = await getAccount(provider.connection, userTokenAccount);

            console.log(`Initial vault balance: ${initialVaultBalance.amount}`);
            console.log(`Initial user balance: ${initialUserBalance.amount}`);

            // Deposit funds
            const tx = await program.methods
                .deposit(new anchor.BN(depositAmount))
                .accounts({
                    vaultState: vaultState,
                    vaultTokenAccount: vaultTokenAccount,
                    depositorTokenAccount: userTokenAccount,
                    depositor: userKeypair.publicKey,
                    tokenProgram: TOKEN_PROGRAM_ID,
                })
                .signers([userKeypair])
                .rpc();

            console.log(`✅ Deposit successful. Tx: ${tx}`);

            // Verify balances
            const finalVaultBalance = await getAccount(provider.connection, vaultTokenAccount);
            const finalUserBalance = await getAccount(provider.connection, userTokenAccount);

            console.log(`Final vault balance: ${finalVaultBalance.amount}`);
            console.log(`Final user balance: ${finalUserBalance.amount}`);

            assert.equal(
                finalVaultBalance.amount,
                initialVaultBalance.amount + depositAmount
            );
            assert.equal(
                finalUserBalance.amount,
                initialUserBalance.amount - depositAmount
            );

            // Verify vault state
            const vaultAccount = await program.account.vaultState.fetch(vaultState);
            assert.equal(vaultAccount.totalAssets, depositAmount);
            assert.equal(vaultAccount.totalShares, depositAmount);

            console.log("✅ Deposit verification completed");

        } catch (error) {
            console.error("❌ Deposit failed:", error);
            throw error;
        }
    });

    it("Should allow withdrawals", async () => {
        console.log("🧪 Testing withdrawals...");

        const withdrawAmount = 50000; // 0.05 tokens

        try {
            // Get initial balances
            const initialVaultBalance = await getAccount(provider.connection, vaultTokenAccount);
            const initialUserBalance = await getAccount(provider.connection, userTokenAccount);

            console.log(`Initial vault balance: ${initialVaultBalance.amount}`);
            console.log(`Initial user balance: ${initialUserBalance.amount}`);

            // Withdraw funds
            const tx = await program.methods
                .withdraw(new anchor.BN(withdrawAmount))
                .accounts({
                    vaultState: vaultState,
                    vaultTokenAccount: vaultTokenAccount,
                    withdrawerTokenAccount: userTokenAccount,
                    withdrawer: userKeypair.publicKey,
                    tokenProgram: TOKEN_PROGRAM_ID,
                })
                .signers([userKeypair])
                .rpc();

            console.log(`✅ Withdrawal successful. Tx: ${tx}`);

            // Verify balances
            const finalVaultBalance = await getAccount(provider.connection, vaultTokenAccount);
            const finalUserBalance = await getAccount(provider.connection, userTokenAccount);

            console.log(`Final vault balance: ${finalVaultBalance.amount}`);
            console.log(`Final user balance: ${finalUserBalance.amount}`);

            assert.equal(
                finalVaultBalance.amount,
                initialVaultBalance.amount - withdrawAmount
            );
            assert.equal(
                finalUserBalance.amount,
                initialUserBalance.amount + withdrawAmount
            );

            // Verify vault state
            const vaultAccount = await program.account.vaultState.fetch(vaultState);
            assert.equal(vaultAccount.totalAssets, 50000); // 100000 - 50000
            assert.equal(vaultAccount.totalShares, 50000);

            console.log("✅ Withdrawal verification completed");

        } catch (error) {
            console.error("❌ Withdrawal failed:", error);
            throw error;
        }
    });

    it("Should handle emergency stop", async () => {
        console.log("🧪 Testing emergency stop...");

        try {
            // Trigger emergency stop
            const tx = await program.methods
                .emergencyStop()
                .accounts({
                    vaultState: vaultState,
                    admin: adminKeypair.publicKey,
                })
                .signers([adminKeypair])
                .rpc();

            console.log(`✅ Emergency stop successful. Tx: ${tx}`);

            // Verify vault state
            const vaultAccount = await program.account.vaultState.fetch(vaultState);
            assert.isTrue(vaultAccount.emergencyStop);

            console.log("✅ Emergency stop verification completed");

        } catch (error) {
            console.error("❌ Emergency stop failed:", error);
            throw error;
        }
    });

    it("Should update vault parameters", async () => {
        console.log("🧪 Testing parameter updates...");

        const newTargetLeverage = 3;
        const newRebalanceThreshold = 150;
        const newMaxSlippage = 75;

        try {
            // Update vault parameters
            const tx = await program.methods
                .updateVaultParams(
                    newTargetLeverage,
                    newRebalanceThreshold,
                    newMaxSlippage
                )
                .accounts({
                    vaultState: vaultState,
                    admin: adminKeypair.publicKey,
                })
                .signers([adminKeypair])
                .rpc();

            console.log(`✅ Parameter update successful. Tx: ${tx}`);

            // Verify vault state
            const vaultAccount = await program.account.vaultState.fetch(vaultState);
            assert.equal(vaultAccount.targetLeverage, newTargetLeverage);
            assert.equal(vaultAccount.rebalanceThreshold, newRebalanceThreshold);
            assert.equal(vaultAccount.maxSlippage, newMaxSlippage);

            console.log("✅ Parameter update verification completed");

        } catch (error) {
            console.error("❌ Parameter update failed:", error);
            throw error;
        }
    });

    after(async () => {
        console.log("🧹 Cleaning up test environment...");
        
        // Log final vault state
        try {
            const vaultAccount = await program.account.vaultState.fetch(vaultState);
            console.log("📊 Final vault state:");
            console.log(`- Total assets: ${vaultAccount.totalAssets}`);
            console.log(`- Total shares: ${vaultAccount.totalShares}`);
            console.log(`- Emergency stop: ${vaultAccount.emergencyStop}`);
            console.log(`- Target leverage: ${vaultAccount.targetLeverage}`);
        } catch (error) {
            console.log("Could not fetch final vault state");
        }

        console.log("✅ Integration tests completed");
    });
});
