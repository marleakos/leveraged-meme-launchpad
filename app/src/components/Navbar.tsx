import { Link } from 'react-router-dom'
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui'
import { Rocket, TrendingUp, Wallet } from 'lucide-react'

export default function Navbar() {
  return (
    <nav className="bg-black/50 backdrop-blur-md border-b border-purple-500/20">
      <div className="container mx-auto px-4 py-4">
        <div className="flex items-center justify-between">
          <Link to="/" className="flex items-center gap-2">
            <Rocket className="w-8 h-8 text-purple-400" />
            <div>
              <h1 className="text-xl font-bold bg-gradient-to-r from-purple-400 to-pink-400 bg-clip-text text-transparent">
                LeveragedMeme
              </h1>
              <p className="text-xs text-gray-400">Pump.fun with leverage</p>
            </div>
          </Link>
          
          <div className="flex items-center gap-6">
            <Link 
              to="/" 
              className="flex items-center gap-2 text-gray-300 hover:text-white transition"
            >
              <TrendingUp className="w-4 h-4" />
              <span>Tokens</span>
            </Link>
            
            <Link 
              to="/launch" 
              className="flex items-center gap-2 px-4 py-2 bg-purple-600 hover:bg-purple-700 rounded-lg transition"
            >
              <Rocket className="w-4 h-4" />
              <span>Launch</span>
            </Link>
            
            <Link 
              to="/portfolio" 
              className="flex items-center gap-2 text-gray-300 hover:text-white transition"
            >
              <Wallet className="w-4 h-4" />
              <span>Portfolio</span>
            </Link>
            
            <WalletMultiButton className="!bg-purple-600 hover:!bg-purple-700" />
          </div>
        </div>
      </div>
    </nav>
  )
}
