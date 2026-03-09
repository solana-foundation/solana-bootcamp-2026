# Step 2: Proving Membership

## Goal

Add Merkle tree root tracking so we can efficiently prove a commitment exists in the pool.

---

## What You'll Change

| Component | Change |
|-----------|--------|
| Pool struct | Add `next_leaf_index`, `current_root_index`, `roots` array |
| deposit() | Add `new_root` parameter, store in ring buffer |
| withdraw() | Add `root` parameter, validate against history |
| DepositEvent | Add `leaf_index`, `new_root` |

---

## Update the Program

**File:** `anchor/programs/private_transfers/src/lib.rs`

### 1. Add constants at the top

Find:

```rust
// Step 2: Add Merkle tree constants here
// Step 5: Add SUNSPOT_VERIFIER_ID here

pub const MIN_DEPOSIT_AMOUNT: u64 = 1_000_000; // 0.001 SOL
```

Replace with:

```rust
pub const TREE_DEPTH: usize = 10;
pub const MAX_LEAVES: u64 = 1 << TREE_DEPTH;  // 1024
pub const ROOT_HISTORY_SIZE: usize = 10;

pub const EMPTY_ROOT: [u8; 32] = [
    0x2a, 0x77, 0x5e, 0xa7, 0x61, 0xd2, 0x04, 0x35,
    0xb3, 0x1f, 0xa2, 0xc3, 0x3f, 0xf0, 0x76, 0x63,
    0xe2, 0x45, 0x42, 0xff, 0xb9, 0xe7, 0xb2, 0x93,
    0xdf, 0xce, 0x30, 0x42, 0xeb, 0x10, 0x46, 0x86,
];

pub const MIN_DEPOSIT_AMOUNT: u64 = 1_000_000;
```

---

### 2. Update Pool struct

Find:

```rust
#[account]
#[derive(InitSpace)]
pub struct Pool {
    pub authority: Pubkey,
    pub total_deposits: u64,
    // Step 2: Add next_leaf_index, current_root_index, roots
}

// Step 2: Add is_known_root method to Pool
// Step 3: Add NullifierSet struct with is_nullifier_used and mark_nullifier_used methods
```

Replace with:

```rust
#[account]
#[derive(InitSpace)]
pub struct Pool {
    pub authority: Pubkey,
    pub total_deposits: u64,
    pub next_leaf_index: u64,
    pub current_root_index: u64,
    pub roots: [[u8; 32]; ROOT_HISTORY_SIZE],
}

impl Pool {
    pub fn is_known_root(&self, root: &[u8; 32]) -> bool {
        self.roots.iter().any(|r| r == root)
    }
}

// Step 3: Add NullifierSet struct with is_nullifier_used and mark_nullifier_used methods
```

---

### 3. Initialize Pool fields

Find:

```rust
        pool.total_deposits = 0;
        // Step 2: Initialize next_leaf_index, current_root_index, roots[0]
        // Step 3: Initialize nullifier_set.pool

        msg!("Pool initialized");
```

Replace with:

```rust
        pool.total_deposits = 0;
        pool.next_leaf_index = 0;
        pool.current_root_index = 0;
        pool.roots[0] = EMPTY_ROOT;
        // Step 3: Initialize nullifier_set.pool

        msg!("Pool initialized");
```

---

### 4. Update deposit function signature

Find:

```rust
    pub fn deposit(
        ctx: Context<Deposit>,
        commitment: [u8; 32],
        amount: u64,
    ) -> Result<()> {
```

Replace with:

```rust
    pub fn deposit(
        ctx: Context<Deposit>,
        commitment: [u8; 32],
        new_root: [u8; 32],
        amount: u64,
    ) -> Result<()> {
```

---

### 5. Add tree full check

Find:

```rust
        require!(
            amount >= MIN_DEPOSIT_AMOUNT,
            PrivateTransfersError::DepositTooSmall
        );
        // Step 2: Add tree full check

        let cpi_context = CpiContext::new(
```

