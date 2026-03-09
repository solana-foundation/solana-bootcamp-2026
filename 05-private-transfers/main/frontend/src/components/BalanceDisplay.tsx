import { useState } from 'react'
import { useWalletConnection, useBalance } from '@solana/react-hooks'
import { getWalletAddress, formatSol } from '../utils'
import { API_URL, LAMPORTS_PER_SOL } from '../constants'

export function BalanceDisplay() {
  const { wallet } = useWalletConnection()
  const [airdropStatus, setAirdropStatus] = useState('')
  const [airdropping, setAirdropping] = useState(false)

  const walletAddress = getWalletAddress(wallet)
  const balanceData = useBalance(walletAddress || undefined)

  const requestAirdrop = async () => {
    if (!walletAddress || airdropping) return
    setAirdropping(true)
    setAirdropStatus('Requesting...')
    try {
      const response = await fetch(`${API_URL}/api/airdrop`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ address: walletAddress, amount: Number(LAMPORTS_PER_SOL) })
      })

      if (!response.ok) {
        const error = await response.json()
        throw new Error(error.error || 'Airdrop failed')
      }

      setAirdropStatus('Success!')
      setTimeout(() => setAirdropStatus(''), 2000)
    } catch (e) {
      setAirdropStatus('Failed')
      setTimeout(() => setAirdropStatus(''), 3000)
    } finally {
      setAirdropping(false)
    }
  }

  const displayBalance = balanceData?.lamports != null
    ? formatSol(balanceData.lamports)
    : null

  return (
    <div className="card p-5">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-4">
          <div className="w-12 h-12 rounded-xl bg-gradient-to-br from-[#9945FF]/20 to-[#14F195]/20 flex items-center justify-center">
            <svg className="w-6 h-6 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
          </div>
          <div>
            <p className="text-sm text-[#8b8d94]">Available Balance</p>
            <p className="text-2xl font-semibold text-white">
              {balanceData?.fetching ? (
                <span className="animate-pulse">Loading...</span>
              ) : displayBalance !== null ? (
                <>
                  {displayBalance}
                  <span className="text-base font-normal text-[#5c5e66] ml-2">SOL</span>
                </>
              ) : (
                'N/A'
              )}
            </p>
          </div>
        </div>

        <button
          onClick={requestAirdrop}
          disabled={airdropping}
          className="btn btn-outline text-sm"
        >
          {airdropStatus ? (
            <span className={airdropStatus === 'Failed' ? 'text-red-400' : airdropStatus === 'Success!' ? 'text-[#14F195]' : ''}>
              {airdropStatus}
            </span>
          ) : (
            <>
              <svg className="w-4 h-4 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
              </svg>
              Airdrop
            </>
          )}
        </button>
      </div>
    </div>
  )
}
