import { Routes, Route } from 'react-router-dom'
import { Toaster } from 'react-hot-toast'
import Navbar from './components/Navbar'
import Home from './pages/Home'
import Launch from './pages/Launch'
import Token from './pages/Token'
import Portfolio from './pages/Portfolio'

function App() {
  return (
    <div className="min-h-screen bg-gradient-to-br from-gray-900 via-purple-900 to-gray-900 text-white">
      <Navbar />
      <main className="container mx-auto px-4 py-8">
        <Routes>
          <Route path="/" element={<Home />} />
          <Route path="/launch" element={<Launch />} />
          <Route path="/token/:mint" element={<Token />} />
          <Route path="/portfolio" element={<Portfolio />} />
        </Routes>
      </main>
      <Toaster position="bottom-right" />
    </div>
  )
}

export default App