Replace with:

```rust
        require!(
            amount >= MIN_DEPOSIT_AMOUNT,
            PrivateTransfersError::DepositTooSmall
        );

        require!(
            pool.next_leaf_index < MAX_LEAVES,
            PrivateTransfersError::TreeFull
        );

        let cpi_context = CpiContext::new(
```

---

### 6. Update root history after transfer

Find:

```rust
        system_program::transfer(cpi_context, amount)?;

        // Step 2: Save leaf_index, update root history

        emit!(DepositEvent {
            commitment,
            amount,
            timestamp: Clock::get()?.unix_timestamp,
        });

        pool.total_deposits += 1;
        // Step 2: Increment next_leaf_index

        msg!("Deposit: {} lamports, commitment: {:?}", amount, commitment);
```

Replace with:

```rust
        system_program::transfer(cpi_context, amount)?;

        let leaf_index = pool.next_leaf_index;
        let new_root_index = ((pool.current_root_index + 1) % ROOT_HISTORY_SIZE as u64) as usize;
        pool.roots[new_root_index] = new_root;
        pool.current_root_index = new_root_index as u64;

        emit!(DepositEvent {
            commitment,
            leaf_index,
            timestamp: Clock::get()?.unix_timestamp,
            new_root,
        });

        pool.next_leaf_index += 1;
        pool.total_deposits += 1;

        msg!("Deposit: {} lamports at leaf index {}", amount, leaf_index);
```

---

### 7. Update DepositEvent

Find:

```rust
#[event]
pub struct DepositEvent {
    pub commitment: [u8; 32],
    pub amount: u64,
    pub timestamp: i64,
}
```

Replace with:

```rust
#[event]
pub struct DepositEvent {
    pub commitment: [u8; 32],
    pub leaf_index: u64,
    pub timestamp: i64,
    pub new_root: [u8; 32],
}
```

---

### 8. Update withdraw function signature

Find:

```rust
    pub fn withdraw(
        ctx: Context<Withdraw>,
        // Step 5: Add proof: Vec<u8>
        // Step 3: Add nullifier_hash: [u8; 32]
        // Step 2: Add root: [u8; 32]
        recipient: Pubkey,
        amount: u64,
    ) -> Result<()> {
```

Replace with:

```rust
    pub fn withdraw(
        ctx: Context<Withdraw>,
        // Step 5: Add proof: Vec<u8>
        // Step 3: Add nullifier_hash: [u8; 32]
        root: [u8; 32],
        recipient: Pubkey,
        amount: u64,
    ) -> Result<()> {
```

---

### 9. Add root validation in withdraw

Find:

```rust
    ) -> Result<()> {
        // Step 3: Check nullifier not used
        // Step 2: Validate root is known

        require!(
            ctx.accounts.recipient.key() == recipient,
```

Replace with:

```rust
    ) -> Result<()> {
        // Step 3: Check nullifier not used

        require!(
            ctx.accounts.pool.is_known_root(&root),
            PrivateTransfersError::InvalidRoot
        );

        require!(
            ctx.accounts.recipient.key() == recipient,
```

---

### 10. Add error codes

Find:

```rust
    #[msg("Deposit amount too small")]
    DepositTooSmall,
```

Add after it:

```rust
    #[msg("Merkle tree is full")]
    TreeFull,
    #[msg("Unknown Merkle root")]
    InvalidRoot,
```

---

## Build

```bash
cd anchor
anchor build
```

---

## What Changed

- Pool now stores a ring buffer of recent Merkle roots
- Each deposit gets a `leaf_index` and updates the root
- Withdrawals must provide a root that exists in history

---

## Next Step

We can prove a commitment exists. But what stops double-spending? Nothing yet.

Continue to [Step 3: Preventing Double-Spend](./step-3-preventing-double-spend.md).
