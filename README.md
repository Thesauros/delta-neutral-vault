# Delta Neutral Vault Protocol

[![Build Status](https://github.com/delta-neutral-vault/protocol/workflows/CI/badge.svg)](https://github.com/delta-neutral-vault/protocol/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Solana](https://img.shields.io/badge/Solana-1.16.0-blue.svg)](https://solana.com/)
[![Anchor](https://img.shields.io/badge/Anchor-0.28.0-orange.svg)](https://www.anchor-lang.com/)

A sophisticated DeFi vault protocol built on Solana that maintains delta-neutral positions using the Drift Protocol. This protocol allows users to deposit assets and automatically maintains market-neutral exposure through sophisticated rebalancing mechanisms.

## 🚀 Features

- **Delta-Neutral Strategy**: Maintains market-neutral exposure through long/short position balancing
- **Automated Rebalancing**: Intelligent rebalancing based on configurable thresholds
- **Risk Management**: Built-in emergency stops and slippage protection
- **Fee Structure**: Management and performance fees for sustainable operations
- **Drift Integration**: Leverages Drift Protocol for perpetual futures trading
- **Multi-Asset Support**: Support for various token types
- **Governance Ready**: Upgradeable contracts with admin controls

## 📋 Prerequisites

- [Node.js](https://nodejs.org/) (v16 or higher)
- [Rust](https://rustup.rs/) (latest stable)
- [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools) (v1.16.0 or higher)
- [Anchor Framework](https://www.anchor-lang.com/docs/installation) (v0.28.0)

## 🛠 Installation

1. **Clone the repository**
   ```bash
   git clone https://github.com/delta-neutral-vault/protocol.git
   cd delta-neutral-vault
   ```

2. **Install dependencies**
   ```bash
   npm install
   ```

3. **Build the program**
   ```bash
   anchor build
   ```

4. **Run tests**
   ```bash
   anchor test
   ```

## 🏗 Architecture

### Core Components

- **Vault State**: Main vault account storing configuration and state
- **Instructions**: Core operations (deposit, withdraw, rebalance, etc.)
- **Drift Integration**: Perpetual futures trading integration
- **Risk Management**: Slippage protection and emergency controls
- **Fee Collection**: Management and performance fee mechanisms

### Key Accounts

- `VaultState`: PDA storing vault configuration and state
- `VaultTokenAccount`: PDA for vault's token holdings
- `DriftUser`: Drift protocol user account
- `DriftUserStats`: Drift protocol user statistics

## 📖 Usage

### Deployment

1. **Setup environment**
   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   ```

2. **Deploy to devnet**
   ```bash
   npm run deploy:devnet
   ```

3. **Deploy to mainnet**
   ```bash
   npm run deploy:mainnet
   ```

### Basic Operations

```typescript
// Initialize vault
await program.methods
  .initializeVault(2, 100, 50) // leverage, threshold, slippage
  .accounts({...})
  .rpc();

// Deposit funds
await program.methods
  .deposit(new BN(1000000)) // 1 USDC (6 decimals)
  .accounts({...})
  .rpc();

// Withdraw funds
await program.methods
  .withdraw(new BN(500000)) // 0.5 USDC
  .accounts({...})
  .rpc();

// Rebalance positions
await program.methods
  .rebalance()
  .accounts({...})
  .rpc();
```

## 🔧 Configuration

### Vault Parameters

- `target_leverage`: Target leverage ratio (e.g., 2x)
- `rebalance_threshold`: Threshold to trigger rebalancing (basis points)
- `max_slippage`: Maximum allowed slippage (basis points)
- `management_fee`: Annual management fee (basis points)
- `performance_fee`: Performance fee (basis points)

### Risk Parameters

- `emergency_stop`: Emergency stop flag
- `max_capacity`: Maximum vault capacity
- `min_rebalance_interval`: Minimum time between rebalances
- `delta_threshold`: Delta threshold for rebalancing

## 🧪 Testing

```bash
# Run all tests
anchor test

# Run specific test file
anchor test tests/vault_tests.rs

# Run with verbose output
anchor test -- --nocapture
```

## 📊 Monitoring

### Key Metrics

- Total Assets Under Management (AUM)
- Current Delta Exposure
- Performance Metrics
- Fee Collection
- Rebalancing Frequency

### Events

The protocol emits events for:
- Deposits and withdrawals
- Rebalancing operations
- Fee collections
- Emergency stops
- Parameter updates

## 🔒 Security

### Audit Status

- [ ] External audit pending
- [ ] Internal security review completed
- [ ] Bug bounty program planned

### Security Features

- Emergency stop functionality
- Slippage protection
- Admin controls
- Upgradeable contracts
- Comprehensive testing

## 📈 Performance

### Gas Optimization

- Efficient account validation
- Minimal instruction data
- Optimized storage usage
- Batch operations support

### Scalability

- Support for multiple vaults
- Configurable parameters
- Modular architecture
- Upgradeable design

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Development Guidelines

- Follow Rust coding standards
- Write comprehensive tests
- Update documentation
- Follow security best practices

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🆘 Support

- **Documentation**: [docs.delta-neutral-vault.com](https://docs.delta-neutral-vault.com)
- **Discord**: [discord.gg/delta-neutral](https://discord.gg/delta-neutral)
- **Twitter**: [@DeltaNeutralVault](https://twitter.com/DeltaNeutralVault)
- **Email**: support@delta-neutral-vault.com

## ⚠️ Disclaimer

This software is for educational purposes only. Use at your own risk. The authors are not responsible for any financial losses incurred through the use of this software.

## 🔗 Links

- [Solana Documentation](https://docs.solana.com/)
- [Anchor Framework](https://www.anchor-lang.com/)
- [Drift Protocol](https://drift.trade/)
- [SPL Token Program](https://spl.solana.com/token)
