# Step 3: Preventing Double-Spend

## Goal

Add nullifier tracking to prevent the same deposit from being withdrawn twice.

---

## What You'll Change

| Component | Change |
|-----------|--------|
| NullifierSet struct | New account to store used nullifiers |
| Initialize | Create nullifier_set PDA |
| withdraw() | Add `nullifier_hash` parameter, check and mark used |
| WithdrawEvent | Add `nullifier_hash`, remove `amount` |

---

## Update the Program

**File:** `anchor/programs/private_transfers/src/lib.rs`

### 1. Add NullifierSet struct

Find:

```rust
impl Pool {
    pub fn is_known_root(&self, root: &[u8; 32]) -> bool {
        self.roots.iter().any(|r| r == root)
    }
}

// Step 3: Add NullifierSet struct with is_nullifier_used and mark_nullifier_used methods
```

Replace with:

```rust
impl Pool {
    pub fn is_known_root(&self, root: &[u8; 32]) -> bool {
        self.roots.iter().any(|r| r == root)
    }
}

#[account]
#[derive(InitSpace)]
pub struct NullifierSet {
    pub pool: Pubkey,
    #[max_len(256)]
    pub nullifiers: Vec<[u8; 32]>,
}

impl NullifierSet {
    pub fn is_nullifier_used(&self, nullifier_hash: &[u8; 32]) -> bool {
        self.nullifiers.contains(nullifier_hash)
    }

    pub fn mark_nullifier_used(&mut self, nullifier_hash: [u8; 32]) -> Result<()> {
        require!(
            self.nullifiers.len() < 256,
            PrivateTransfersError::NullifierSetFull
        );
        self.nullifiers.push(nullifier_hash);
        Ok(())
    }
}
```

---

### 2. Add NullifierSet to Initialize accounts

Find:

```rust
    pub pool: Account<'info, Pool>,

    // Step 3: Add nullifier_set account here

    #[account(seeds = [b"vault", pool.key().as_ref()], bump)]
    pub pool_vault: SystemAccount<'info>,
```

Replace with:

```rust
    pub pool: Account<'info, Pool>,

    #[account(
        init,
        payer = authority,
        space = 8 + NullifierSet::INIT_SPACE,
        seeds = [b"nullifiers", pool.key().as_ref()],
        bump
    )]
    pub nullifier_set: Account<'info, NullifierSet>,

    #[account(seeds = [b"vault", pool.key().as_ref()], bump)]
    pub pool_vault: SystemAccount<'info>,
```

---

### 3. Initialize nullifier_set in initialize function

Find:

```rust
        pool.roots[0] = EMPTY_ROOT;
        // Step 3: Initialize nullifier_set.pool

        msg!("Pool initialized");
```

Replace with:

```rust
        pool.roots[0] = EMPTY_ROOT;

        let nullifier_set = &mut ctx.accounts.nullifier_set;
        nullifier_set.pool = pool.key();

        msg!("Pool initialized");
```

---

### 4. Add NullifierSet to Withdraw accounts

Find:

```rust
#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(seeds = [b"pool"], bump)]
    pub pool: Account<'info, Pool>,

    // Step 3: Add nullifier_set account here

    #[account(mut, seeds = [b"vault", pool.key().as_ref()], bump)]
    pub pool_vault: SystemAccount<'info>,
```

Replace with:

```rust
#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut, seeds = [b"pool"], bump)]
    pub pool: Account<'info, Pool>,

    #[account(
        mut,
        seeds = [b"nullifiers", pool.key().as_ref()],
        bump
    )]
    pub nullifier_set: Account<'info, NullifierSet>,

    #[account(mut, seeds = [b"vault", pool.key().as_ref()], bump)]
    pub pool_vault: SystemAccount<'info>,
```

---

### 5. Update withdraw function signature

Find:

```rust
    pub fn withdraw(
        ctx: Context<Withdraw>,
        // Step 5: Add proof: Vec<u8>
        // Step 3: Add nullifier_hash: [u8; 32]
        root: [u8; 32],
        recipient: Pubkey,
        amount: u64,
    ) -> Result<()> {
        // Step 3: Check nullifier not used

        require!(
            ctx.accounts.pool.is_known_root(&root),
```

Replace with:

```rust
    pub fn withdraw(
        ctx: Context<Withdraw>,
        // Step 5: Add proof: Vec<u8>
        nullifier_hash: [u8; 32],
        root: [u8; 32],
        recipient: Pubkey,
        amount: u64,
    ) -> Result<()> {
        let nullifier_set = &mut ctx.accounts.nullifier_set;

        require!(
            !nullifier_set.is_nullifier_used(&nullifier_hash),
            PrivateTransfersError::NullifierUsed
        );

        require!(
            ctx.accounts.pool.is_known_root(&root),
```

---

### 6. Mark nullifier as used before transfer

Find:

```rust
        require!(
            ctx.accounts.pool_vault.lamports() >= amount,
            PrivateTransfersError::InsufficientVaultBalance
        );

        // Step 5: Verify ZK proof via CPI
        // Step 3: Mark nullifier as used

        let pool_key = ctx.accounts.pool.key();
```

Replace with:

```rust
        require!(
            ctx.accounts.pool_vault.lamports() >= amount,
            PrivateTransfersError::InsufficientVaultBalance
        );

        // Step 5: Verify ZK proof via CPI

        nullifier_set.mark_nullifier_used(nullifier_hash)?;

        let pool_key = ctx.accounts.pool.key();
```

---

### 7. Update WithdrawEvent

Find:

```rust
#[event]
pub struct WithdrawEvent {
    pub recipient: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
    // Step 3: Replace amount with nullifier_hash: [u8; 32]
}
```

Replace with:

```rust
#[event]
pub struct WithdrawEvent {
    pub nullifier_hash: [u8; 32],
    pub recipient: Pubkey,
    pub timestamp: i64,
}
```

---

### 8. Update emit! in withdraw

Find:

```rust
        emit!(WithdrawEvent {
            recipient: ctx.accounts.recipient.key(),
            amount,
            timestamp: Clock::get()?.unix_timestamp,
            // Step 3: Replace amount with nullifier_hash
        });
```

Replace with:

```rust
        emit!(WithdrawEvent {
            nullifier_hash,
            recipient: ctx.accounts.recipient.key(),
            timestamp: Clock::get()?.unix_timestamp,
        });
```

---

### 9. Update log message

Find:

```rust
        msg!("Public withdrawal: {} lamports to {}", amount, recipient);
```

Replace with:

```rust
        msg!("Withdrawal: {} lamports to {}", amount, recipient);
```

---

### 10. Add error codes

Find:

```rust
    #[msg("Unknown Merkle root")]
    InvalidRoot,
```

Add after it:

```rust
    #[msg("Nullifier has already been used")]
    NullifierUsed,
    #[msg("Nullifier set is full")]
    NullifierSetFull,
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
