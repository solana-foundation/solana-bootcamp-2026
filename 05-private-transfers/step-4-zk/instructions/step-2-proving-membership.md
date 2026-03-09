**~8 min**

# Step 2: Proving Membership

## Goal

Change our pool to hold a Merkle Tree root. We dont need to store the whole merkle tree - this can be maintained offchain. It's the reason that we emit deposit events with the commitment, so indexers can keep a full Merkle Tree. So we only need to keep track of where we are in the tree and what the Merkle root is.

---

## Update the Program

**File:** `anchor/programs/private_transfers/src/lib.rs`

---

## Part 1: Define the Tree Structure

First, we need constants that define our Merkle tree's properties.

### 1. Add constants at the top

```rust
pub const TREE_DEPTH: usize = 10;
pub const MAX_LEAVES: u64 = 1 << TREE_DEPTH;  // 1024
pub const ROOT_HISTORY_SIZE: usize = 10;

// uncomment empty root and explain 
pub const MIN_DEPOSIT_AMOUNT: u64 = 1_000_000;
```

**What these constants mean:**

- **`TREE_DEPTH`**: Our Merkle tree has 10 levels. Each level doubles the number of possible leaves, so a depth of 10 gives us 2^10 = 1024 leaf positions for commitments.

- **`MAX_LEAVES`**: The maximum number of deposits our pool can hold. `1 << 10` is a bit-shift that computes 2^10 = 1024. Once the tree is full, no more deposits can be made.

- **`ROOT_HISTORY_SIZE`**: We store the last 10 Merkle roots in a ring buffer. Why? When someone deposits, the root changes. If we only stored the current root, a user who generated a proof against the old root would have their withdrawal fail. By keeping a history of recent roots, we give users a window of time to submit their withdrawal before their proof becomes invalid.

- **`EMPTY_ROOT`**: This is the Merkle root of a tree with no leaves - all positions filled with zeros, hashed up to the root. We precompute this value because we need to initialize the pool with a valid starting root. This specific 32-byte value is the Poseidon hash result for an empty tree of depth 10.

they are hashed zeros, this is what a poseidon hash looks like - a 32 byte root

---

## Part 2: Store Tree State On-Chain

Now we update the Pool struct to track where we are in the tree and store recent roots.

### 2. Update Pool struct

Find:

Replace with:

```rust
#[account]
#[derive(InitSpace)]
pub struct Pool {
    pub authority: Pubkey,
    pub total_deposits: u64,
    // Position in the tree where the next commitment will be inserted (0, 1, 2, ...)
    // A Merkle tree is append-only - new commitments are added at the next available leaf position, never overwriting existing ones. We need `next_leaf_index` to know exactly where to place each new deposit (leaf 0, then leaf 1, then leaf 2...).
    pub next_leaf_index: u64,
    // Points to the most recent root in the ring buffer, which of the 10 storage slots contains the most recent root
    pub current_root_index: u64,
    // Ring buffer storing the last 10 Merkle roots
    pub roots: [[u8; 32]; ROOT_HISTORY_SIZE], // array of 10 poseidon hashes
}


```

### 3. Initialize Pool fields

When the pool is created, we set up the initial tree state.

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

 We've set up the on-chain storage for our Merkle tree - the Pool now tracks the next leaf position and stores recent roots. But the deposit and withdraw functions don't use any of this yet. Next, we'll update them:


## Part 3: Update Deposit to Track Leaves and Roots

Each deposit needs to:
1. Accept the new Merkle root (computed offchain after inserting the commitment)
2. Store that root in our history
3. Emit the leaf index so clients can build Merkle proofs, technically we dont need this if we emit every deposit event with the commitment. but its good practice. makes it easier to look up specific commitments or handle faults if you miss a deposit

### 4. Update deposit function signature


Replace with:

```rust
    pub fn deposit(
        ctx: Context<Deposit>,
        commitment: [u8; 32],
        // The Merkle tree is maintained off-chain by the client.
        // After inserting the commitment as a leaf, the client computes
        // the new root and passes it here. The program just stores it.
        new_root: [u8; 32],
        amount: u64,
    ) -> Result<()> {
```

