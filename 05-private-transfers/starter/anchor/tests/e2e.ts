/**
 * Full End-to-End Test with Fresh Proof Generation
 *
 * This test uses the backend API to generate fresh ZK proofs,
 * ensuring each test run has a unique nullifier that hasn't been used.
 *
 * Requirements:
 * - Backend server running at http://localhost:4001
 * - Sunspot CLI installed
 * - Circuit compiled with nargo
 */

import * as anchor from '@coral-xyz/anchor'
import { Program, BN } from '@coral-xyz/anchor'
import { PrivateTransfers } from '../target/types/private_transfers'
import { expect } from 'chai'
import { PublicKey, Keypair, SystemProgram, LAMPORTS_PER_SOL, ComputeBudgetProgram } from '@solana/web3.js'

const API_URL = 'http://localhost:4001'
const SUNSPOT_VERIFIER_ID = new PublicKey('CU2Vgym4wiTNcJCuW6r7DV6bCGULJxKdwFjfGfmksSVZ')

interface DepositNote {
  nullifier: string
  secret: string
  amount: string
  commitment: string
  nullifierHash: string
  merkleRoot: string
  leafIndex: number
  timestamp: number
}

interface OnChainData {
  commitment: number[]
  newRoot: number[]
  amount: string
}

interface WithdrawalProof {
  proof: number[]
  publicWitness: number[]
  nullifierHash: string
  merkleRoot: string
  recipient: string
  amount: string
}

