**~6 min**

# Step 1: Hiding Deposits

## Goal

Update the deposit function to accept a commitment instead of storing the depositor's address. Instead of storing the deposit, we store a commitment, and then the person shows that they know the commitment.

---

## Update the Program

## Main thing we need to do

Change the depositor struct to remove account and store commitment
Remove mentions to account in deposit event

**File:** `anchor/programs/private_transfers/src/lib.rs`

### 1. Update the deposit function signature

Find:

```rust
    pub fn deposit(
        ctx: Context<Deposit>,
        // Step 1: Add commitment: [u8; 32]
        // Step 2: Add new_root: [u8; 32]
        amount: u64,
    ) -> Result<()> {
```

Replace with:

```rust
    pub fn deposit(
        ctx: Context<Deposit>,
        commitment: [u8; 32],
        amount: u64,
    ) -> Result<()> {
```

> **What is `[u8; 32]`?**
> 
> This is Rust's syntax for a fixed-size array of 32 bytes. The commitment is a hash, which outputs 256 bits (32 bytes × 8 bits = 256 bits). The backend computes this hash, converts it to bytes, and sends it to the program.

---

### 2. Update the DepositEvent struct

Replace with:

```rust
#[event]
pub struct DepositEvent {
    pub commitment: [u8; 32],
    pub amount: u64,
    pub timestamp: i64,
}
```

---

### 3. Update the emit! call


Replace with:

```rust
        emit!(DepositEvent {
            commitment,
            amount,
            timestamp: Clock::get()?.unix_timestamp,
        });
```

---

### 4. Update the log message

Find:

```rust
        msg!("Public deposit: {} lamports from {}", amount, ctx.accounts.depositor.key());
```

Replace with:

```rust
        msg!("Deposit: {} lamports, commitment: {:?}", amount);
```

---


## How the Commitment Gets to Solana

Let's see how we compute this commitment

### 1. Backend computes the Poseidon hash

In `backend/src/server.ts`, we use a JavaScript Poseidon2 library:

```typescript
import { poseidon2Hash } from "@zkpassport/poseidon2";

// Generate random secrets
const nullifier = generateRandomField();  // Random 256-bit number
const secret = generateRandomField();     // Another random 256-bit number

// Compute commitment = Poseidon2(nullifier, secret, amount)
const commitment = poseidon2Hash([nullifier, secret, amount]);
const commitmentHex = "0x" + commitment.toString(16).padStart(64, "0");
```

**Why Poseidon?** Regular hash functions like SHA-256 require millions of constraints to prove in a ZK circuit. Poseidon was designed to be "ZK-friendly" - it achieves the same security with far fewer constraints, making proofs faster and cheaper.

Poseidon is becoming a standard in the Solana ecosystem for more than just privacy. Light Protocol uses Poseidon hashes for their compressed state - not for privacy, but for scalability. They hash account data off-chain, store just the hash on-chain, and use ZK proofs to verify state transitions. Same cryptographic primitives, different use case.

### 2. Convert to bytes for Solana

Scroll down to `api/deposit` when this is acutally called. Poseidon hash is a BigInt. Solana expects a byte array:

```typescript
// Strip "0x" prefix, convert hex string to bytes
const commitmentBytes = Array.from(
  Buffer.from(commitmentHex.slice(2), "hex")
);
// Result: [122, 59, ...] - 32 numbers, each 0-255
```

### 3. Frontend sends to Solana

The frontend receives these bytes from the backend, then uses Solana Kit to build and send the transaction. We'll cover the full frontend code in the demo step - for now, just know that Kit's `sendTransaction` handles wallet signing and submission.

---

## What Changed


The depositor's wallet still signs the transaction, but the on-chain event no longer links their identity to this deposit.

---

## Next Step

We've hidden the deposit. But how do we prove our deposit exists without revealing which one? We need Merkle trees.

Continue to [Step 2: Proving Membership](./step-2-proving-membership.md).