### 5. Update root history after transfer

After the SOL transfer succeeds, we update the tree state and emit an event.


Replace with:

```rust
        system_program::transfer(cpi_context, amount)?;

        // Save which leaf position this commitment was inserted at
        let leaf_index = pool.next_leaf_index;
        
        // Calculate next position in ring buffer using modulo to wrap around.
        // The modulo (%) makes it wrap:
        //  % 10 just means 'divide by 10 and give me the remainder.' When your counter hits 10, the remainder is 0 - back to the start
        // means we can use fixed storage instead of growing forever. Oldest root gets overwritten.
     
        let new_root_index = ((pool.current_root_index + 1) % ROOT_HISTORY_SIZE as u64) as usize;
        
        // Store the new Merkle root at this position (overwrites oldest root)
        pool.roots[new_root_index] = new_root;
        
        // Update pointer to track which slot has the current root
        pool.current_root_index = new_root_index as u64;

        emit!(DepositEvent {
            commitment,
            leaf_index,      // So client can track where in tree this commitment lives
            timestamp: Clock::get()?.unix_timestamp,
            new_root,        // So clients can update their local tree copy
        });

        // Move to next leaf position for the next deposit
        pool.next_leaf_index += 1;
        pool.total_deposits += 1;

        msg!("Deposit: {} lamports at leaf index {}", amount, leaf_index);

        

```
### 6. Add tree full check

We can't accept more deposits once all 1024 leaf positions are used.


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


**Deposit recap:** We've updated `deposit` to:
- Accept a `new_root` from the client,
- Store that root in our ring buffer (overwriting the oldest if full)
- Emit the `leaf_index` so clients know where the commitment lives in the tree
- Check the tree isn't full before accepting new deposits

---

## Part 4: Update Withdraw to Validate Roots

Withdrawals must prove the commitment exists in the tree. Before, we just validate that the provided root is one we've seen before. (The actual ZK proof verification comes in Step 5.)

### 8. Update withdraw function signature

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

### How Users Calculate Their Root

**The client maintains their own copy of the Merkle tree:**

1. **Index deposit events** - The client listens to `DepositEvent` emissions. Each event contains `commitment`, `leaf_index`, and `new_root`. By replaying all events, the client rebuilds the full tree locally.

2. **Compute the current root** - With the full tree in memory, the client hashes up from their commitment's leaf position to get the current root.

3. **Generate a Merkle proof** - The proof is the list of sibling hashes needed to recompute the root from their leaf. For a depth-10 tree, this is 10 hashes.


### 9. Add root validation in withdraw

Reject withdrawals if the root isn't in our recent history.

Replace with:

```rust
    ) -> Result<()> {
        // Step 3: Check nullifier not used

        require!(
            ctx.accounts.pool.is_known_root(&root),
            PrivateTransfersError::InvalidRoot
        );

    
```

Add this after `pub struct Pool { ... }`:

```rust
impl Pool {
    // Check if a root exists in our history - used during withdrawal
    // to verify the user's proof was made against a valid recent root
    pub fn is_known_root(&self, root: &[u8; 32]) -> bool {
        self.roots.iter().any(|r| r == root)
    }
}
```



In Rust, `struct` defines what data a type holds, and `impl` defines what methods (functions) it has.

- `&self` - takes a reference to the Pool instance (like `this` in other languages)
- `root: &[u8; 32]` - parameter `root` is a reference (`&`) to a 32-byte array
- `self.roots.iter()` - iterate over the roots array
- `.any(|r| r == root)` - check if any element matches. for each element `|r|` check that  `r` and checks  equals `root`


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

We can prove a commitment exists. But you might have noticed that if we emit every single commitment publicly, then everyone can just create their own Merkle root, and therefore be able to withdraw. In the next step we're going to talk about nullifiers which accomplish two things - proving we are the owner of a commitment, and proving that it hasn't been withdrawn already.

Continue to [Step 3: Preventing Double-Spend](./step-3-preventing-double-spend.md).
