# Devnet Testing Guide

## Prerequisites

```bash
# 1. Ensure you have Solana CLI installed
solana --version

# 2. Set to devnet
solana config set --url devnet

# 3. Generate or use existing keypair
solana-keygen new --outfile ~/.config/solana/devnet.json

# 4. Set as default
solana config set --keypair ~/.config/solana/devnet.json

# 5. Get devnet SOL
solana airdrop 10
```

## Build & Deploy

```bash
# Navigate to project
cd leveraged-meme-launchpad

# Build the program
anchor build

# Deploy to devnet
anchor deploy --provider.cluster devnet

# Note the program ID output
# Update Anchor.toml and frontend with new program ID
```

## Test Script

```bash
# Run the test suite
anchor test --provider.cluster devnet

# Or run specific test
anchor test --grep "initialize_token"
```

## Manual Testing Steps

### 1. Initialize Token

```typescript
import { useProgram } from './hooks/useProgram'

const program = useProgram()

// Create token mint
const tokenMint = Keypair.generate()

// Derive PDAs
const [tokenState] = PublicKey.findProgramAddressSync(
  [Buffer.from('token_state'), tokenMint.publicKey.toBuffer()],
  PROGRAM_ID
)

// Initialize
tx = await program.methods
  .initializeToken(
    "Test Token",
    "TEST",
    "https://example.com/metadata.json",
    3, // 3x leverage
    { long: {} },
    0 // SOL-PERP
  )
  .accounts({
    creator: wallet.publicKey,
    tokenMint: tokenMint.publicKey,
    tokenState,
    // ... other accounts
  })
  .signers([tokenMint])
  .rpc()
```

### 2. Buy Tokens

```typescript
// Buy with 1 SOL
tx = await program.methods
  .buyWithDrift(new BN(1 * LAMPORTS_PER_SOL))
  .accounts({
    buyer: wallet.publicKey,
    tokenState,
    // ... other accounts including Drift
  })
  .rpc()
```

### 3. Check Token State

```typescript
const state = await program.account.tokenState.fetch(tokenState)
console.log({
  name: state.name,
  symbol: state.symbol,
  leverage: state.leverage,
  direction: state.direction,
  curveState: state.curveState,
  perpPosition: state.perpPosition,
})
```

### 4. Sell Tokens

```typescript
tx = await program.methods
  .sell(new BN(1000000)) // Sell 1 token
  .accounts({
    seller: wallet.publicKey,
    tokenState,
    // ... other accounts
  })
  .rpc()
```

## Expected Behavior

### Buy Flow
1. User sends SOL
2. Program opens perp position on Drift
3. Program mints tokens to user
4. Token price updates with perp PnL

### Price Movement
```
Initial: Token price = $0.0001
SOL pumps 10% with 3x leverage
New price = $0.0001 × 1.30 = $0.00013
```

### Graduation
```
When market cap reaches $69,000:
1. Close perp position
2. Migrate liquidity to Raydium
3. Burn remaining curve tokens
```

## Troubleshooting

### "Insufficient funds"
```bash
solana airdrop 10
```

### "Account not found"
- Ensure PDAs are derived correctly
- Check account initialization

### "Drift error"
- Verify Drift program ID
- Check if markets are live on devnet

### "Oracle error"
- Pyth feeds may be stale on devnet
- Use mock prices for testing

## Monitoring

### Check Token Accounts
```bash
solana account <token_state_pubkey>
```

### View Transaction
```bash
solana confirm -v <transaction_signature>
```

### Check Balance
```bash
solana balance
```

## Frontend Testing

```bash
cd app

# Install dependencies
npm install

# Create .env file
cat > .env << EOF
VITE_SOLANA_NETWORK=devnet
VITE_SOLANA_RPC_URL=https://api.devnet.solana.com
VITE_PROGRAM_ID=<your_program_id>
EOF

# Start dev server
npm run dev
```

## Test Checklist

- [ ] Initialize token
- [ ] Buy tokens
- [ ] Price updates correctly
- [ ] Sell tokens
- [ ] Perp position tracked
- [ ] Fees collected
- [ ] Pause/unpause works
- [ ] Graduation triggers

## Next Steps After Devnet

1. **Security Audit**
   - Get professional audit
   - Fix any issues

2. **Mainnet Preparation**
   - Update program ID
   - Deploy to mainnet
   - Initialize with real parameters

3. **Launch**
   - Announce on Twitter
   - Community building
   - Monitor metrics

---

**Note**: Devnet is for testing only. Tokens have no real value.
