**~12 min**

# Step 4.4: Frontend and Demo

## Goal

Understand how the frontend builds and sends withdraw transactions using Solana Kit.

---

## What You'll Do

* Generate TypeScript client with Codama
* Walk through the withdraw transaction code
* Run and test the complete flow

---

## Generate Client with Codama

Codama generates TypeScript interfaces from your Anchor IDL. This gives you type-safe encoders that handle serialization automatically.

```bash
cd frontend
bun run scripts/generate-client.ts
```

This creates the `generated/` folder with:
- Instruction encoders (handles the 8-byte discriminator + field serialization)
- Account decoders
- Type definitions

---

## Update Constants

In `frontend/src/constants.ts`, set your verifier program ID:

```typescript
export const SUNSPOT_VERIFIER_ID = address("YOUR_VERIFIER_PROGRAM_ID_HERE")
```

In `backend/src/server.ts` set the private transfers program ID

This is automaticaly done for us on the frotnend with codama

---

## The Withdraw Transaction

Let's walk through `frontend/src/components/WithdrawSection.tsx` line by line.

**File:** `frontend/src/components/WithdrawSection.tsx`

### Imports - What Each Kit Function Does

```typescript
import { useWalletConnection, useSendTransaction } from '@solana/react-hooks'
// useWalletConnection - React hook that gives you the connected wallet
// useSendTransaction - React hook that handles signing + sending transactions

import { address, getProgramDerivedAddress, getBytesEncoder, getAddressEncoder } from '@solana/kit'
// address() - Converts a base58 string to a Solana Address type
// getProgramDerivedAddress() - Derives a PDA from seeds + program ID
// getBytesEncoder() - Creates an encoder that converts strings/Uint8Arrays to bytes
// getAddressEncoder() - Creates an encoder that converts Address to 32 bytes

import { getWithdrawInstructionDataEncoder, PRIVATE_TRANSFERS_PROGRAM_ADDRESS } from '../generated'
// getWithdrawInstructionDataEncoder() - Codama-generated encoder for our withdraw instruction
// PRIVATE_TRANSFERS_PROGRAM_ADDRESS - Our program's address from the IDL
```

### Preparing the Proof Data

```typescript
// Backend generates the ZK proof and returns all the data we need
const { withdrawalProof }: WithdrawApiResponse = await response.json()

// Convert the proof from number[] to Uint8Array (Solana's preferred byte format)
const proof = new Uint8Array(withdrawalProof.proof)

// Convert hex strings to byte arrays using our utility function
// hexToBytes("0x1234...") -> Uint8Array([0x12, 0x34, ...])
const nullifierHash = hexToBytes(withdrawalProof.nullifierHash)
const root = hexToBytes(withdrawalProof.merkleRoot)

// address() validates and converts the base58 string to Kit's Address type
// This ensures the recipient is a valid Solana address before we build the tx
const recipientAddress = address(withdrawalProof.recipient)

// JavaScript numbers lose precision above 2^53, so we use BigInt for u64 values
// Solana amounts are in lamports (1 SOL = 1_000_000_000 lamports)
const amountBN = BigInt(withdrawalProof.amount)
```

### Deriving PDAs

```typescript
// getProgramDerivedAddress finds the deterministic address for a PDA
// Returns [address, bump] - we only need the address here
const [poolPda] = await getProgramDerivedAddress({
  programAddress,  // The program that owns this PDA
  seeds: [getBytesEncoder().encode(SEEDS.POOL)],  // "pool" as bytes
})

// NullifierSet PDA uses two seeds: "nullifiers" + the pool's address
// This links the nullifier set to a specific pool
const [nullifierSetPda] = await getProgramDerivedAddress({
  programAddress,
  seeds: [
    getBytesEncoder().encode(SEEDS.NULLIFIERS),  // "nullifiers" as bytes
    getAddressEncoder().encode(poolPda),          // Pool address as 32 bytes
  ],
})

// Vault PDA - holds the actual SOL
const [poolVaultPda] = await getProgramDerivedAddress({
  programAddress,
  seeds: [
    getBytesEncoder().encode(SEEDS.VAULT),
    getAddressEncoder().encode(poolPda),
  ],
})
```

### Encoding Instruction Data

```typescript
// Codama generates this encoder from your IDL
// It knows: 8-byte discriminator + exact field order + byte sizes
const withdrawDataEncoder = getWithdrawInstructionDataEncoder()

// encode() serializes all fields into a single Uint8Array
// The encoder handles: discriminator, proof bytes, hashes, address, amount
const instructionData = withdrawDataEncoder.encode({
  proof,              // Vec<u8> - variable length, prefixed with length
  nullifierHash,      // [u8; 32] - exactly 32 bytes
  root,               // [u8; 32] - exactly 32 bytes  
  recipient: recipientAddress,  // Pubkey - 32 bytes
  amount: amountBN,   // u64 - 8 bytes, little-endian
})
```

