import { useState, useCallback, ReactNode, Component, ErrorInfo } from 'react'
import { createClient, autoDiscover } from '@solana/client'
import { SolanaProvider, useWalletConnection } from '@solana/react-hooks'
import { WalletButton, BalanceDisplay, DepositSection, WithdrawSection } from './components'
import { getWalletAddress } from './utils'
import { DEVNET_ENDPOINT } from './constants'
import type { DepositNote } from './types'

export type { DepositNote, WithdrawalProof } from './types'

interface ErrorBoundaryState {
  hasError: boolean
  error: Error | null
}

class ErrorBoundary extends Component<{ children: ReactNode }, ErrorBoundaryState> {
  constructor(props: { children: ReactNode }) {
    super(props)
    this.state = { hasError: false, error: null }
  }

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { hasError: true, error }
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error('[ErrorBoundary] Caught error:', error, errorInfo)
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="min-h-screen bg-[#0a0b0d] flex items-center justify-center p-4">
          <div className="card p-8 max-w-md text-center">
            <div className="w-12 h-12 rounded-full bg-red-500/10 flex items-center justify-center mx-auto mb-4">
              <svg className="w-6 h-6 text-red-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
              </svg>
            </div>
            <h2 className="text-xl font-semibold text-white mb-2">Something went wrong</h2>
            <p className="text-[#8b8d94] mb-6">
              {this.state.error?.message || 'An unexpected error occurred'}
            </p>
            <button
              onClick={() => window.location.reload()}
              className="btn btn-primary"
            >
              Reload Page
            </button>
          </div>
        </div>
      )
    }

    return this.props.children
  }
}

const client = createClient({
  endpoint: DEVNET_ENDPOINT,
  walletConnectors: autoDiscover(),
})

function AppProviders({ children }: { children: ReactNode }) {
  return (
    <SolanaProvider client={client}>
      {children}
    </SolanaProvider>
  )
}

function MainApp() {
  const { wallet } = useWalletConnection()
  const [depositNote, setDepositNote] = useState<DepositNote | null>(null)

  const walletAddress = getWalletAddress(wallet)

  const handleDepositComplete = useCallback((note: DepositNote) => {
    setDepositNote(note)
  }, [])

  const handleClearNote = useCallback(() => {
    setDepositNote(null)
  }, [])

  return (
    <div className="min-h-screen bg-[#0a0b0d] bg-pattern">
      {/* Header */}
      <header className="border-b border-[#232529]">
        <div className="max-w-4xl mx-auto px-6 py-4 flex justify-between items-center">
          <div className="flex items-center gap-3">
            <div className="w-8 h-8 rounded-lg bg-gradient-to-br from-[#9945FF] to-[#14F195] flex items-center justify-center">
              <svg className="w-4 h-4 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2.5} d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
              </svg>
            </div>
            <h1 className="text-xl font-semibold text-white">Private Transfers</h1>
          </div>
          <WalletButton />
        </div>
      </header>

      {/* Main Content */}
      <main className="max-w-4xl mx-auto px-6 py-10">
        {!walletAddress ? (
          <div className="card p-12 text-center animate-fade-in">
            <div className="w-16 h-16 rounded-full bg-[#9945FF]/10 flex items-center justify-center mx-auto mb-6">
              <svg className="w-8 h-8 text-[#9945FF]" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M17 9V7a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2m2 4h10a2 2 0 002-2v-6a2 2 0 00-2-2H9a2 2 0 00-2 2v6a2 2 0 002 2zm7-5a2 2 0 11-4 0 2 2 0 014 0z" />
              </svg>
            </div>
            <h2 className="text-2xl font-semibold text-white mb-3">Connect Your Wallet</h2>
            <p className="text-[#8b8d94] mb-8 max-w-sm mx-auto">
              Connect a Solana wallet to make private deposits and withdrawals
            </p>
            <WalletButton />
          </div>
        ) : (
          <div className="space-y-6 animate-fade-in">
            <BalanceDisplay />

            <div className="grid lg:grid-cols-2 gap-6">
              <DepositSection
                onDepositComplete={handleDepositComplete}
                depositNote={depositNote}
                onClearNote={handleClearNote}
              />
              <WithdrawSection />
            </div>
          </div>
        )}
      </main>

      {/* Footer */}
      <footer className="border-t border-[#232529] mt-auto">
        <div className="max-w-4xl mx-auto px-6 py-4 flex justify-between items-center">
          <span className="text-[#5c5e66] text-sm">Devnet</span>
          <div className="flex items-center gap-1 text-[#5c5e66] text-sm">
            <span>Powered by</span>
            <span className="gradient-text font-medium">Noir + Solana</span>
          </div>
        </div>
      </footer>
    </div>
  )
}

function App() {
  return (
    <ErrorBoundary>
      <AppProviders>
        <MainApp />
      </AppProviders>
    </ErrorBoundary>
  )
}

export default App
