import { useState, useEffect } from 'react'
import { useConnection, useWallet } from '@solana/wallet-adapter-react'
import { TrendingUp, TrendingDown, Wallet, ExternalLink } from 'lucide-react'

interface Position {
  token: string
  symbol: string
  balance: number
  avgPrice: number
  currentPrice: number
  value: number
  pnl: number
  pnlPercent: number
  leverage: number
  direction: 'long' | 'short'
}

export default function Portfolio() {
  const { connection } = useConnection()
  const wallet = useWallet()
  const [positions, setPositions] = useState<Position[]>([])
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    // TODO: Fetch from blockchain
    const mockPositions: Position[] = [
      {
        token: 'Super Bull',
        symbol: 'BULL',
        balance: 5000000,
        avgPrice: 0.00004,
        currentPrice: 0.000045,
        value: 225,
        pnl: 25,
        pnlPercent: 12.5,
        leverage: 3,
        direction: 'long',
      },
      {
        token: 'Bear Hunter',
        symbol: 'BEAR',
        balance: 2500000,
        avgPrice: 0.00005,
        currentPrice: 0.000048,
        value: 120,
        pnl: -5,
        pnlPercent: -4,
        leverage: 5,
        direction: 'short',
      },
    ]
    
    setPositions(mockPositions)
    setLoading(false)
  }, [wallet.publicKey])

  const totalValue = positions.reduce((sum, pos) => sum + pos.value, 0)
  const totalPnl = positions.reduce((sum, pos) => sum + pos.pnl, 0)
  const totalPnlPercent = totalValue > 0 ? (totalPnl / (totalValue - totalPnl)) * 100 : 0

  if (!wallet.publicKey) {
    return (
      <div className="text-center py-20">
        <Wallet className="w-16 h-16 mx-auto mb-4 text-gray-600" />
        <h2 className="text-2xl font-bold mb-2">Connect Your Wallet</h2>
        <p className="text-gray-400">Connect your wallet to view your portfolio</p>
      </div>
    )
  }

  return (
    <div>
      <h1 className="text-3xl font-bold mb-8">Your Portfolio</h1>

      {/* Summary Cards */}
      <div className="grid grid-cols-3 gap-6 mb-8">
        <div className="bg-gray-800/50 rounded-xl p-6 border border-purple-500/20">
          <p className="text-gray-400 text-sm mb-1">Total Value</p>
          <p className="text-3xl font-bold">{totalValue.toFixed(2)} SOL</p>
          <p className="text-sm text-gray-400">${(totalValue * 45).toFixed(2)} USD</p>
        </div>
        
        <div className="bg-gray-800/50 rounded-xl p-6 border border-purple-500/20">
          <p className="text-gray-400 text-sm mb-1">Total PnL</p>
          <p className={`text-3xl font-bold ${totalPnl >= 0 ? 'text-green-400' : 'text-red-400'}`}>
            {totalPnl >= 0 ? '+' : ''}{totalPnl.toFixed(2)} SOL
          </p>
          <p className={`text-sm ${totalPnlPercent >= 0 ? 'text-green-400' : 'text-red-400'}`}>
            {totalPnlPercent >= 0 ? '+' : ''}{totalPnlPercent.toFixed(2)}%
          </p>
        </div>
        
        <div className="bg-gray-800/50 rounded-xl p-6 border border-purple-500/20">
          <p className="text-gray-400 text-sm mb-1">Positions</p>
          <p className="text-3xl font-bold">{positions.length}</p>
          <p className="text-sm text-gray-400">
            {positions.filter(p => p.pnl > 0).length} profitable
          </p>
        </div>
      </div>

      {/* Positions Table */}
      <div className="bg-gray-800/50 rounded-xl border border-purple-500/20 overflow-hidden">
        <div className="p-6 border-b border-gray-700">
          <h2 className="text-xl font-bold">Your Positions</h2>
        </div>
        
        {loading ? (
          <div className="flex justify-center py-12">
            <div className="w-8 h-8 border-2 border-purple-500/30 border-t-purple-500 rounded-full animate-spin" />
          </div>
        ) : positions.length === 0 ? (
          <div className="text-center py-12">
            <p className="text-gray-400">No positions yet</p>
            <p className="text-sm text-gray-500 mt-1">Start trading to see your portfolio</p>
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead className="bg-gray-900/50">
                <tr>
                  <th className="text-left p-4 text-sm font-medium text-gray-400">Token</th>
                  <th className="text-right p-4 text-sm font-medium text-gray-400">Balance</th>
                  <th className="text-right p-4 text-sm font-medium text-gray-400">Avg Price</th>
                  <th className="text-right p-4 text-sm font-medium text-gray-400">Current</th>
                  <th className="text-right p-4 text-sm font-medium text-gray-400">Value</th>
                  <th className="text-right p-4 text-sm font-medium text-gray-400">PnL</th>
                  <th className="text-center p-4 text-sm font-medium text-gray-400">Leverage</th>
                </tr>
              </thead>
              <tbody>
                {positions.map((pos, index) => (
                  <tr key={index} className="border-t border-gray-700 hover:bg-gray-700/30 transition">
                    <td className="p-4">
                      <div className="flex items-center gap-3">
                        <div className="w-10 h-10 bg-gradient-to-br from-purple-500 to-pink-500 rounded-full flex items-center justify-center font-bold">
                          {pos.symbol[0]}
                        </div>
                        <div>
                          <p className="font-medium">{pos.token}</p>
                          <p className="text-sm text-gray-400">${pos.symbol}</p>
                        </div>
                      </div>
                    </td>
                    <td className="p-4 text-right">
                      <p className="font-medium">{pos.balance.toLocaleString()}</p>
                      <p className="text-sm text-gray-400">${pos.symbol}</p>
                    </td>
                    <td className="p-4 text-right text-gray-400">
                      {pos.avgPrice.toFixed(8)} SOL
                    </td>
                    <td className="p-4 text-right">
                      {pos.currentPrice.toFixed(8)} SOL
                    </td>
                    <td className="p-4 text-right">
                      <p className="font-medium">{pos.value.toFixed(2)} SOL</p>
                      <p className="text-sm text-gray-400">${(pos.value * 45).toFixed(2)}</p>
                    </td>
                    <td className="p-4 text-right">
                      <p className={`font-medium ${pos.pnl >= 0 ? 'text-green-400' : 'text-red-400'}`}>
                        {pos.pnl >= 0 ? '+' : ''}{pos.pnl.toFixed(2)} SOL
                      </p>
                      <p className={`text-sm ${pos.pnlPercent >= 0 ? 'text-green-400' : 'text-red-400'}`}>
                        {pos.pnlPercent >= 0 ? '+' : ''}{pos.pnlPercent.toFixed(2)}%
                      </p>
                    </td>
                    <td className="p-4 text-center">
                      <span className={`inline-flex items-center gap-1 px-3 py-1 rounded-full text-sm font-medium ${
                        pos.direction === 'long' 
                          ? 'bg-green-500/20 text-green-400' 
                          : 'bg-red-500/20 text-red-400'
                      }`}>
                        {pos.direction === 'long' ? (
                          <TrendingUp className="w-4 h-4" />
                        ) : (
                          <TrendingDown className="w-4 h-4" />
                        )}
                        {pos.leverage}x {pos.direction === 'long' ? 'Long' : 'Short'}
                      </span>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>

      {/* Recent Activity */}
      <div className="mt-8 bg-gray-800/50 rounded-xl border border-purple-500/20">
        <div className="p-6 border-b border-gray-700">
          <h2 className="text-xl font-bold">Recent Activity</h2>
        </div>
        <div className="p-6">
          <div className="space-y-4">
            <div className="flex items-center justify-between p-4 bg-gray-900/50 rounded-lg">
              <div className="flex items-center gap-3">
                <div className="w-10 h-10 bg-green-500/20 rounded-full flex items-center justify-center">
                  <TrendingUp className="w-5 h-5 text-green-400" />
                </div>
                <div>
                  <p className="font-medium">Bought BULL</p>
                  <p className="text-sm text-gray-400">2 hours ago</p>
                </div>
              </div>
              <div className="text-right">
                <p className="font-medium">+5,000,000 BULL</p>
                <p className="text-sm text-gray-400">-0.2 SOL</p>
              </div>
            </div>
            
            <div className="flex items-center justify-between p-4 bg-gray-900/50 rounded-lg">
              <div className="flex items-center gap-3">
                <div className="w-10 h-10 bg-red-500/20 rounded-full flex items-center justify-center">
                  <TrendingDown className="w-5 h-5 text-red-400" />
                </div>
                <div>
                  <p className="font-medium">Sold BEAR</p>
                  <p className="text-sm text-gray-400">5 hours ago</p>
                </div>
              </div>
              <div className="text-right">
                <p className="font-medium">-2,500,000 BEAR</p>
                <p className="text-sm text-gray-400">+0.12 SOL</p>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
