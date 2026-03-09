import { Address } from '@solana/kit'
import { useWalletConnection } from '@solana/react-hooks'

export function getWalletAddress(
  wallet: ReturnType<typeof useWalletConnection>['wallet']
): Address | null {
  if (!wallet?.account?.address) return null
  return wallet.account.address as Address
}

export function hexToBytes(hex: string): Uint8Array {
  const cleanHex = hex.startsWith('0x') ? hex.slice(2) : hex
  const bytes = new Uint8Array(cleanHex.length / 2)
  for (let i = 0; i < bytes.length; i++) {
    bytes[i] = parseInt(cleanHex.substring(i * 2, i * 2 + 2), 16)
  }
  return bytes
}

export function formatSol(lamports: bigint | number): string {
  const value = typeof lamports === 'bigint' ? lamports : BigInt(lamports)
  return (Number(value) / 1_000_000_000).toFixed(4)
}

export function truncateAddress(address: string, chars = 4): string {
  return `${address.slice(0, chars)}...${address.slice(-chars)}`
}
