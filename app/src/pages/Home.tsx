import { useState, useEffect } from 'react'
import { Link } from 'react-router-dom'
import { TrendingUp, TrendingDown, ExternalLink } from 'lucide-react'

interface Token {
  mint: string
  name: string
  symbol: string
  leverage: number
  direction: 'long' | 'short'
  perpMarket: string
  marketCap: number
  price: number
  graduated: boolean
}

export default function Home() {
  const [tokens, setTokens] = useState<Token[]>([])
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    // TODO: Fetch from program
    const mockTokens: Token[] = [
      {
        mint: 'token1',
        name: 'Super Bull',
        symbol: 'BULL',
        leverage: 3,
        direction: 'long',
        perpMarket: 'SOL',
        marketCap: 45000,
        price: 0.000045,
        graduated: false,
      },
      {
        mint: 'token2',
        name: 'Bear Hunter',
        symbol: 'BEAR',
        leverage: 5,
        direction: 'short',
        perpMarket: 'BTC',
        marketCap: 120000,
        price: 0.00012,
        graduated: true,
      },
    ]
    
    setTokens(mockTokens)
    setLoading(false)
  }, [])

  return (
    <div>
      <div className="text-center mb-12">
        <h1 className="text-5xl font-bold mb-4">
          <span className="bg-gradient-to-r from-purple-400 to-pink-400 bg-clip-text text-transparent">
            Leveraged Meme Tokens
          </span>
        </h1>
        <p className="text-xl text-gray-400 max-w-2xl mx-auto">
          Launch meme tokens with built-in leverage. 
          Your token price moves with the underlying perp market.
        </p>
      </div>

      {/* Stats */}
      <div className="grid grid-cols-4 gap-6 mb-12">
        <div className="bg-gray-800/50 rounded-xl p-6 border border-purple-500/20">
          <p className="text-gray-400 text-sm">Total Tokens</p>
          <p className="text-3xl font-bold">{tokens.length}</p>
        </div>
        <div className="bg-gray-800/50 rounded-xl p-6 border border-purple-500/20">
          <p className="text-gray-400 text-sm">Graduated</p>
          <p className="text-3xl font-bold">{tokens.filter(t => t.graduated).length}</p>
        </div>
        <div className="bg-gray-800/50 rounded-xl p-6 border border-purple-500/20">
          <p className="text-gray-400 text-sm">Total Volume</p>
          <p className="text-3xl font-bold">1.2M SOL</p>
        </div>
        <div className="bg-gray-800/50 rounded-xl p-6 border border-purple-500/20">
          <p className="text-gray-400 text-sm">Fees Generated</p>
          <p className="text-3xl font-bold">45K SOL</p>
        </div>
      </div>

      {/* Token List */}
      <div>
        <h2 className="text-2xl font-bold mb-6">Active Tokens</h2>
        
        {loading ? (
          <div className="flex justify-center py-12">
            <div className="w-8 h-8 border-2 border-purple-500/30 border-t-purple-500 rounded-full animate-spin" />
          </div>
        ) : (
          <div className="grid gap-4">
            {tokens.map((token) => (
              <Link
                key={token.mint}
                to={`/token/${token.mint}`}
                className="bg-gray-800/50 rounded-xl p-6 border border-purple-500/20 hover:border-purple-500/50 transition group"
              >
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-4">
                    <div className="w-12 h-12 bg-gradient-to-br from-purple-500 to-pink-500 rounded-full flex items-center justify-center font-bold text-lg">
                      {token.symbol[0]}
                    </div>
                    <div>
                      <h3 className="font-bold text-lg group-hover:text-purple-400 transition">
                        {token.name}
                      </h3>
                      <p className="text-gray-400">${token.symbol}</p>
                    </div>
                  </div>
                  
                  <div className="flex items-center gap-8">
                    <div className="text-center">
                      <p className="text-sm text-gray-400">Leverage</p>
                      <p className={`font-bold ${
                        token.direction === 'long' ? 'text-green-400' : 'text-red-400'
                      }`}>
                        {token.leverage}x {token.direction === 'long' ? 'Long' : 'Short'}
                      </p>
                    </div>
                    
                    <div className="text-center">
                      <p className="text-sm text-gray-400">Underlying</p>
                      <p className="font-bold">{token.perpMarket}-PERP</p>
                    </div>
                    
                    <div className="text-center">
                      <p className="text-sm text-gray-400">Market Cap</p>
                      <p className="font-bold">${token.marketCap.toLocaleString()}</p>
                    </div>
                    
                    <div className="text-center">
                      <p className="text-sm text-gray-400">Price</p>
                      <p className="font-bold">{token.price.toFixed(8)} SOL</p>
                    </div>
                    
                    {token.graduated ? (
                      <span className="px-3 py-1 bg-green-500/20 text-green-400 rounded-full text-sm font-medium">
                        Graduated
                      </span>
                    ) : (
                      <span className="px-3 py-1 bg-purple-500/20 text-purple-400 rounded-full text-sm font-medium">
                        Trading
                      </span>
                    )}
                    
                    <ExternalLink className="w-5 h-5 text-gray-400 group-hover:text-white transition" />
                  </div>
                </div>
              </Link>
            ))}
          </div>
        )}
      </div>
    </div>
  )
}
