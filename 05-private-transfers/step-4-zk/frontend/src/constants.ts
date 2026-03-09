import { address } from '@solana/kit'

// Lamports conversion
export const LAMPORTS_PER_SOL = 1_000_000_000n

// Program IDs - Devnet deployment
export const SUNSPOT_VERIFIER_ID = address('DCybqMyoiEjRoBsLEXyLcfxJ88sDCEYVCZ7nsKFYy6EZ')

// Backend API URL - configurable via environment variable for production
export const API_URL = import.meta.env.VITE_API_URL || 'http://localhost:4001'

// Devnet endpoint
export const DEVNET_ENDPOINT = 'https://api.devnet.solana.com'

// System program
export const SYSTEM_PROGRAM_ID = address('11111111111111111111111111111111')

// Compute budget program
export const COMPUTE_BUDGET_PROGRAM_ID = address('ComputeBudget111111111111111111111111111111')

// Compute units required for ZK proof verification via Sunspot
// Groth16 verification on Solana requires significant compute
// This value provides headroom for the verifier CPI call
export const ZK_VERIFY_COMPUTE_UNITS = 1_400_000

// Default deposit amount in SOL (for UI)
export const DEFAULT_DEPOSIT_AMOUNT = '0.1'

// Minimum deposit amount in SOL
export const MIN_DEPOSIT_SOL = 0.001

// PDA Seeds (as readable constants)
export const SEEDS = {
  POOL: new Uint8Array([112, 111, 111, 108]), // "pool"
  VAULT: new Uint8Array([118, 97, 117, 108, 116]), // "vault"
  NULLIFIERS: new Uint8Array([110, 117, 108, 108, 105, 102, 105, 101, 114, 115]), // "nullifiers"
} as const
