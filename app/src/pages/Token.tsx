import { useState } from 'react'
import { useParams } from 'react-router-dom'
import { useConnection, useWallet } from '@solana/wallet-adapter-react'
import { TrendingUp, TrendingDown, Wallet, ArrowUpRight, ArrowDownRight } from 'lucide-react'
import toast from 'react-hot-toast'

export default function Token() {
  const { mint } = useParams()
  const { connection } = useConnection()
  const wallet = useWallet()
  
  const [activeTab, setActiveTab] = useState<'buy' | 'sell'>('buy')
  const [amount, setAmount] = useState('')
  const [loading, setLoading] = useState(false)

  // Mock token data
  const token = {
    name: 'Super Bull',
    symbol: 'BULL',
    leverage: 3,
    direction: 'long' as const,
    perpMarket: 'SOL',
    marketCap: 45000,
    price: 0.000045,
    solReserve: 5230,
    tokenReserve: 115000000,
    graduated: false,
    creator: '7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU',
    createdAt: '2024-01-15',
  }

  const handleTrade = async () => {
    if (!wallet.publicKey) {
      toast.error('Please connect your wallet')
      return
    }
    
    if (!amount || Number(amount) <= 0) {
      toast.error('Please enter a valid amount')
      return
    }
    
    setLoading(true)
    
    try {
      // TODO: Call program instruction
      toast.success(`${activeTab === 'buy' ? 'Buy' : 'Sell'} transaction sent!`)
    } catch (error) {
      console.error('Trade error:', error)
      toast.error('Transaction failed')
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="max-w-6xl mx-auto">
      <div className="grid grid-cols-3 gap-8">
        {/* Left Column - Token Info */}
        <div className="col-span-2 space-y-6">
          {/* Header */}
          <div className="bg-gray-800/50 rounded-2xl p-8 border border-purple-500/20">
            <div className="flex items-start justify-between">
              <div className="flex items-center gap-4">
                <div className="w-20 h-20 bg-gradient-to-br from-purple-500 to-pink-500 rounded-full flex items-center justify-center font-bold text-3xl">
                  {token.symbol[0]}
                </div>
                <div>
                  <h1 className="text-3xl font-bold">{token.name}</h1>
                  <p className="text-gray-400 text-lg">${token.symbol}</p>
                </div>
              </div>
              <div className="text-right">
                <p className="text-sm text-gray-400">Current Price</p>
                <p className="text-3xl font-bold">{token.price.toFixed(8)} SOL</p>
                <p className="text-sm text-gray-400">${(token.price * 45).toFixed(6)} USD</p>
              </div>
            </div>
            
            <div className="grid grid-cols-4 gap-4 mt-8">
              <div className="bg-gray-900/50 rounded-lg p-4">
                <p className="text-sm text-gray-400">Market Cap</p>
                <p className="text-xl font-bold">${token.marketCap.toLocaleString()}</p>
              </div>
              <div className="bg-gray-900/50 rounded-lg p-4">
                <p className="text-sm text-gray-400">Leverage</p>
                <p className={`text-xl font-bold ${
                  token.direction === 'long' ? 'text-green-400' : 'text-red-400'
                }`}>
                  {token.leverage}x {token.direction === 'long' ? 'Long' : 'Short'}
                </p>
              </div>
              <div className="bg-gray-900/50 rounded-lg p-4">
                <p className="text-sm text-gray-400">Underlying</p>
                <p className="text-xl font-bold">{token.perpMarket}-PERP</p>
              </div>
              <div className="bg-gray-900/50 rounded-lg p-4">
                <p className="text-sm text-gray-400">Status</p>
                <p className="text-xl font-bold text-purple-400">
                  {token.graduated ? 'Graduated' : 'Trading'}
                </p>
              </div>
            </div>
          </div>

          {/* Price Chart Placeholder */}
          <div className="bg-gray-800/50 rounded-2xl p-8 border border-purple-500/20">
            <h2 className="text-xl font-bold mb-4">Price Chart</h2>
            <div className="h-64 bg-gray-900/50 rounded-lg flex items-center justify-center">
              <p className="text-gray-500">Price chart will be displayed here</p>
            </div>
          </div>

          {/* Bonding Curve Info */}
          <div className="bg-gray-800/50 rounded-2xl p-8 border border-purple-500/20">
            <h2 className="text-xl font-bold mb-4">Bonding Curve</h2>
            <div className="grid grid-cols-2 gap-6">
              <div>
                <p className="text-sm text-gray-400 mb-1">SOL Reserve</p>
                <p className="text-2xl font-bold">{token.solReserve.toLocaleString()} SOL</p>
              </div>
              <div>
                <p className="text-sm text-gray-400 mb-1">Token Reserve</p>
                <p className="text-2xl font-bold">{token.tokenReserve.toLocaleString()} {token.symbol}</p>
              </div>
            </div>
            
            {/* Progress to graduation */}
            <div className="mt-6">
              <div className="flex justify-between text-sm mb-2">
                <span className="text-gray-400">Progress to Graduation</span>
                <span className="font-medium">{Math.round((token.marketCap / 69000) * 100)}%</span>
              </div>
              <div className="h-2 bg-gray-700 rounded-full overflow-hidden">
                <div 
                  className="h-full bg-gradient-to-r from-purple-500 to-pink-500 rounded-full"
                  style={{ width: `${Math.min((token.marketCap / 69000) * 100, 100)}%` }}
                />
              </div>
              <p className="text-sm text-gray-400 mt-2">
                ${token.marketCap.toLocaleString()} / $69,000
              </p>
            </div>
          </div>
        </div>

        {/* Right Column - Trading */}
        <div className="space-y-6">
          <div className="bg-gray-800/50 rounded-2xl p-6 border border-purple-500/20 sticky top-4">
            {/* Tabs */}
            <div className="flex gap-2 mb-6">
              <button
                onClick={() => setActiveTab('buy')}
                className={`flex-1 py-3 rounded-lg font-bold flex items-center justify-center gap-2 transition ${
                  activeTab === 'buy'
                    ? 'bg-green-600 text-white'
                    : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
                }`}
              >
                <ArrowUpRight className="w-5 h-5" />
                Buy
              </button>
              <button
                onClick={() => setActiveTab('sell')}
                className={`flex-1 py-3 rounded-lg font-bold flex items-center justify-center gap-2 transition ${
                  activeTab === 'sell'
                    ? 'bg-red-600 text-white'
                    : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
                }`}
              >
                <ArrowDownRight className="w-5 h-5" />
                Sell
              </button>
            </div>

            {/* Amount Input */}
            <div className="mb-4">
              <label className="block text-sm text-gray-400 mb-2">
                {activeTab === 'buy' ? 'SOL Amount' : 'Token Amount'}
              </label>
              <div className="relative">
                <input
                  type="number"
                  value={amount}
                  onChange={(e) => setAmount(e.target.value)}
                  className="w-full px-4 py-3 bg-gray-900 rounded-lg border border-gray-700 focus:border-purple-500 focus:outline-none"
                  placeholder="0.00"
                  min="0"
                  step="0.01"
                />
                <span className="absolute right-4 top-1/2 -translate-y-1/2 text-gray-400">
                  {activeTab === 'buy' ? 'SOL' : token.symbol}
                </span>
              </div>
            </div>

            {/* Estimated Output */}
            {amount && (
              <div className="bg-gray-900/50 rounded-lg p-4 mb-4">
                <p className="text-sm text-gray-400 mb-1">You will receive</p>
                <p className="text-xl font-bold">
                  {activeTab === 'buy' 
                    ? `${(Number(amount) / token.price).toFixed(2)} ${token.symbol}`
                    : `${(Number(amount) * token.price).toFixed(4)} SOL`
                  }
                </p>
                <p className="text-sm text-gray-400 mt-1">
                  Fee: {(Number(amount) * 0.005).toFixed(4)} {activeTab === 'buy' ? 'SOL' : token.symbol}
                </p>
              </div>
            )}

            {/* Trade Button */}
            <button
              onClick={handleTrade}
              disabled={loading || !wallet.publicKey}
              className={`w-full py-4 rounded-lg font-bold text-lg transition disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2 ${
                activeTab === 'buy'
                  ? 'bg-green-600 hover:bg-green-700'
                  : 'bg-red-600 hover:bg-red-700'
              }`}
            >
              {loading ? (
                <div className="w-6 h-6 border-2 border-white/30 border-t-white rounded-full animate-spin" />
              ) : (
                <>
                  <Wallet className="w-5 h-5" />
                  {activeTab === 'buy' ? 'Buy' : 'Sell'} {token.symbol}
                </>
              )}
            </button>

            {/* Wallet Balance */}
            {wallet.publicKey && (
              <div className="mt-4 pt-4 border-t border-gray-700">
                <p className="text-sm text-gray-400">Your Balance</p>
                <p className="font-medium">0.00 {token.symbol}</p>
                <p className="text-sm text-gray-400">0.00 SOL</p>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  )
}
