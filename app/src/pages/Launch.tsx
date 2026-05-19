import { useState } from 'react'
import { useConnection, useWallet } from '@solana/wallet-adapter-react'
import { Keypair, SystemProgram, PublicKey } from '@solana/web3.js'
import { getAssociatedTokenAddress, TOKEN_PROGRAM_ID } from '@solana/spl-token'
import { Program, AnchorProvider, web3, BN } from '@coral-xyz/anchor'
import toast from 'react-hot-toast'
import { Rocket, TrendingUp, TrendingDown, Info } from 'lucide-react'

const PROGRAM_ID = new PublicKey('LMEME111111111111111111111111111111111111111')

export default function Launch() {
  const { connection } = useConnection()
  const wallet = useWallet()
  
  const [formData, setFormData] = useState({
    name: '',
    symbol: '',
    uri: '',
    leverage: 3,
    direction: 'long',
    perpMarket: 0,
  })
  const [loading, setLoading] = useState(false)

  const handleLaunch = async (e: React.FormEvent) => {
    e.preventDefault()
    
    if (!wallet.publicKey || !wallet.signTransaction) {
      toast.error('Please connect your wallet')
      return
    }
    
    setLoading(true)
    
    try {
      const provider = new AnchorProvider(connection, wallet as any, {})
      // const program = new Program(idl, PROGRAM_ID, provider)
      
      // Generate token mint
      const tokenMint = Keypair.generate()
      
      // Derive PDAs
      const [tokenState] = PublicKey.findProgramAddressSync(
        [Buffer.from('token_state'), tokenMint.publicKey.toBuffer()],
        PROGRAM_ID
      )
      
      const [curveState] = PublicKey.findProgramAddressSync(
        [Buffer.from('curve_state'), tokenMint.publicKey.toBuffer()],
        PROGRAM_ID
      )
      
      const [feeVault] = PublicKey.findProgramAddressSync(
        [Buffer.from('fee_vault'), tokenMint.publicKey.toBuffer()],
        PROGRAM_ID
      )
      
      const curveTokenAccount = await getAssociatedTokenAddress(
        tokenMint.publicKey,
        tokenState,
        true
      )
      
      const lpTokenAccount = await getAssociatedTokenAddress(
        tokenMint.publicKey,
        tokenState,
        true
      )
      
      // TODO: Call program instruction
      toast.success('Token launch transaction sent!')
      
      console.log('Launch params:', {
        ...formData,
        tokenMint: tokenMint.publicKey.toString(),
        tokenState: tokenState.toString(),
      })
      
    } catch (error) {
      console.error('Launch error:', error)
      toast.error('Failed to launch token')
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="max-w-2xl mx-auto">
      <div className="text-center mb-8">
        <h1 className="text-4xl font-bold mb-4">Launch Leveraged Token</h1>
        <p className="text-gray-400">
          Create a meme token backed by leveraged perp positions
        </p>
      </div>
      
      <form onSubmit={handleLaunch} className="bg-gray-800/50 rounded-2xl p-8 border border-purple-500/20">
        {/* Token Name */}
        <div className="mb-6">
          <label className="block text-sm font-medium mb-2">Token Name</label>
          <input
            type="text"
            value={formData.name}
            onChange={(e) => setFormData({ ...formData, name: e.target.value })}
            className="w-full px-4 py-3 bg-gray-900 rounded-lg border border-gray-700 focus:border-purple-500 focus:outline-none"
            placeholder="e.g., Super Bull"
            maxLength={32}
            required
          />
        </div>
        
        {/* Symbol */}
        <div className="mb-6">
          <label className="block text-sm font-medium mb-2">Symbol</label>
          <input
            type="text"
            value={formData.symbol}
            onChange={(e) => setFormData({ ...formData, symbol: e.target.value.toUpperCase() })}
            className="w-full px-4 py-3 bg-gray-900 rounded-lg border border-gray-700 focus:border-purple-500 focus:outline-none"
            placeholder="e.g., BULL"
            maxLength={10}
            required
          />
        </div>
        
        {/* Image URI */}
        <div className="mb-6">
          <label className="block text-sm font-medium mb-2">Image URL</label>
          <input
            type="url"
            value={formData.uri}
            onChange={(e) => setFormData({ ...formData, uri: e.target.value })}
            className="w-full px-4 py-3 bg-gray-900 rounded-lg border border-gray-700 focus:border-purple-500 focus:outline-none"
            placeholder="https://..."
          />
        </div>
        
        {/* Leverage */}
        <div className="mb-6">
          <label className="block text-sm font-medium mb-2">Leverage</label>
          <div className="grid grid-cols-3 gap-4">
            {[2, 3, 5].map((lev) => (
              <button
                key={lev}
                type="button"
                onClick={() => setFormData({ ...formData, leverage: lev })}
                className={`py-3 rounded-lg font-bold transition ${
                  formData.leverage === lev
                    ? 'bg-purple-600 text-white'
                    : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
                }`}
              >
                {lev}x
              </button>
            ))}
          </div>
        </div>
        
        {/* Direction */}
        <div className="mb-6">
          <label className="block text-sm font-medium mb-2">Direction</label>
          <div className="grid grid-cols-2 gap-4">
            <button
              type="button"
              onClick={() => setFormData({ ...formData, direction: 'long' })}
              className={`py-3 rounded-lg font-bold flex items-center justify-center gap-2 transition ${
                formData.direction === 'long'
                  ? 'bg-green-600 text-white'
                  : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
              }`}
            >
              <TrendingUp className="w-5 h-5" />
              Long
            </button>
            <button
              type="button"
              onClick={() => setFormData({ ...formData, direction: 'short' })}
              className={`py-3 rounded-lg font-bold flex items-center justify-center gap-2 transition ${
                formData.direction === 'short'
                  ? 'bg-red-600 text-white'
                  : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
              }`}
            >
              <TrendingDown className="w-5 h-5" />
              Short
            </button>
          </div>
        </div>
        
        {/* Perp Market */}
        <div className="mb-8">
          <label className="block text-sm font-medium mb-2">Underlying Asset</label>
          <select
            value={formData.perpMarket}
            onChange={(e) => setFormData({ ...formData, perpMarket: Number(e.target.value) })}
            className="w-full px-4 py-3 bg-gray-900 rounded-lg border border-gray-700 focus:border-purple-500 focus:outline-none"
          >
            <option value={0}>SOL-PERP</option>
            <option value={1}>BTC-PERP</option>
            <option value={2}>ETH-PERP</option>
          </select>
        </div>
        
        {/* Info Box */}
        <div className="mb-6 p-4 bg-blue-900/30 rounded-lg border border-blue-500/30">
          <div className="flex items-start gap-3">
            <Info className="w-5 h-5 text-blue-400 mt-0.5" />
            <div className="text-sm text-blue-200">
              <p className="font-medium mb-1">How it works:</p>
              <ul className="list-disc list-inside space-y-1">
                <li>Deploy fee: 0.1 SOL</li>
                <li>Token price moves with {formData.leverage}x leverage on {formData.perpMarket === 0 ? 'SOL' : formData.perpMarket === 1 ? 'BTC' : 'ETH'}</li>
                <li>Graduates to Raydium at $69k market cap</li>
                <li>Risk: Liquidation if perp moves against you</li>
              </ul>
            </div>
          </div>
        </div>
        
        {/* Submit */}
        <button
          type="submit"
          disabled={loading || !wallet.publicKey}
          className="w-full py-4 bg-gradient-to-r from-purple-600 to-pink-600 rounded-lg font-bold text-lg hover:from-purple-700 hover:to-pink-700 transition disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
        >
          {loading ? (
            <div className="w-6 h-6 border-2 border-white/30 border-t-white rounded-full animate-spin" />
          ) : (
            <>
              <Rocket className="w-5 h-5" />
              Launch Token
            </>
          )}
        </button>
      </form>
    </div>
  )
}
