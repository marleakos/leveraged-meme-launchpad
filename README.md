# Leveraged Meme Launchpad

**Pump.fun with built-in leverage.** Launch meme tokens backed by leveraged perp positions on Solana.

## 🚀 What is This?

A token launchpad that combines:
- **Pump.fun's bonding curve** (proven mechanics)
- **Drift Protocol's perps** (leverage)
- **Your own features** (bundles, analytics, auto-sell)

### How It Works

1. Launch a token with 2x, 3x, or 5x leverage
2. Token is backed by a perp position (SOL-PERP, BTC-PERP, ETH-PERP)
3. Token price moves with the underlying × leverage
4. Graduate to Raydium at $69k market cap

### Example

Launch $BULL with 3x SOL-PERP long:
- SOL pumps 10% → $BULL pumps 30%
- SOL dumps 10% → $BULL dumps 30%
- Built-in liquidation protection

## 📁 Project Structure

```
leveraged-meme-launchpad/
├── programs/
│   └── leveraged_meme/        # Anchor smart contract
│       ├── src/
│       │   ├── lib.rs         # Program entry
│       │   ├── constants.rs   # Config
│       │   ├── state.rs       # Account structures
│       │   ├── errors.rs      # Error codes
│       │   └── instructions/  # Instruction handlers
│       └── Cargo.toml
├── app/                       # React frontend
│   ├── src/
│   │   ├── components/        # UI components
│   │   ├── pages/             # Page components
│   │   ├── hooks/             # Custom hooks
│   │   └── utils/             # Utilities
│   └── package.json
├── tests/                     # TypeScript tests
├── scripts/                   # Deployment scripts
└── Anchor.toml               # Anchor config
```

## 🛠️ Tech Stack

### Smart Contract
- **Anchor Framework** (Rust)
- **Drift Protocol** (perp integration)
- **Pyth Oracles** (price feeds)

### Frontend
- **React + TypeScript**
- **Vite** (build tool)
- **Tailwind CSS** (styling)
- **Solana Wallet Adapter**

### Blockchain
- **Solana** (mainnet/devnet)
- **SPL Tokens**
- **Raydium AMM** (graduation)

## 🚀 Quick Start

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/v1.18.0/install)"

# Install Anchor
npm install -g @coral-xyz/anchor-cli

# Install Node dependencies
cd app && npm install
```

### Build & Deploy

```bash
# 1. Clone repository
git clone <repo-url>
cd leveraged-meme-launchpad

# 2. Build program
anchor build

# 3. Deploy to devnet
./scripts/deploy.sh devnet

# 4. Run tests
anchor test

# 5. Start frontend
cd app
npm run dev
```

### Environment Variables

Create `app/.env`:

```env
VITE_SOLANA_NETWORK=devnet
VITE_SOLANA_RPC_URL=https://api.devnet.solana.com
VITE_PROGRAM_ID=your_program_id_here
```

## 📊 Features

### Core Features
- ✅ **Leveraged Token Launch** (2x, 3x, 5x)
- ✅ **Bonding Curve Trading**
- ✅ **Perp Position Backing**
- ✅ **Graduation to Raydium**
- ✅ **Fee Collection**

### Advanced Features (TODO)
- [ ] Multi-wallet bundles
- [ ] Auto-sell strategies
- [ ] AI token name generator
- [ ] Social media integration
- [ ] Analytics dashboard
- [ ] Copy trading

## 💰 Fee Structure

| Fee Type | Amount | Recipient |
|----------|--------|-----------|
| Deploy | 0.1 SOL | Protocol |
| Trading | 0.5% | 50/50 Protocol/Creator |
| Leverage | 0.1% | Protocol |
| Graduation | 1% | Protocol |

## 🔒 Security

### Audits
- [ ] Pending audit

### Risk Management
- Liquidation protection
- Circuit breakers (pause)
- Maximum leverage limits
- Emergency withdrawal

## 📝 Smart Contract Architecture

### Accounts

```rust
TokenState {
    creator: Pubkey,
    token_mint: Pubkey,
    name: String,
    symbol: String,
    leverage: u8,
    direction: Direction,
    curve_state: CurveState,
    perp_position: PerpPosition,
    graduated: bool,
}

CurveState {
    virtual_sol_reserve: u64,
    virtual_token_reserve: u64,
    real_sol_reserve: u64,
    real_token_reserve: u64,
    k: u128, // constant product
}
```

### Instructions

1. `initialize_token` - Launch new token
2. `buy` - Buy from curve
3. `sell` - Sell to curve
4. `graduate` - Migrate to AMM
5. `set_pause` - Emergency pause
6. `sync_perp_position` - Update perp PnL

## 🧪 Testing

```bash
# Run all tests
anchor test

# Run specific test
anchor test --grep "initialize_token"

# Test with logs
anchor test -- --nocapture
```

## 🚀 Deployment Checklist

### Devnet
- [ ] Build succeeds
- [ ] All tests pass
- [ ] Deploy to devnet
- [ ] Test full flow
- [ ] Frontend integration

### Mainnet
- [ ] Security audit
- [ ] Bug bounty program
- [ ] Graduation threshold review
- [ ] Fee structure finalization
- [ ] Deploy to mainnet
- [ ] Monitor metrics

## 📈 Roadmap

### Phase 1: MVP (Weeks 1-2)
- [x] Basic contract
- [x] Frontend UI
- [ ] Drift integration
- [ ] Devnet deployment

### Phase 2: Features (Weeks 3-4)
- [ ] Multi-wallet bundles
- [ ] Auto-sell
- [ ] Analytics
- [ ] Mainnet prep

### Phase 3: Scale (Weeks 5-8)
- [ ] Audit
- [ ] Mainnet launch
- [ ] Marketing
- [ ] Community

## 🤝 Contributing

1. Fork the repository
2. Create feature branch
3. Commit changes
4. Push to branch
5. Open Pull Request

## 📄 License

MIT License - see LICENSE file

## 🔗 Links

- [Documentation](docs/)
- [Frontend Demo](https://your-demo-url.com)
- [Program Explorer](https://explorer.solana.com)

## 💬 Support

- Discord: [your-discord]
- Twitter: [@yourtwitter]
- Email: support@leveragedmeme.io

---

**Built with ❤️ on Solana**
