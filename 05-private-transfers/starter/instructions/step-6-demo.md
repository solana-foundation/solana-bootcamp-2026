# Step 6: Demo

## Goal

Run the complete privacy flow end-to-end.

---

## Start the Application

### 1. Start the Backend

**File:** `backend/src/server.ts`

The backend handles cryptographic operations (hashing, proof generation).

```bash
cd backend
bun install
bun run dev
```

Runs on `http://localhost:4001`.

### 2. Start the Frontend

**File:** `frontend/src/App.tsx`

```bash
cd frontend
bun install
bun run dev
```

Open `http://localhost:3000` in your browser.

---

## The Complete Flow

### Part 1: Connect Wallet

1. Click **Connect Wallet** in the top right
2. Make sure you're on **devnet**
3. Get some devnet SOL from a faucet if needed

---

### Part 2: Deposit

**What you do:** Enter an amount (e.g., 0.1 SOL) and click **Deposit**

**What happens behind the scenes:**

```
FRONTEND (frontend/src/components/DepositSection.tsx):
1. User enters amount
2. Frontend calls POST /api/deposit with amount

BACKEND (backend/src/server.ts):
3. Generate random nullifier + secret
4. Compute commitment = Poseidon2(nullifier, secret, amount)
5. Compute new Merkle root
6. Return deposit note + on-chain data

FRONTEND (continued):
7. Compute PDAs (pool, vault)
8. Encode instruction data
9. Build transaction
10. User signs with wallet

SOLANA PROGRAM (anchor/programs/private_transfers/src/lib.rs):
11. Transfer SOL to vault
12. Store new root in history
13. Emit DepositEvent with commitment (NOT your address!)
```

**Save your deposit note!** You need it to withdraw. If you lose it, your funds are gone forever.

---

### Part 3: Switch Wallets

Switch to a **different wallet** in Phantom (or your wallet extension).

This simulates Alice depositing and Bob withdrawing.

---

### Part 4: Withdraw

**What you do:**
1. Paste the deposit note
2. Enter recipient address (or leave as current wallet)
3. Click **Withdraw**
4. Wait ~30 seconds for proof generation
5. Approve the transaction

**What happens behind the scenes:**

```
FRONTEND (frontend/src/components/WithdrawSection.tsx):
1. Parse deposit note
2. Call POST /api/withdraw with deposit note + recipient

BACKEND (backend/src/server.ts):
3. Compute Merkle proof path using leafIndex
4. Write inputs to Prover.toml
5. Run `nargo execute` → generate witness (~5 sec)
6. Run `sunspot prove` → generate 256-byte proof (~25 sec)
7. Return proof + public inputs

FRONTEND (continued):
8. Compute PDAs (pool, nullifier_set, vault, verifier)
9. Encode instruction data (proof, nullifier_hash, root, recipient, amount)
10. Build ComputeBudget instruction (1.4M units)
11. Build withdraw instruction
12. User signs with wallet

SOLANA PROGRAM (anchor/programs/private_transfers/src/lib.rs):
13. Check nullifier_hash not used
14. Check root exists in history
15. Call verifier via CPI → verify proof
16. Mark nullifier_hash as used
17. Transfer SOL to recipient
```

---

### Part 5: Verify on Explorer

Open [Solana Explorer](https://explorer.solana.com/?cluster=devnet) and look at both transactions:

**Deposit transaction shows:**
- `commitment: 0x7a3b...` (hides who deposited)
- `leaf_index: 0`
- `new_root: 0xabc1...`
- Your wallet signed but is NOT in the event!

**Withdrawal transaction shows:**
- `nullifier_hash: 0x9c2f...` (different hash)
- `recipient: Bob's_address`
- NO reference to original commitment or depositor!

**Why these can't be linked:**
```
At deposit:    commitment = Hash(nullifier, secret, amount)
At withdrawal: nullifier_hash = Hash(nullifier)

Both use the same nullifier, but:
- Can't compute commitment from nullifier_hash (missing secret + amount)
- Can't compute nullifier_hash from commitment (hash is one-way)
- Observer sees two unrelated 256-bit values
```

---

## Understanding the Frontend Code

### Client Setup

**File:** `frontend/src/App.tsx`

```typescript
import { createSolanaRpc } from '@solana/kit'

// Create RPC connection
const rpc = createSolanaRpc('https://api.devnet.solana.com')
```

### Computing PDAs

**File:** `frontend/src/components/DepositSection.tsx`

```typescript
import { getProgramDerivedAddress, getBytesEncoder, getAddressEncoder } from '@solana/kit'

// Pool PDA: seeds = [b"pool"]
const [poolPda] = await getProgramDerivedAddress({
  programAddress,
  seeds: [getBytesEncoder().encode(SEEDS.POOL)],
})

// Vault PDA: seeds = [b"vault", pool.key().as_ref()]
const [poolVaultPda] = await getProgramDerivedAddress({
  programAddress,
  seeds: [
    getBytesEncoder().encode(SEEDS.VAULT),
    getAddressEncoder().encode(poolPda),
  ],
})
```

### Using Generated Code (Codama)

**File:** `frontend/src/generated/instructions/deposit.ts`

Codama generates type-safe encoders from your Anchor IDL:

```typescript
import { getDepositInstructionDataEncoder } from '../generated'

const dataEncoder = getDepositInstructionDataEncoder()
const instructionData = dataEncoder.encode({
  commitment: new Uint8Array(onChainData.commitment),
  newRoot: new Uint8Array(onChainData.newRoot),
  amount: BigInt(onChainData.amount),
})
```

To regenerate after IDL changes:
```bash
cd frontend
bun run generate
```

### Building Instructions

**File:** `frontend/src/components/DepositSection.tsx`

```typescript
const depositInstruction = {
  programAddress,
  accounts: [
    { address: poolPda, role: 1 },        // writable
    { address: poolVaultPda, role: 1 },   // writable
    { address: walletAddress, role: 3 },  // writable + signer
    { address: SYSTEM_PROGRAM_ID, role: 0 }, // readonly
  ],
  data: instructionData,
}
```

Account roles: 0 = readonly, 1 = writable, 2 = signer, 3 = writable + signer

### Compute Budget for Withdrawals

**File:** `frontend/src/components/WithdrawSection.tsx`

ZK verification needs extra compute units:

```typescript
import { getSetComputeUnitLimitInstruction } from '@solana-program/compute-budget'

const computeBudgetInstruction = getSetComputeUnitLimitInstruction({
  units: 1_400_000
})

// Send BOTH instructions
await sendTransaction({
  instructions: [computeBudgetInstruction, withdrawInstruction],
})
```

---

## What You Built

| Component | What it does |
|-----------|--------------|
| Commitment | Hides who deposited |
| Merkle tree | Proves deposit exists without revealing which one |
| Nullifier hash | Prevents double-spending |
| ZK proof | Proves everything without revealing secrets |
| Verifier CPI | On-chain proof verification |

---

## FAQs

**Q: Why does proof generation take 30 seconds?**
Groth16 proving is computationally intensive. The prover does heavy work so verification is fast.

**Q: What if I lose my deposit note?**
Funds are lost forever. There's no recovery mechanism.

**Q: Can the pool operator steal funds?**
No. Funds can only move with a valid ZK proof. Only the depositor has the secrets.

**Q: How much does a withdrawal cost?**
About 1.4M compute units - a few extra cents in fees.

---

## Congratulations!

You've built a privacy-preserving transfer system on Solana.

The core concepts - commitments, nullifiers, Merkle trees, ZK proofs - are the foundation of privacy applications like Tornado Cash and Zcash, now running on Solana.
