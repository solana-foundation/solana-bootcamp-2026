# Step 6: Verification CPI

## Goal

Update the program to verify ZK proofs by calling the Sunspot verifier via CPI.

---

## What You'll Change

| Component | Change |
|-----------|--------|
| Constants | Add `SUNSPOT_VERIFIER_ID` |
| withdraw() | Add `proof` parameter, call verifier via CPI |
| Withdraw accounts | Add `verifier_program` |
| Imports | Add `Instruction` and `invoke` |

---

## Update the Program

**File:** `anchor/programs/private_transfers/src/lib.rs`

### 1. Add imports at the top

Find:

```rust
use anchor_lang::prelude::*;
use anchor_lang::system_program;
```

Replace with:

```rust
use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::solana_program::program::invoke;
use anchor_lang::system_program;
```

---

### 2. Add verifier ID constant

Find:

```rust
// Step 2: Add Merkle tree constants here
// Step 5: Add SUNSPOT_VERIFIER_ID here

pub const MIN_DEPOSIT_AMOUNT: u64 = 1_000_000;
```

Replace with (use YOUR verifier ID from Step 5):

```rust
pub const SUNSPOT_VERIFIER_ID: Pubkey = pubkey!("YOUR_VERIFIER_PROGRAM_ID_HERE");

pub const MIN_DEPOSIT_AMOUNT: u64 = 1_000_000;
```

---

### 3. Add encode_public_inputs function

Find:

```rust
// Step 5: Add encode_public_inputs function here

#[derive(Accounts)]
pub struct Initialize<'info> {
```

Replace with:

```rust
/// Encodes public inputs in the format expected by the Gnark/Sunspot verifier.
/// The verifier expects a specific binary format: a 12-byte header followed by
/// each public input as a 32-byte big-endian field element.
/// Big-endian is a way of ordering bytes when storing multi-byte numbers in memory
///  Solana and most modern CPUs use little-endian
// Big-endian: Most significant byte first (the "big end" comes first)
fn encode_public_inputs(
    root: &[u8; 32],
    nullifier_hash: &[u8; 32],
    recipient: &Pubkey,
    amount: u64,
) -> Vec<u8> {
    const NR_PUBLIC_INPUTS: u32 = 4;
    
    // Pre-allocate: 12 bytes header + 4 inputs × 32 bytes each = 140 bytes
    let mut inputs = Vec::with_capacity(12 + 128);

    // === Gnark Header (12 bytes) ===
    // The Gnark verifier expects a specific header format:
    // - Bytes 0-3:  Number of public inputs (big-endian u32)
    // - Bytes 4-7:  Number of commitments, always 0 for our use case (big-endian u32)
    // - Bytes 8-11: Number of public inputs again (big-endian u32)
    inputs.extend_from_slice(&NR_PUBLIC_INPUTS.to_be_bytes());  // 4 inputs
    inputs.extend_from_slice(&0u32.to_be_bytes());              // 0 commitments
    inputs.extend_from_slice(&NR_PUBLIC_INPUTS.to_be_bytes());  // 4 inputs (repeated)

    // === Public Inputs (each 32 bytes, big-endian) ===
    // IMPORTANT: Order must exactly match the circuit's public input declaration!
    // Our circuit declares: root, nullifier_hash, recipient, amount
    
    // 1. Merkle root - proves the commitment exists in the tree
    inputs.extend_from_slice(root);
    
    // 2. Nullifier hash - prevents double-spending (derived from secret + nullifier)
    inputs.extend_from_slice(nullifier_hash);
    
    // 3. Recipient pubkey - who receives the withdrawn funds (32 bytes for Solana pubkey)
    inputs.extend_from_slice(recipient.as_ref());

    // 4. Amount - the withdrawal amount, must be padded to 32 bytes
    //    u64 is only 8 bytes, so we left-pad with 24 zero bytes to make it 32 bytes
    //    Big-endian format: zeros first, then the actual value in the last 8 bytes
    let mut amount_bytes = [0u8; 32];                          // Create a 32-byte array initialized with zeros
    amount_bytes[24..32].copy_from_slice(&amount.to_be_bytes()); // Copy the 8-byte big-endian u64 into the last 8 bytes (indices 24-31)
    inputs.extend_from_slice(&amount_bytes);                     // Append the padded 32-byte amount to our inputs vector

    inputs                                                       // Return the complete public inputs byte array
}

#[derive(Accounts)]
pub struct Initialize<'info> {
```

