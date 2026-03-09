**~8 min**

# Step 3: Preventing Double-Spend

## Goal

Add nullifier tracking to prove ownership of a speific commitment and prevent the same deposit from being withdrawn twice.

---
## Update the Program

**File:** `anchor/programs/private_transfers/src/lib.rs`

---

A nullifier is a unique value derived from a deposit's secret. When you withdraw, you reveal the nullifier (but not the secret). The program records it, so if you try to withdraw again with the same nullifier, it gets rejected.

Each deposit produces exactly one nullifier, and the nullifier can't be linked back to the commitment. So we know "this deposit was spent" without knowing "which deposit was spent."

### 1. Add NullifierSet struct
:

Replace with:

```rust

#[account]  // Anchor macro - marks this struct as a Solana account that can be stored on-chain
#[derive(InitSpace)]  // Anchor macro - auto-calculates how many bytes this struct needs for rent
pub struct NullifierSet {
    // The pool this nullifier set belongs to
    pub pool: Pubkey,
    // List of all nullifiers that have been used (max 256 for this demo)
    #[max_len(256)]  // Anchor macro - tells Anchor the max size of the Vec for space calculation
    pub nullifiers: Vec<[u8; 32]>,  // Vec = dynamic array that can grow. This is a list of 32-byte hashes
}

// Usually in production youd use another Merkle Tree for nullifiers

impl NullifierSet {
    // Check if this nullifier was already used (i.e., deposit already withdrawn)
    pub fn is_nullifier_used(&self, nullifier_hash: &[u8; 32]) -> bool {
        self.nullifiers.contains(nullifier_hash)
    }

    // Record a nullifier as used - called after successful withdrawal
    pub fn mark_nullifier_used(&mut self, nullifier_hash: [u8; 32]) -> Result<()> {
        require!( // AFTER
            self.nullifiers.len() < 256,
            PrivateTransfersError::NullifierSetFull
        );
        self.nullifiers.push(nullifier_hash); 
        Ok(())
    }
}
```
## Solana Deep Dive: Why a Separate Account?

You might wonder - why not just add a `nullifiers` field to the Pool struct? A few reasons:

**Account size limits:** Solana accounts can grow up to 10MB, but you pay rent proportional to size (~6.9 SOL/MB/year). Our NullifierSet with 256 nullifiers is about 8KB. If we needed thousands of nullifiers, we'd want to keep it separate so we could reallocate space independently.

**Separation of concerns:** The Pool tracks tree state (roots, leaf index). The NullifierSet tracks spent deposits. Different data, different access patterns. In production, you might even use a Merkle tree for nullifiers (like Light Protocol does) to support unlimited nullifiers with constant on-chain storage.

**Upgrade flexibility:** If you wanted to change how nullifiers are stored (say, switch to a Merkle tree), you could deploy a new NullifierSet implementation without touching the Pool.

---
---

## Part 2: Set Up NullifierSet During Initialization

We need to create the NullifierSet account when the pool is initialized, and link it to the pool.

### 2. Add NullifierSet to Initialize accounts


Replace with:

```rust
    pub pool: Account<'info, Pool>,

    // PDA that stores all used nullifiers for this pool
    #[account( // THIS IS NEW
        init,
        payer = authority,
        space = 8 + NullifierSet::INIT_SPACE,
        seeds = [b"nullifiers", pool.key().as_ref()],
        bump
    )]
    pub nullifier_set: Account<'info, NullifierSet>, // THIS IS NEW

    #[account(seeds = [b"vault", pool.key().as_ref()], bump)]
    pub pool_vault: SystemAccount<'info>,
```

### 3. Initialize nullifier_set in initialize function

Find:


Replace with:

```rust
        pool.roots[0] = EMPTY_ROOT;

        // Link the nullifier set to this pool
        let nullifier_set = &mut ctx.accounts.nullifier_set;
        nullifier_set.pool = pool.key();

        msg!("Pool initialized");
```

---

