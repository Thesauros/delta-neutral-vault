# Delta Neutral Vault Protocol - Deployment Guide

This guide provides comprehensive instructions for deploying the Delta Neutral Vault Protocol to different Solana networks.

## Prerequisites

### Required Software

- **Rust**: Latest stable version (1.70+)
- **Solana CLI**: Version 1.16.0 or higher
- **Anchor Framework**: Version 0.28.0
- **Node.js**: Version 16 or higher
- **Git**: Latest version

### Installation Commands

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/v1.16.0/install)"

# Install Anchor
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
avm install 0.28.0
avm use 0.28.0

# Install Node.js (using nvm)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
nvm install 18
nvm use 18
```

## Environment Setup

### 1. Clone Repository

```bash
git clone https://github.com/delta-neutral-vault/protocol.git
cd delta-neutral-vault
```

### 2. Install Dependencies

```bash
npm install
anchor build
```

### 3. Configure Environment

```bash
cp env.example .env
```

Edit `.env` file with your configuration:

```env
# Solana Network Configuration
SOLANA_NETWORK=devnet
SOLANA_RPC_URL=https://api.devnet.solana.com

# Program Configuration
PROGRAM_ID=DNVt1111111111111111111111111111111111111111

# Admin Configuration
ADMIN_KEYPAIR_PATH=./keys/admin.json

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
```

### 4. Generate Keypairs

```bash
# Generate admin keypair
solana-keygen new --outfile keys/admin.json

# Generate program keypair
solana-keygen new --outfile keys/program-keypair.json
```

### 5. Update Program ID

Update the program ID in `src/lib.rs`:

```rust
declare_id!("YOUR_PROGRAM_ID_HERE");
```

And in `Anchor.toml`:

```toml
[programs.localnet]
delta_neutral_vault = "YOUR_PROGRAM_ID_HERE"

[programs.devnet]
delta_neutral_vault = "YOUR_PROGRAM_ID_HERE"

[programs.mainnet]
delta_neutral_vault = "YOUR_PROGRAM_ID_HERE"
```

## Local Development

### 1. Start Local Validator

```bash
solana-test-validator
```

### 2. Build and Deploy

```bash
# Build the program
anchor build

# Deploy to localnet
anchor deploy --provider.cluster localnet
```

### 3. Run Tests

```bash
# Run all tests
anchor test

# Run specific test file
anchor test tests/integration_tests.ts

# Run with verbose output
anchor test -- --nocapture
```

## Devnet Deployment

### 1. Configure for Devnet

```bash
# Set Solana cluster to devnet
solana config set --url devnet

# Airdrop SOL for deployment
solana airdrop 2 $(solana-keygen pubkey keys/admin.json)
```

### 2. Deploy Program

```bash
# Build the program
anchor build

# Deploy to devnet
anchor deploy --provider.cluster devnet
```

### 3. Verify Deployment

```bash
# Check program account
solana account YOUR_PROGRAM_ID_HERE

# Verify program binary
solana program show YOUR_PROGRAM_ID_HERE
```

### 4. Initialize Vault

```bash
# Run setup script
npm run setup

# Deploy vault using deployment script
npm run deploy:devnet
```

## Mainnet Deployment

### 1. Security Checklist

Before deploying to mainnet, ensure:

- [ ] All tests pass
- [ ] Security audit completed
- [ ] Code review completed
- [ ] Emergency procedures documented
- [ ] Monitoring setup configured
- [ ] Backup procedures in place

### 2. Configure for Mainnet

```bash
# Set Solana cluster to mainnet
solana config set --url mainnet-beta

# Ensure sufficient SOL balance
solana balance $(solana-keygen pubkey keys/admin.json)
```

### 3. Deploy Program

```bash
# Build the program
anchor build

# Deploy to mainnet
anchor deploy --provider.cluster mainnet-beta
```

### 4. Verify Deployment

```bash
# Verify program deployment
solana program show YOUR_PROGRAM_ID_HERE

# Check program account
solana account YOUR_PROGRAM_ID_HERE
```

### 5. Initialize Vault

```bash
# Deploy vault to mainnet
npm run deploy:mainnet
```

## Post-Deployment Steps

### 1. Program Verification

```bash
# Verify program on Solana Explorer
# Visit: https://explorer.solana.com/address/YOUR_PROGRAM_ID_HERE

# Verify program on Anchor Registry
anchor verify YOUR_PROGRAM_ID_HERE
```

### 2. Monitor Deployment

```bash
# Monitor program logs
solana logs YOUR_PROGRAM_ID_HERE