### Building the Instruction

```typescript
// An instruction tells Solana: which program, which accounts, what data
const withdrawInstruction = {
  programAddress,  // The program to call
  
  // Account roles:
  // 0 = readonly     - program can read but not modify
  // 1 = writable     - program can modify the account
  // 2 = readonly + signer  - must sign, but not modified
  // 3 = writable + signer  - must sign AND can be modified
  //
  // ORDER MATTERS! Must match the Withdraw struct in your Anchor program
  accounts: [
    { address: poolPda, role: 1 },          // pool: writable (update root history)
    { address: nullifierSetPda, role: 1 },  // nullifier_set: writable (mark nullifier used)
    { address: poolVaultPda, role: 1 },     // pool_vault: writable (sends SOL)
    { address: recipientAddress, role: 1 }, // recipient: writable (receives SOL)
    { address: SUNSPOT_VERIFIER_ID, role: 0 }, // verifier_program: readonly (CPI target)
    { address: SYSTEM_PROGRAM_ID, role: 0 },   // system_program: readonly
  ],
  data: instructionData,
}
```

### Setting Compute Budget

```typescript
// ZK verification uses ~1.4M compute units, but default budget is 200K
// We need to request more compute units or the transaction will fail
const computeBudgetData = new Uint8Array(5)
computeBudgetData[0] = 2  // Instruction index for SetComputeUnitLimit
new DataView(computeBudgetData.buffer).setUint32(1, ZK_VERIFY_COMPUTE_UNITS, true)
// true = little-endian (Solana's byte order)

const computeBudgetInstruction = {
  programAddress: COMPUTE_BUDGET_PROGRAM_ID,  // Native Solana program
  accounts: [] as const,  // No accounts needed
  data: computeBudgetData,
}
```

### Sending the Transaction

```typescript
// sendTransaction from useSendTransaction hook handles:
// 1. Getting a recent blockhash
// 2. Building the transaction message
// 3. Requesting wallet signature
// 4. Submitting to the network
// 5. Waiting for confirmation
const result = await sendTransaction({
  // Multiple instructions execute atomically - if any fails, all revert
  instructions: [computeBudgetInstruction, withdrawInstruction],
})
```

---

## Run and Test the Demo

### Sync anchor keys

```bash
cd anchor
anchor keys sync
```

### 1. Start the Backend

The backend handles proof generation (runs Noir prover server-side).

```bash
cd backend
bun install
bun run dev
```

Runs on `http://localhost:4001`.

### 2. Start the Frontend

```bash
cd frontend
bun install
bun run dev
```

Open `http://localhost:3000`.

## Initialize the Pool (First Time Only!)

Before any deposits or withdrawals can happen, the pool must be initialized. This creates the Pool and NullifierSet accounts on-chain.

```bash
cd anchor
anchor run initialize
```

> This only needs to run once per deployment. If you redeploy the program, you'll need to initialize again. The initialize instruction creates PDAs for the pool, vault, and nullifier set.

---

## Test the Complete Flow

### Deposit

1. Connect wallet (devnet)
2. Enter amount (e.g., 0.1 SOL)
3. Click **Deposit**
4. **Save your deposit note!** (contains secret + nullifier needed for withdrawal)

### Withdraw

1. Switch to a different wallet (optional - simulates Alice → Bob)
2. Paste the deposit note
3. Enter recipient address
4. Click **Withdraw**
5. Wait ~30 seconds for proof generation
6. Approve the transaction

---

## Verify Privacy on Explorer

Open [Solana Explorer](https://explorer.solana.com/?cluster=devnet).

**Deposit transaction shows:**
- `commitment: 0x7a3b...` (not your address!)
- `leaf_index: 0`
- `new_root: 0xabc1...`

**Withdrawal transaction shows:**
- `nullifier_hash: 0x9c2f...` (different hash)
- `recipient: Bob's_address`
- NO link to original deposit!

**Why unlinkable:**

```
Deposit:    commitment = Hash(nullifier, secret, amount)
Withdrawal: nullifier_hash = Hash(nullifier)

Different hashes, same nullifier internally - can't be linked without knowing the secret!
```

---

## What You Built

| Component | Purpose |
|-----------|---------|
| Commitment | Hides who deposited |
| Merkle tree | Proves deposit exists |
| Nullifier hash | Prevents double-spending |
| ZK proof | Proves everything privately |
| Verifier CPI | On-chain proof verification |
| Codama | Type-safe client generation |

---

## Congratulations!

You've built a privacy-preserving transfer system on Solana using:
- **Noir** for ZK circuits
- **Sunspot** for Groth16 proofs
- **Anchor** for the Solana program
- **Codama** for client generation
- **Solana Kit** for the frontend

The core concepts - commitments, nullifiers, Merkle trees, ZK proofs - are the foundation of privacy applications like Tornado Cash and Zcash, now running on Solana.
