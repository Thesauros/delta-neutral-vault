# Delta Neutral Vault Protocol - Quick Start Guide

Get up and running with the Delta Neutral Vault Protocol in minutes!

## 🚀 Quick Start

### 1. Prerequisites

Ensure you have the following installed:
- Rust (latest stable)
- Solana CLI (v1.16.0+)
- Anchor Framework (v0.28.0)
- Node.js (v16+)

### 2. Setup

```bash
# Clone the repository
git clone https://github.com/delta-neutral-vault/protocol.git
cd delta-neutral-vault

# Install dependencies
npm install

# Setup environment and generate keys
npm run setup

# Build the program
npm run build
```

### 3. Test

```bash
# Run all tests
npm test

# Run specific test file
npm test tests/integration_tests.ts
```

### 4. Deploy

```bash
# Deploy to devnet
npm run deploy:devnet

# Deploy to mainnet (after testing)
npm run deploy:mainnet
```

## 📋 What's Included

### Core Features
- ✅ Delta-neutral position management
- ✅ Automated rebalancing
- ✅ Risk management controls
- ✅ Fee collection system
- ✅ Drift Protocol integration
- ✅ Emergency stop functionality

### Development Tools
- ✅ Comprehensive test suite
- ✅ CI/CD pipeline
- ✅ Security auditing
- ✅ Documentation
- ✅ Deployment scripts

### Documentation
- 📖 [Architecture Guide](docs/ARCHITECTURE.md)
- 📖 [Deployment Guide](docs/DEPLOYMENT.md)
- 📖 [API Reference](docs/API.md)

## 🔧 Configuration

Edit `.env` file to configure:

```env
# Network
SOLANA_NETWORK=devnet

# Vault parameters
DEFAULT_TARGET_LEVERAGE=2
DEFAULT_REBALANCE_THRESHOLD=100
DEFAULT_MAX_SLIPPAGE=50

# Fees
DEFAULT_MANAGEMENT_FEE=200    # 2% annual
DEFAULT_PERFORMANCE_FEE=2000  # 20% of profits
```

## 🧪 Testing

```bash
# Run unit tests
cargo test

# Run integration tests
anchor test

# Run with coverage
cargo tarpaulin
```

## 📊 Monitoring

Monitor your vault with:

```bash
# Check vault state
solana account VAULT_ADDRESS

# Monitor logs
solana logs PROGRAM_ID

# Track events
# Events are emitted for all major operations
```

## 🔒 Security

- Emergency stop functionality
- Slippage protection
- Admin controls
- Comprehensive validation
- Security best practices

## 🚨 Emergency Procedures

```bash
# Emergency stop
npm run emergency-stop

# Pause operations
npm run pause-vault

# Resume operations
npm run resume-vault
```

## 📈 Performance

- Gas optimized operations
- Efficient storage usage
- Batch processing support
- Scalable architecture

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## 📞 Support

- 📧 Email: support@delta-neutral-vault.com
- 💬 Discord: [discord.gg/delta-neutral](https://discord.gg/delta-neutral)
- 🐛 Issues: [GitHub Issues](https://github.com/delta-neutral-vault/protocol/issues)

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.

---

**Ready to deploy?** Check out the [Deployment Guide](docs/DEPLOYMENT.md) for detailed instructions!