# Monitor vault activity
solana logs VAULT_ADDRESS_HERE
```

### 3. Setup Monitoring

Configure monitoring for:

- **Program health**: Monitor program account status
- **Vault performance**: Track vault metrics
- **Error tracking**: Monitor for errors and exceptions
- **Performance metrics**: Track gas usage and transaction success rates

## Configuration Management

### 1. Environment-Specific Configs

Create environment-specific configuration files:

```bash
# Development
cp .env .env.development

# Staging
cp .env .env.staging

# Production
cp .env .env.production
```

### 2. Network-Specific Settings

Update `Anchor.toml` for different networks:

```toml
[provider]
cluster = "devnet"  # localnet, devnet, mainnet-beta
wallet = "~/.config/solana/id.json"

[programs.localnet]
delta_neutral_vault = "YOUR_PROGRAM_ID_HERE"

[programs.devnet]
delta_neutral_vault = "YOUR_PROGRAM_ID_HERE"

[programs.mainnet]
delta_neutral_vault = "YOUR_PROGRAM_ID_HERE"
```

## Troubleshooting

### Common Issues

#### 1. Build Errors

```bash
# Clean and rebuild
anchor clean
anchor build

# Check Rust version
rustc --version
cargo --version
```

#### 2. Deployment Errors

```bash
# Check Solana cluster status
solana cluster-version

# Check account balance
solana balance $(solana-keygen pubkey keys/admin.json)

# Check program account
solana account YOUR_PROGRAM_ID_HERE
```

#### 3. Test Failures

```bash
# Run tests with verbose output
anchor test -- --nocapture

# Check test validator
solana logs

# Restart test validator
pkill -f solana-test-validator
solana-test-validator
```

### Error Resolution

#### Insufficient Balance

```bash
# Airdrop SOL (devnet only)
solana airdrop 2 $(solana-keygen pubkey keys/admin.json)

# Transfer SOL from another account
solana transfer --from ~/.config/solana/id.json $(solana-keygen pubkey keys/admin.json) 1
```

#### Program Account Not Found

```bash
# Check if program was deployed
solana program show YOUR_PROGRAM_ID_HERE

# Redeploy if necessary
anchor deploy --provider.cluster devnet
```

#### Invalid Program ID

```bash
# Update program ID in all files
# 1. src/lib.rs
# 2. Anchor.toml
# 3. package.json (if applicable)

# Rebuild and redeploy
anchor build
anchor deploy
```

## Security Best Practices

### 1. Key Management

- **Use hardware wallets** for mainnet deployments
- **Secure key storage** with proper encryption
- **Regular key rotation** for admin accounts
- **Multi-signature setup** for critical operations

### 2. Access Control

- **Limit admin access** to trusted parties
- **Implement role-based access** for different functions
- **Monitor admin actions** with proper logging
- **Emergency procedures** for admin key compromise

### 3. Monitoring

- **Real-time monitoring** of vault operations
- **Alert systems** for unusual activity
- **Performance tracking** of all operations
- **Error logging** and analysis

## Backup and Recovery

### 1. Backup Procedures

```bash
# Backup keypairs
cp -r keys/ backup/keys/

# Backup configuration
cp .env backup/config/

# Backup program binary
cp target/deploy/delta_neutral_vault.so backup/program/
```

### 2. Recovery Procedures

```bash
# Restore from backup
cp backup/keys/* keys/
cp backup/config/.env .
cp backup/program/delta_neutral_vault.so target/deploy/
```

### 3. Disaster Recovery

- **Document recovery procedures** for different scenarios
- **Test recovery procedures** regularly
- **Maintain multiple backups** in different locations
- **Version control** for all configuration changes

## Performance Optimization

### 1. Gas Optimization

- **Batch operations** when possible
- **Optimize account validation** to minimize gas usage
- **Use efficient data structures** for storage
- **Implement caching** for frequently accessed data

### 2. Network Optimization

- **Use reliable RPC endpoints** for production
- **Implement retry logic** for failed transactions
- **Monitor network performance** and adjust accordingly
- **Use connection pooling** for multiple requests

## Conclusion

This deployment guide provides comprehensive instructions for deploying the Delta Neutral Vault Protocol to different Solana networks. Follow the security best practices and monitoring recommendations to ensure a successful deployment.

For additional support, refer to:

- [Solana Documentation](https://docs.solana.com/)
- [Anchor Framework Documentation](https://www.anchor-lang.com/docs)
- [Drift Protocol Documentation](https://docs.drift.trade/)
- [Project Issues](https://github.com/delta-neutral-vault/protocol/issues)
