// TypeScript types for LeveragedMeme program
// Generated from Anchor IDL

import { PublicKey, BN } from '@solana/web3.js'

// Program ID
export const PROGRAM_ID = new PublicKey('LMEME111111111111111111111111111111111111111')

// Enums
export enum Direction {
  Long = 'long',
  Short = 'short',
}

export enum OrderType {
  Market = 0,
  Limit = 1,
  TriggerMarket = 2,
  TriggerLimit = 3,
  MarketIfTouched = 4,
  LimitIfTouched = 5,
  Oracle = 6,
}

// Account Types
export interface TokenState {
  creator: PublicKey
  tokenMint: PublicKey
  name: string
  symbol: string
  uri: string
  leverage: number
  direction: Direction
  perpMarketIndex: number
  curveState: CurveState
  perpPosition: PerpPosition
  feeVault: PublicKey
  graduated: boolean
  ammPool: PublicKey | null
  createdAt: BN
  updatedAt: BN
  paused: boolean
  totalFeesCollected: BN
}

export interface CurveState {
  virtualSolReserve: BN
  virtualTokenReserve: BN
  realSolReserve: BN
  realTokenReserve: BN
  k: BN
}

export interface PerpPosition {
  baseAssetAmount: BN
  entryPrice: BN
  lastMarkPrice: BN
  unrealizedPnl: BN
  margin: BN
  leverage: number
  direction: Direction
  lastUpdate: BN
}

export interface FeeVault {
  tokenMint: PublicKey
  totalCollected: BN
  creatorClaimed: BN
  protocolClaimed: BN
  creatorShareBps: BN
}

// Instruction Types
export interface InitializeTokenArgs {
  name: string
  symbol: string
  uri: string
  leverage: number
  direction: Direction
  perpMarketIndex: number
}

export interface BuyArgs {
  amount: BN
}

export interface SellArgs {
  amount: BN
}

// Event Types
export interface TokenCreatedEvent {
  creator: PublicKey
  tokenMint: PublicKey
  name: string
  symbol: string
  leverage: number
  direction: Direction
  timestamp: BN
}

export interface TokenBoughtEvent {
  buyer: PublicKey
  tokenMint: PublicKey
  solAmount: BN
  tokenAmount: BN
  perpPositionSize: BN
  timestamp: BN
}

export interface TokenSoldEvent {
  seller: PublicKey
  tokenMint: PublicKey
  tokenAmount: BN
  solAmount: BN
  timestamp: BN
}

export interface TokenGraduatedEvent {
  tokenMint: PublicKey
  ammPool: PublicKey
  finalMarketCap: BN
  timestamp: BN
}

// Constants
export const TOKEN_DECIMALS = 6
export const TOTAL_SUPPLY = 1_000_000_000_000_000 // 1B with 6 decimals
export const CURVE_RESERVE_AMOUNT = TOTAL_SUPPLY * 75 / 100
export const LP_RESERVE_AMOUNT = TOTAL_SUPPLY * 25 / 100
export const GRADUATION_THRESHOLD_USD = 69_000_000_000 // $69k
export const MIN_LEVERAGE = 2
export const MAX_LEVERAGE = 5

// Perp Markets
export const PERP_MARKETS = {
  SOL: { index: 0, name: 'SOL-PERP' },
  BTC: { index: 1, name: 'BTC-PERP' },
  ETH: { index: 2, name: 'ETH-PERP' },
  APT: { index: 3, name: 'APT-PERP' },
  ARB: { index: 4, name: 'ARB-PERP' },
  DOGE: { index: 5, name: 'DOGE-PERP' },
  BNB: { index: 6, name: 'BNB-PERP' },
  SUI: { index: 7, name: 'SUI-PERP' },
  BONK: { index: 8, name: '1MBONK-PERP' },
  MATIC: { index: 9, name: 'MATIC-PERP' },
} as const

// Fee Structure
export const FEES = {
  deploy: 0.1, // SOL
  trading: 0.5, // %
  leverage: 0.1, // %
  graduation: 1, // %
  protocolShare: 50, // % of trading fees
  creatorShare: 50, // % of trading fees
} as const

// Helper Functions
export function calculateTokenPrice(
  curveState: CurveState,
  perpPosition: PerpPosition
): number {
  const basePrice = curveState.virtualSolReserve.toNumber() / 
                    curveState.virtualTokenReserve.toNumber()
  
  if (perpPosition.entryPrice.isZero()) {
    return basePrice
  }
  
  const priceChange = perpPosition.direction === Direction.Long
    ? perpPosition.lastMarkPrice.sub(perpPosition.entryPrice).toNumber()
    : perpPosition.entryPrice.sub(perpPosition.lastMarkPrice).toNumber()
  
  const pnlRatio = priceChange / perpPosition.entryPrice.toNumber()
  const leveragedPnl = pnlRatio * perpPosition.leverage
  
  return basePrice * (1 + leveragedPnl)
}

export function calculateMarketCap(
  tokenState: TokenState
): number {
  const price = calculateTokenPrice(
    tokenState.curveState,
    tokenState.perpPosition
  )
  const supply = tokenState.curveState.realTokenReserve.toNumber()
  return price * supply / 1_000_000
}

export function canGraduate(tokenState: TokenState): boolean {
  if (tokenState.graduated) return false
  const marketCap = calculateMarketCap(tokenState)
  return marketCap >= 69_000
}

export function getPerpMarketName(index: number): string {
  const market = Object.values(PERP_MARKETS).find(m => m.index === index)
  return market?.name || 'Unknown'
}
