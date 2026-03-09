# Step 7: Frontend and Demo

## Goal

Generate TypeScript clients with Codama and run the complete demo.

---

## What You'll Do

| Task | Purpose |
|------|---------|
| Generate client with Codama | Type-safe instruction encoders |
| Understand the generated code | How to use encoders and PDAs |
| Run the demo | Test the complete privacy flow |

---

## Generate Client with Codama

Codama generates TypeScript code from your Anchor IDL.

### Look at the Codama script

**File:** `frontend/scripts/generate-client.ts`

```typescript
import { createFromRoot } from 'codama'
import { renderVisitor } from '@codama/renderers-js'
import { rootNodeFromAnchor } from '@codama/nodes-from-anchor'
import { readFileSync } from 'fs'
import { join, dirname } from 'path'
import { fileURLToPath } from 'url'

const __dirname = dirname(fileURLToPath(import.meta.url))

// Read the Anchor IDL
const idlPath = join(__dirname, '../../anchor/target/idl/private_transfers.json')
const idl = JSON.parse(readFileSync(idlPath, 'utf-8'))

// Create Codama tree from Anchor IDL
const codama = createFromRoot(rootNodeFromAnchor(idl))

// Generate Kit-compatible TypeScript client
const outputDir = join(__dirname, '../src/generated')
codama.accept(renderVisitor(outputDir, { formatCode: true }))
```

### Generate the client

```bash
cd frontend
bun run scripts/generate-client.ts
```

This creates `src/generated/` with:
- `instructions/deposit.ts` - Deposit instruction encoder
- `instructions/withdraw.ts` - Withdraw instruction encoder
- `accounts/pool.ts` - Pool account decoder
- `types/` - All custom types

---

## Using the Generated Code

### Import instruction encoders

**File:** `frontend/src/generated/instructions/deposit.ts`

```typescript
import {
  getDepositInstructionDataEncoder,
  getWithdrawInstructionDataEncoder
} from './generated';
```

### Encode deposit instruction data

```typescript
const encoder = getDepositInstructionDataEncoder();
const data = encoder.encode({
  commitment: new Uint8Array(commitment),
  newRoot: new Uint8Array(newRoot),
  amount: BigInt(amount),
});
```

### Encode withdraw instruction data

```typescript
const encoder = getWithdrawInstructionDataEncoder();
const data = encoder.encode({
  proof: new Uint8Array(proof),
  nullifierHash: new Uint8Array(nullifierHash),
  root: new Uint8Array(root),
  recipient: recipientAddress,
  amount: BigInt(amount),
});
```

---

## Computing PDAs with Solana Kit

**File:** `frontend/src/components/DepositSection.tsx`

```typescript
import { getProgramDerivedAddress, getUtf8Encoder, getAddressEncoder } from '@solana/kit';

const PROGRAM_ID = 'YOUR_PROGRAM_ID';

// Pool PDA: seeds = [b"pool"]
const [poolPda] = await getProgramDerivedAddress({
  programAddress: PROGRAM_ID,
  seeds: [getUtf8Encoder().encode('pool')],
});

// Vault PDA: seeds = [b"vault", pool]
const [vaultPda] = await getProgramDerivedAddress({
  programAddress: PROGRAM_ID,
  seeds: [
    getUtf8Encoder().encode('vault'),
    getAddressEncoder().encode(poolPda),
  ],
});

// Nullifier set PDA: seeds = [b"nullifiers", pool]
const [nullifierSetPda] = await getProgramDerivedAddress({
  programAddress: PROGRAM_ID,
  seeds: [
    getUtf8Encoder().encode('nullifiers'),
    getAddressEncoder().encode(poolPda),
  ],
});
```

---

## Building Instructions

```typescript
// Account roles: 0=readonly, 1=writable, 2=signer, 3=writable+signer
const depositInstruction = {
  programAddress: PROGRAM_ID,
  accounts: [
    { address: poolPda, role: 1 },           // writable
    { address: vaultPda, role: 1 },          // writable
    { address: walletAddress, role: 3 },     // writable + signer
    { address: SYSTEM_PROGRAM_ID, role: 0 }, // readonly
  ],
  data: instructionData,
};
```

---

## Compute Budget for Withdrawals

ZK verification needs extra compute units:

```typescript
import { getSetComputeUnitLimitInstruction } from '@solana-program/compute-budget';

const computeBudgetIx = getSetComputeUnitLimitInstruction({
  units: 1_400_000,
});

// Include FIRST in transaction
const transaction = {
  instructions: [computeBudgetIx, withdrawInstruction],
};
```

---

## Run the Demo

### 1. Start the Backend

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

Open `http://localhost:5173`.

---

## Test the Complete Flow

### Deposit

1. Connect wallet (devnet)
2. Enter amount (e.g., 0.1 SOL)
3. Click **Deposit**
4. **Save your deposit note!**

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

Different hashes, same nullifier - can't be linked!
```

---

## Regenerating Client Code

After changing the Anchor program:

```bash
cd anchor
anchor build

cd ../frontend
bun run scripts/generate-client.ts
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