Now we have a `NullifierSet` struct to store used nullifiers, and we initialize it when the pool is created. The nullifier set is linked to the pool via a PDA (Program Derived Address). Now we need to actually use it.

---


## Part 3: Update Withdraw to Check and Mark Nullifiers

The withdrawal flow now needs to:
1. Accept a nullifier_hash from the user
2. Check it hasn't been used before
3. Mark it as used before transferring funds (prevents reentrancy)

### 4. Add NullifierSet to Withdraw accounts

Find:

Add
```rust
    // Must be mutable so we can add the nullifier after withdrawal
    #[account(
        mut,  // we're modifying this account (adding a nullifier), so it must be mutable
        seeds = [b"nullifiers", pool.key().as_ref()],  // PDA seeds - derives address from "nullifiers" 
        bump  // uses the bump seed found during PDA derivation (Anchor finds it automatically)
    )]
    pub nullifier_set: Account<'info, NullifierSet>, // solana account containing data in the sahpe of our NullifierSet struct, info = lifetime of transaction

```

### 5. Update withdraw function signature and add nullifier check

Find:

Replace with:

```rust
    pub fn withdraw(
        ctx: Context<Withdraw>,
      
        nullifier_hash: [u8; 32], // NEW
        root: [u8; 32],
        recipient: Pubkey,
        amount: u64,
    ) -> Result<()> {
        let nullifier_set = &mut ctx.accounts.nullifier_set; // NEW

        // Check if this nullifier was already used (double-spend attempt)
        require!( // NEW
            !nullifier_set.is_nullifier_used(&nullifier_hash),
            PrivateTransfersError::NullifierUsed
        );

        require!(
            ctx.accounts.pool.is_known_root(&root),
            PrivateTransfersError::InvalidRoot
        );
```


### 6. Mark nullifier as used before transfer

We mark the nullifier BEFORE transferring funds. This is important for security - if we marked it after, a reentrant call could withdraw twice.
1. Attacker calls `withdraw` with valid nullifier
2. Check passes (nullifier not used yet)
3. Transfer starts sending funds to attacker's account
4. Before step 3 executes, attacker's receiving program could potentially call `withdraw` again with the same nullifier
5. The second call also passes the nullifier check (it's still not marked!)
6. Attacker gets paid twice (or more) from a single deposit

By marking the nullifier as used BEFORE the transfer, any reentrant call would fail the nullifier check.

Replace with:

```rust
        require!(
            ctx.accounts.pool_vault.lamports() >= amount,
            PrivateTransfersError::InsufficientVaultBalance
        );

        // Step 5: Verify ZK proof via CPI

        // Mark nullifier as used BEFORE transfer to prevent reentrancy
        nullifier_set.mark_nullifier_used(nullifier_hash)?;

        let pool_key = ctx.accounts.pool.key();
```

---

## Part 4: Update Events and Logging

The withdrawal event should include the nullifier so clients can track which nullifiers have been used. Useful to avoid failed transactions or track spends for someones specific wallet.

### 8. Update emit! in withdraw

Replace with:

```rust
        emit!(WithdrawEvent {
            nullifier_hash,
            recipient: ctx.accounts.recipient.key(),
            timestamp: Clock::get()?.unix_timestamp,
        });
```

### 9. Update log message

Find:

```rust
        msg!("Public withdrawal: {} lamports to {}", amount, recipient);
```

Replace with:

```rust
        msg!("Withdrawal: {} lamports to {} with nullifier {:x?}", amount, recipient, nullifier_hash);
```
---

## Build

```bash
cd anchor
anchor build
```

---

## What Changed

- New `NullifierSet` account stores all used nullifier hashes
- Withdrawals must provide a nullifier_hash
- If the same nullifier_hash is submitted twice, the second withdrawal is rejected
- The nullifier_hash can't be linked back to the original commitment

---

## Next Step

We can hide deposits, prove membership, and prevent double-spending. But we're not verifying any proofs yet - anyone could submit fake data!

Continue to [Step 4: The ZK Circuit](./step-4-zk-circuit.md).