describe('Full E2E Test with Fresh Proof Generation', () => {
  const provider = anchor.AnchorProvider.env()
  anchor.setProvider(provider)

  const program = anchor.workspace.PrivateTransfers as Program<PrivateTransfers>

  let poolPda: PublicKey
  let poolVaultPda: PublicKey
  let nullifierSetPda: PublicKey

  before(async () => {
    // Find PDAs
    ;[poolPda] = PublicKey.findProgramAddressSync(
      [Buffer.from('pool')],
      program.programId
    )
    ;[poolVaultPda] = PublicKey.findProgramAddressSync(
      [Buffer.from('vault'), poolPda.toBuffer()],
      program.programId
    )
    ;[nullifierSetPda] = PublicKey.findProgramAddressSync(
      [Buffer.from('nullifiers'), poolPda.toBuffer()],
      program.programId
    )

    console.log('=== E2E Test Setup ===')
    console.log('Pool PDA:', poolPda.toString())
    console.log('Vault PDA:', poolVaultPda.toString())
    console.log('Nullifier Set PDA:', nullifierSetPda.toString())
    console.log('Sunspot Verifier ID:', SUNSPOT_VERIFIER_ID.toString())
    console.log('')

    // Check backend is running
    try {
      const healthRes = await fetch(`${API_URL}/api/health`)
      if (!healthRes.ok) throw new Error('Backend not healthy')
      console.log('Backend server: ✓ Running')
    } catch (e) {
      console.error('ERROR: Backend server not running at', API_URL)
      console.error('Start it with: cd backend && bun run dev')
      throw new Error('Backend server required for E2E tests')
    }
  })

  it('should complete full deposit -> withdraw cycle with fresh ZK proof', async () => {
    const depositAmount = 0.05 * LAMPORTS_PER_SOL // 0.05 SOL

    // ============ STEP 1: Generate fresh deposit note via backend ============
    console.log('\n--- Step 1: Generate deposit note ---')

    const depositRes = await fetch(`${API_URL}/api/deposit`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ amount: depositAmount })
    })

    if (!depositRes.ok) {
      const error = await depositRes.json()
      throw new Error(`Failed to generate deposit: ${error.error}`)
    }

    const { depositNote, onChainData }: { depositNote: DepositNote, onChainData: OnChainData } = await depositRes.json()

    console.log('Generated deposit note:')
    console.log('  Commitment:', depositNote.commitment.slice(0, 20) + '...')
    console.log('  Nullifier hash:', depositNote.nullifierHash.slice(0, 20) + '...')
    console.log('  Merkle root:', depositNote.merkleRoot.slice(0, 20) + '...')
    console.log('  Amount:', depositNote.amount, 'lamports')

    // ============ STEP 2: Submit deposit to blockchain ============
    console.log('\n--- Step 2: Submit deposit transaction ---')

    const commitment = new Uint8Array(onChainData.commitment)
    const newRoot = new Uint8Array(onChainData.newRoot)

    const depositTx = await program.methods
      .deposit(
        Array.from(commitment),
        Array.from(newRoot),
        new BN(depositNote.amount)
      )
      .accounts({
        pool: poolPda,
        poolVault: poolVaultPda,
        depositor: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc()

    console.log('Deposit tx:', depositTx)

    // Verify deposit
    const poolAfterDeposit = await program.account.pool.fetch(poolPda)
    const storedRoot = poolAfterDeposit.roots[Number(poolAfterDeposit.currentRootIndex)]
    expect(Buffer.from(storedRoot).equals(Buffer.from(newRoot))).to.be.true
    console.log('Deposit verified: ✓')

    // ============ STEP 3: Generate fresh ZK proof via backend ============
    console.log('\n--- Step 3: Generate ZK proof ---')

    const recipient = Keypair.generate()
    console.log('Recipient:', recipient.publicKey.toString())

    const withdrawRes = await fetch(`${API_URL}/api/withdraw`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        depositNote,
        recipient: recipient.publicKey.toString()
      })
    })

    if (!withdrawRes.ok) {
      const error = await withdrawRes.json()
      throw new Error(`Failed to generate proof: ${error.error}`)
    }

    const { withdrawalProof }: { withdrawalProof: WithdrawalProof } = await withdrawRes.json()

    console.log('ZK proof generated:')
    console.log('  Proof size:', withdrawalProof.proof.length, 'bytes')
    console.log('  Public witness size:', withdrawalProof.publicWitness.length, 'bytes')

    // ============ STEP 4: Submit withdrawal to blockchain ============
    console.log('\n--- Step 4: Submit withdrawal transaction ---')

    // Parse hex strings to bytes
    const hexToBytes = (hex: string): Uint8Array => {
      const cleanHex = hex.startsWith('0x') ? hex.slice(2) : hex
      const bytes = new Uint8Array(cleanHex.length / 2)
      for (let i = 0; i < bytes.length; i++) {
        bytes[i] = parseInt(cleanHex.substr(i * 2, 2), 16)
      }
      return bytes
    }

    const nullifierHash = hexToBytes(withdrawalProof.nullifierHash)
    const root = hexToBytes(withdrawalProof.merkleRoot)
    const proofBytes = Buffer.from(withdrawalProof.proof)

    console.log('Submitting withdrawal with on-chain ZK verification...')

    const withdrawTx = await program.methods
      .withdraw(
        proofBytes,
        Array.from(nullifierHash),
        Array.from(root),
        recipient.publicKey,
        new BN(withdrawalProof.amount)
      )
      .accounts({
        pool: poolPda,
        nullifierSet: nullifierSetPda,
        poolVault: poolVaultPda,
        recipient: recipient.publicKey,
        verifierProgram: SUNSPOT_VERIFIER_ID,
        systemProgram: SystemProgram.programId,
      })
      .preInstructions([
        ComputeBudgetProgram.setComputeUnitLimit({ units: 1_400_000 })
      ])
      .rpc()

    console.log('Withdrawal tx:', withdrawTx)

    // ============ STEP 5: Verify withdrawal succeeded ============
    console.log('\n--- Step 5: Verify withdrawal ---')

    // Check nullifier is marked as used
    const nullifierSet = await program.account.nullifierSet.fetch(nullifierSetPda)
    const isUsed = nullifierSet.nullifiers.some(
      (n: number[]) => Buffer.from(n).equals(Buffer.from(nullifierHash))
    )
    expect(isUsed).to.be.true
    console.log('Nullifier marked as used: ✓')

    // Check recipient received funds (approximately, accounting for rent)
    const recipientBalance = await provider.connection.getBalance(recipient.publicKey)
    expect(recipientBalance).to.be.greaterThan(0)
    console.log('Recipient balance:', recipientBalance / LAMPORTS_PER_SOL, 'SOL')

    console.log('\n=== E2E TEST PASSED ===')
    console.log('✓ Generated fresh deposit note')
    console.log('✓ Deposited to pool')
    console.log('✓ Generated fresh ZK proof')
    console.log('✓ Verified proof on-chain via Sunspot')
    console.log('✓ Withdrew funds to recipient')
  })
})
