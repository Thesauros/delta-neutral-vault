import * as anchor from "@coral-xyz/anchor";
import { Keypair, Connection, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { writeFileSync, existsSync, mkdirSync } from "fs";
import { join } from "path";
import * as dotenv from "dotenv";

// Load environment variables
dotenv.config();

const NETWORK = process.env.SOLANA_NETWORK || "devnet";
const ADMIN_KEYPAIR_PATH = process.env.ADMIN_KEYPAIR_PATH || "./keys/admin.json";

async function main() {
    console.log("🚀 Setting up Delta Neutral Vault Protocol...");
    console.log(`Network: ${NETWORK}`);

    // Create keys directory if it doesn't exist
    const keysDir = join(__dirname, "../keys");
    if (!existsSync(keysDir)) {
        mkdirSync(keysDir, { recursive: true });
        console.log("📁 Created keys directory");
    }

    // Generate admin keypair if it doesn't exist
    let adminKeypair: Keypair;
    if (existsSync(ADMIN_KEYPAIR_PATH)) {
        console.log("🔑 Loading existing admin keypair...");
        const secretKey = JSON.parse(require("fs").readFileSync(ADMIN_KEYPAIR_PATH, "utf8"));
        adminKeypair = Keypair.fromSecretKey(Uint8Array.from(secretKey));
    } else {
        console.log("🔑 Generating new admin keypair...");
        adminKeypair = Keypair.generate();
        writeFileSync(ADMIN_KEYPAIR_PATH, JSON.stringify(Array.from(adminKeypair.secretKey)));
        console.log(`✅ Admin keypair saved to ${ADMIN_KEYPAIR_PATH}`);
    }

    // Setup connection
    const connection = new Connection(
        anchor.web3.clusterApiUrl(NETWORK as anchor.web3.Cluster),
        "confirmed"
    );

    console.log(`🔗 Connected to ${NETWORK}`);

    // Check admin balance
    const balance = await connection.getBalance(adminKeypair.publicKey);
    console.log(`💰 Admin balance: ${balance / LAMPORTS_PER_SOL} SOL`);

    if (balance < LAMPORTS_PER_SOL) {
        console.log("⚠️  Low balance detected. Consider funding the admin account.");
        if (NETWORK === "devnet") {
            console.log("💡 You can airdrop SOL on devnet using: solana airdrop 2 <PUBKEY>");
        }
    }

    // Generate program keypair if it doesn't exist
    const programKeypairPath = join(keysDir, "program-keypair.json");
    if (!existsSync(programKeypairPath)) {
        console.log("🔑 Generating program keypair...");
        const programKeypair = Keypair.generate();
        writeFileSync(programKeypairPath, JSON.stringify(Array.from(programKeypair.secretKey)));
        console.log(`✅ Program keypair saved to ${programKeypairPath}`);
        console.log(`📋 Program ID: ${programKeypair.publicKey.toString()}`);
    }

    // Create .env file if it doesn't exist
    const envPath = join(__dirname, "../.env");
    if (!existsSync(envPath)) {
        console.log("📝 Creating .env file...");
        const envContent = `# Solana Network Configuration
SOLANA_NETWORK=${NETWORK}
SOLANA_RPC_URL=${anchor.web3.clusterApiUrl(NETWORK as anchor.web3.Cluster)}

# Program Configuration
PROGRAM_ID=DNVt1111111111111111111111111111111111111111

# Admin Configuration
ADMIN_KEYPAIR_PATH=${ADMIN_KEYPAIR_PATH}

# Drift Protocol Configuration
DRIFT_PROGRAM_ID=DRiFTvSoSLjH8XJx2wSJ1GL9jR8qXhF2vC2QBK5FyL9

# Token Configuration
USDC_MINT=EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v

# Vault Configuration
DEFAULT_TARGET_LEVERAGE=2
DEFAULT_REBALANCE_THRESHOLD=100
DEFAULT_MAX_SLIPPAGE=50
DEFAULT_MANAGEMENT_FEE=200
DEFAULT_PERFORMANCE_FEE=2000

# Risk Management
EMERGENCY_STOP_ENABLED=true
MAX_VAULT_CAPACITY=1000000000000

# Monitoring
ENABLE_LOGGING=true
LOG_LEVEL=info

# Development
ANCHOR_PROVIDER_URL=${anchor.web3.clusterApiUrl(NETWORK as anchor.web3.Cluster)}
ANCHOR_WALLET=${ADMIN_KEYPAIR_PATH}
`;
        writeFileSync(envPath, envContent);
        console.log("✅ .env file created");
    }

    // Check if Anchor.toml exists and update if needed
    const anchorTomlPath = join(__dirname, "../Anchor.toml");
    if (existsSync(anchorTomlPath)) {
        console.log("✅ Anchor.toml found");
    } else {
        console.log("⚠️  Anchor.toml not found. Please create it manually.");
    }

    // Check if Cargo.toml exists
    const cargoTomlPath = join(__dirname, "../Cargo.toml");
    if (existsSync(cargoTomlPath)) {
        console.log("✅ Cargo.toml found");
    } else {
        console.log("⚠️  Cargo.toml not found. Please create it manually.");
    }

    console.log("\n🎉 Setup completed successfully!");
    console.log("\n📋 Next steps:");
    console.log("1. Build the program: npm run build");
    console.log("2. Run tests: npm test");
    console.log("3. Deploy to devnet: npm run deploy:devnet");
    console.log("4. Deploy to mainnet: npm run deploy:mainnet");

    console.log("\n🔑 Key addresses:");
    console.log(`Admin: ${adminKeypair.publicKey.toString()}`);
    console.log(`Program: DNVt1111111111111111111111111111111111111111`);

    return {
        adminKeypair,
        connection,
        network: NETWORK
    };
}

main().catch((error) => {
    console.error("❌ Setup failed:", error);
    process.exit(1);
});