---

### 4. Add verifier to Withdraw accounts

Find:

```rust
    /// CHECK: Validated in instruction logic
    #[account(mut)]
    pub recipient: UncheckedAccount<'info>,

    // Step 5: Add verifier_program account here

    pub system_program: Program<'info, System>,
}
```

Replace with:

```rust
    /// CHECK: This is a plain wallet address receiving funds, not a typed account.
    /// We use UncheckedAccount because it's just a wallet
    /// Safety: The recipient is validated against the ZK proof's public inputs.
    #[account(mut)]
    pub recipient: UncheckedAccount<'info>,

    /// CHECK: External program without an Anchor IDL in our project.
    /// We use UncheckedAccount because we can't use Program<'info, SunspotVerifier>
    /// without a type definition. Safety: Validated by the constraint below.
    #[account(
        // constraint = custom validation that runs at runtime
        // verifier_program.key() = get the public key of the account passed in
        // == SUNSPOT_VERIFIER_ID = compare it to our hardcoded verifier program ID
        // @ PrivateTransfersError::InvalidVerifier = error to throw if constraint fails
        constraint = verifier_program.key() == SUNSPOT_VERIFIER_ID @ PrivateTransfersError::InvalidVerifier
    )]
    // UncheckedAccount = raw account wrapper, Anchor won't validate owner/data
    // 'info = lifetime parameter, ties account to the transaction context
    pub verifier_program: UncheckedAccount<'info>,

    // System Program is a typed account - Anchor validates this IS the System Program
    // Used for transferring SOL from the vault to the recipient
    pub system_program: Program<'info, System>,
}
```

---

### 5. Update withdraw function signature

Find:

```rust
    pub fn withdraw(
        ctx: Context<Withdraw>,
        // Step 5: Add proof: Vec<u8>
        nullifier_hash: [u8; 32],
        root: [u8; 32],
        recipient: Pubkey,
        amount: u64,
    ) -> Result<()> {
```

Replace with:

```rust
    pub fn withdraw(
        ctx: Context<Withdraw>,
        proof: Vec<u8>,
        nullifier_hash: [u8; 32],
        root: [u8; 32],
        recipient: Pubkey,
        amount: u64,
    ) -> Result<()> {
```

---

### 6. Add verification CPI call

Find:

```rust
        require!(
            ctx.accounts.pool_vault.lamports() >= amount,
            PrivateTransfersError::InsufficientVaultBalance
        );

        // Step 5: Verify ZK proof via CPI

        nullifier_set.mark_nullifier_used(nullifier_hash)?;
```

Replace with:

```rust
        require!(
            ctx.accounts.pool_vault.lamports() >= amount,
            PrivateTransfersError::InsufficientVaultBalance
        );

        // Verify ZK proof via CPI
        let public_inputs = encode_public_inputs(&root, &nullifier_hash, &recipient, amount);
        let instruction_data = [proof.as_slice(), public_inputs.as_slice()].concat();

        invoke(
            &Instruction {
                program_id: ctx.accounts.verifier_program.key(),
                accounts: vec![],
                data: instruction_data,
            },
            &[ctx.accounts.verifier_program.to_account_info()],
        )?;

        nullifier_set.mark_nullifier_used(nullifier_hash)?;
```

---

### 7. Add error code

Find:

```rust
    #[msg("Nullifier set is full")]
    NullifierSetFull,
```

Add after it:

```rust
    #[msg("Invalid verifier program")]
    InvalidVerifier,
```

---

## Build and Deploy

```bash
cd anchor
anchor build
anchor deploy --provider.cluster devnet
```

---

## What Changed

- Withdrawals now require a ZK proof
- Program calls verifier via CPI before releasing funds
- If verification fails, entire transaction reverts
- No funds can move without a valid proof

---

## Next Step

The program is complete! Let's build the frontend and test everything.

Continue to [Step 7: Frontend and Demo](./step-7-frontend-demo.md).
