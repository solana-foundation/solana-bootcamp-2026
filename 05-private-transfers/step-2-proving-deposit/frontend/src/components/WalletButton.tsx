import { useState } from 'react'
import { useWalletConnection } from '@solana/react-hooks'
import { getWalletAddress, truncateAddress } from '../utils'

export function WalletButton() {
  const { connectors, connect, disconnect, connecting, wallet } = useWalletConnection()
  const [showDropdown, setShowDropdown] = useState(false)

  const walletAddress = getWalletAddress(wallet)

  if (walletAddress) {
    return (
      <div className="flex items-center gap-3">
        <div className="flex items-center gap-2 px-3 py-2 rounded-lg bg-[#111214] border border-[#232529]">
          <div className="w-2 h-2 rounded-full bg-[#14F195]" />
          <span className="text-sm text-[#8b8d94] font-mono">
            {truncateAddress(walletAddress)}
          </span>
        </div>
        <button
          onClick={() => disconnect()}
          className="btn btn-ghost text-sm"
        >
          Disconnect
        </button>
      </div>
    )
  }

  return (
    <div className="relative">
      <button
        onClick={() => setShowDropdown(!showDropdown)}
        disabled={connecting}
        className="btn btn-primary"
      >
        {connecting ? (
          <span className="flex items-center gap-2">
            <svg className="animate-spin w-4 h-4" fill="none" viewBox="0 0 24 24">
              <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
              <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
            </svg>
            Connecting...
          </span>
        ) : (
          'Connect Wallet'
        )}
      </button>

      {showDropdown && connectors.length > 0 && (
        <>
          <div
            className="fixed inset-0 z-10"
            onClick={() => setShowDropdown(false)}
          />
          <div className="absolute right-0 mt-2 w-52 bg-[#111214] border border-[#232529] rounded-xl shadow-2xl z-20 overflow-hidden animate-fade-in">
            <div className="p-2">
              {connectors.map((connector) => (
                <button
                  key={connector.id}
                  onClick={() => {
                    connect(connector.id)
                    setShowDropdown(false)
                  }}
                  className="flex items-center gap-3 w-full text-left px-3 py-2.5 text-[#f4f4f5] hover:bg-[#18191c] rounded-lg transition-colors"
                >
                  <div className="w-8 h-8 rounded-lg bg-[#18191c] flex items-center justify-center">
                    <svg className="w-4 h-4 text-[#9945FF]" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17 9V7a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2m2 4h10a2 2 0 002-2v-6a2 2 0 00-2-2H9a2 2 0 00-2 2v6a2 2 0 002 2zm7-5a2 2 0 11-4 0 2 2 0 014 0z" />
                    </svg>
                  </div>
                  <span className="text-sm font-medium">{connector.name}</span>
                </button>
              ))}
            </div>
          </div>
        </>
      )}
    </div>
  )
}
