# Drift Protocol Integration Guide

## Overview

This document explains how the Leveraged Meme Launchpad integrates with Drift Protocol for perpetual futures trading.

## Architecture

```
User Buy Request
       ↓
LeveragedMeme Program
       ↓
   ┌─────────────┐
   │  1. Calculate│
   │    position  │
   │    size      │
   └─────────────┘
       ↓
   ┌─────────────┐
   │  2. Place    │
   │    perp order│
   │    on Drift  │
   └─────────────┘
       ↓
   ┌─────────────┐
   │  3. Mint     │
   │    tokens to │
   │    user      │
   └─────────────┘
```

## Drift CPI Integration

### Programs Involved

1. **Drift Program**: `dRiftyHA39MWEi3m9aunc5MzRF1JYuBsbn6VPcn33UH`
2. **Drift State**: Global state account
3. **Drift User**: Per-user account for positions
4. **Pyth Oracles**: Price feeds for mark prices

### Account Structure

```rust
/// Drift accounts needed for perp trading
pub struct DriftAccounts<'info> {
    /// Drift program
    pub drift_program: AccountInfo<'info>,
    
    /// Drift state (global)
    pub drift_state: AccountInfo<'info>,
    
    /// User account (PDA owned by our program)
    pub drift_user: AccountInfo<'info>,
    
    /// User stats
    pub drift_user_stats: AccountInfo<'info>,
    
    /// Pyth price feed
    pub pyth_price_feed: AccountInfo<'info>,
}
```

## Perp Markets Supported

| Market Index | Asset | Pyth Feed |
|-------------|-------|-----------|
| 0 | SOL-PERP | SOL/USD |
| 1 | BTC-PERP | BTC/USD |
| 2 | ETH-PERP | ETH/USD |
| 3 | APT-PERP | APT/USD |
| 4 | ARB-PERP | ARB/USD |
| 5 | DOGE-PERP | DOGE/USD |
| 6 | BNB-PERP | BNB/USD |
| 7 | SUI-PERP | SUI/USD |
| 8 | 1MBONK-PERP | BONK/USD |
| 9 | MATIC-PERP | MATIC/USD |

## Position Calculation

### Formula

```rust
position_size = (collateral * leverage * 10^9) / mark_price
```

Where:
- `collateral`: SOL amount from buyer
- `leverage`: 2x, 3x, or 5x
- `mark_price`: Current price from Pyth oracle

### Example

```
User buys with: 1 SOL
Leverage: 3x
SOL price: $150

Position size = (1 * 3 * 10^9) / 150
              = 20,000,000 base units
              = 0.02 SOL-PERP
```

## Order Types

### Market Order (Default)

```rust
OrderParams {
    order_type: OrderType::Market,
    market_type: MarketType::Perp,
    direction: PositionDirection::Long, // or Short
    base_asset_amount: position_size,
    market_index: 0, // SOL-PERP
    ..Default::default()
}
```

### Order Flow

1. **Buy Token** → Open long perp position
2. **Sell Token** → Reduce/close perp position
3. **Graduate** → Close all perp positions

## Risk Management

### Liquidation Protection

```rust
// Check if position is at risk
if perp_position.is_at_liquidation_risk(625_000) { // 6.25%
    // Trigger deleveraging
    deleverage_position()?;
}
```

### Maintenance Margin

- **5x leverage**: 6.25% maintenance margin
- **3x leverage**: 10% maintenance margin
- **2x leverage**: 15% maintenance margin

### Circuit Breakers

```rust
// Emergency pause if needed
if extreme_volatility_detected {
    token_state.paused = true;
}
```

## Price Oracle Integration

### Pyth Price Feeds

```rust
// Fetch current price
let price = fetch_price(
    &pyth_price_account,
    clock.slot,
)?;
```

### Price Validation

