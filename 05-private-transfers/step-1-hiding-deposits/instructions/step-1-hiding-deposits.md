# Step 1: Hiding Deposits

## Goal

Update the deposit function to accept a commitment instead of storing the depositor's address.

---

## What You'll Change

| Before | After |
|--------|-------|
| `deposit(amount)` | `deposit(commitment, amount)` |
| DepositEvent stores `depositor` | DepositEvent stores `commitment` |

---

## Update the Program

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

**Why `[u8; 32]`?** Poseidon2 outputs a 256-bit (32-byte) hash.

---

### 2. Update the DepositEvent struct

Find:

```rust
pub struct DepositEvent {
    pub depositor: Pubkey, // Step 1: Change to commitment: [u8; 32]
    pub amount: u64,
    pub timestamp: i64,
    // Step 2: Add leaf_index: u64, new_root: [u8; 32]
}
```

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

Find:

```rust
        emit!(DepositEvent {
            depositor: ctx.accounts.depositor.key(), // Step 1: Change to commitment
            amount,
            timestamp: Clock::get()?.unix_timestamp,
            // Step 2: Add leaf_index, new_root
        });
```

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
        msg!("Deposit: {} lamports, commitment: {:?}", amount, commitment);
```

---

## Build

```bash
cd anchor
anchor build
```

If the build succeeds, your program now accepts commitments.

---

## What Changed

**Before:** `DepositEvent { depositor: "Alice_pubkey", amount: 1000000 }`

**After:** `DepositEvent { commitment: "0x7a3b...", amount: 1000000 }`

The depositor's wallet still signs the transaction, but the on-chain event no longer links their identity to this deposit.

---

## Next Step

We've hidden the deposit. But how do we prove our deposit exists without revealing which one? We need Merkle trees.

Continue to [Step 2: Proving Membership](./step-2-proving-membership.md).