```rust
// Check staleness
if price.is_stale(current_slot) {
    return Err(OracleError::StalePrice);
}

// Check confidence
let confidence = price.get_confidence()?;
if confidence > price * 0.01 { // >1% confidence interval
    return Err(OracleError::LowConfidence);
}
```

## Fee Structure

### Drift Fees

| Fee Type | Amount | Notes |
|----------|--------|-------|
| Taker Fee | 0.1% | Market orders |
| Maker Rebate | -0.02% | Limit orders (not used) |
| Funding | Variable | Every hour |

### Our Fees

| Fee Type | Amount | Recipient |
|----------|--------|-----------|
| Trading Fee | 0.5% | 50/50 split |
| Leverage Fee | 0.1% | Protocol |
| Graduation | 1% | Protocol |

## Testing on Devnet

### Setup

```bash
# 1. Fund wallet
solana airdrop 10

# 2. Create Drift user account
# This is done automatically on first deposit

# 3. Deposit collateral
drift deposit --amount 1 --market 0

# 4. Place test order
drift place-perp-order --market 0 --size 0.01 --direction long
```

### Test Scenarios

1. **Normal Buy/Sell**
   - User buys token
   - Perp position opened
   - Price moves with SOL

2. **Liquidation Scenario**
   - Open 5x long
   - SOL dumps 20%
   - Position liquidated
   - Token value drops

3. **Graduation**
   - Token reaches $69k
   - Close perp position
   - Migrate to Raydium

## Mainnet Deployment

### Prerequisites

1. **Drift User Account**
   - Must be created before trading
   - One-time setup cost: ~0.02 SOL

2. **Collateral Requirements**
   - Minimum deposit: 1 SOL
   - Buffer for fees: 0.1 SOL

3. **Oracle Accounts**
   - Pyth price feeds must be live
   - Valid slot threshold: < 60 slots

### Deployment Steps

```bash
# 1. Deploy program
anchor deploy --provider.cluster mainnet

# 2. Initialize Drift user
# (Done via CPI in initialize_token)

# 3. Test with small amount
# Buy 0.1 SOL worth

# 4. Monitor metrics
# - Perp position health
# - Liquidation levels
# - Funding rates
```

## Monitoring

### Key Metrics

```rust
// Position health
let health = calculate_position_health()?;

// Distance to liquidation
let liq_distance = calculate_liquidation_distance()?;

// Funding PnL
let funding_pnl = get_funding_pnl()?;

// Total perp exposure
let total_exposure = get_total_perp_exposure()?;
```

### Alerts

- Position health < 20%
- Oracle price stale > 60 slots
- Funding rate > 0.1%/hour
- Liquidation imminent

## Troubleshooting

### Common Issues

1. **"Insufficient collateral"**
   - Check Drift user balance
   - Ensure enough SOL for margin

2. **"Oracle price stale"**
   - Pyth feed may be down
   - Check slot difference

3. **"Position not found"**
   - Drift user not initialized
   - Run initialize_user first

4. **"Order too small"**
   - Minimum order size: 0.001 base units
   - Increase trade size

### Debug Commands

```bash
# Check Drift user
solana account <drift_user_pubkey>

# Check perp position
drift get-perp-position --market 0

# Check oracle price
solana account <pyth_feed_pubkey>
```

## Future Improvements

1. **Limit Orders**: Use Drift limit orders instead of market
2. **Multiple Collateral**: Support USDC deposits
3. **Cross-Margin**: Use portfolio margin
4. **Advanced Orders**: Stop-loss, take-profit
5. **Insurance Fund**: Protect against bad debt

## Resources

- [Drift Documentation](https://docs.drift.trade/)
- [Drift SDK](https://github.com/drift-labs/protocol-v2)
- [Pyth Documentation](https://docs.pyth.network/)
- [Solana Cookbook](https://solanacookbook.com/)

---

**Note**: This integration is for advanced users. Ensure you understand perpetual futures risks before deploying.
