**~14 min**

# Step 4.3: Verification CPI

## Goal

Update the program to verify ZK proofs by calling the Sunspot verifier via CPI.

---


## Update the Program

**File:** `anchor/programs/private_transfers/src/lib.rs`

---

> `Instruction` is just a struct with: program_id, accounts, and data. `invoke` sends that instruction to another program.

### 2. Add verifier ID constant

This is the program ID of the Sunspot verifier you deployed in Step 5. Every CPI call needs the target program's address.

Replace with (use YOUR verifier ID from Step 5):

```rust
pub const SUNSPOT_VERIFIER_ID: Pubkey = pubkey!("YOUR_VERIFIER_PROGRAM_ID_HERE");

pub const MIN_DEPOSIT_AMOUNT: u64 = 1_000_000;
```

---

## Part 2: Encode Public Inputs for the Verifier

The Gnark/Sunspot verifier expects a specific binary format for its input. We need to encode our public inputs (root, nullifier, recipient, amount) exactly as the verifier expects.

### What the Verifier Expects

The instruction data format is: `proof_bytes || public_witness_bytes`


### 3. Add encode_public_inputs function

This function builds the public inputs in the exact format the verifier expects.

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
///
/// Big-endian means most significant byte first - the opposite of how Solana
/// and most CPUs store numbers (little-endian).
fn encode_public_inputs(
    root: &[u8; 32],
    nullifier_hash: &[u8; 32],
    recipient: &Pubkey,
    amount: u64,
) -> Vec<u8> {
    // NR = "number of" - standard abbreviation in cryptographic code
    const NR_PUBLIC_INPUTS: u32 = 4;

    // Pre-allocate: 12 bytes header + 4 inputs × 32 bytes each = 140 bytes
    // We write 12 + 128 instead of 140 to show WHERE the number comes from
    // Vec::with_capacity pre-allocates memory so we don't resize during extend_from_slice calls
    let mut inputs = Vec::with_capacity(12 + 128);

    // === Gnark Header (12 bytes) ===
    // The Gnark verifier expects:
    // - Bytes 0-3:  Number of public inputs (big-endian u32)
    // - Bytes 4-7:  Number of commitments, always 0 for us (big-endian u32)
    //              (Gnark supports "commitment" schemes but we don't use them - different from our deposit commitments!)
    // - Bytes 8-11: Number of public inputs again (big-endian u32) - yes, twice, it's the Gnark format

    // extend_from_slice appends a byte slice to our Vec - efficient way to build byte arrays
    inputs.extend_from_slice(&NR_PUBLIC_INPUTS.to_be_bytes());
    inputs.extend_from_slice(&0u32.to_be_bytes());  // 0 commitments
    inputs.extend_from_slice(&NR_PUBLIC_INPUTS.to_be_bytes());

    // === Public Inputs (each 32 bytes, big-endian) ===
    // IMPORTANT: Order must exactly match the circuit's public input declaration!
    // Our circuit declares: root, nullifier_hash, recipient, amount

    // 1. Merkle root - proves the commitment exists in the tree
    inputs.extend_from_slice(root);

    // 2. Nullifier hash - prevents double-spending
    inputs.extend_from_slice(nullifier_hash);

    // 3. Recipient pubkey - who receives the funds (32 bytes)
    // as_ref() converts Pubkey to &[u8] - needed because Pubkey isn't a byte array directly
    // root and nullifier_hash are already [u8; 32] so they don't need conversion
    inputs.extend_from_slice(recipient.as_ref());

    // 4. Amount - padded to 32 bytes (u64 is only 8 bytes)
    //    Left-pad with 24 zero bytes, then the 8-byte big-endian value
    let mut amount_bytes = [0u8; 32];
    amount_bytes[24..32].copy_from_slice(&amount.to_be_bytes());
    inputs.extend_from_slice(&amount_bytes);

    inputs
}

#[derive(Accounts)]
pub struct Initialize<'info> {
```

---

## Part 3: Add Verifier to Withdraw Accounts

The withdraw instruction needs access to the verifier program so it can call it via CPI. We validate that the passed program is actually our verifier (not some malicious program).

### 4. Add verifier to Withdraw accounts


Add below:

```rust
    // #[account(constraint = ...)] is Anchor's way to add custom validation
    // The constraint runs BEFORE your instruction code - if it fails, tx reverts
    // @ PrivateTransfersError::InvalidVerifier sets a custom error message
    #[account(
        constraint = verifier_program.key() == SUNSPOT_VERIFIER_ID @ PrivateTransfersError::InvalidVerifier
    )]
    /// CHECK: External program without an Anchor IDL in our project.
    /// We use UncheckedAccount because we can't use Program<'info, SunspotVerifier>
    /// without importing that program's types. The constraint above validates it.
    pub verifier_program: UncheckedAccount<'info>,
}
```

> Note on `/// CHECK:` - Anchor requires this comment on every UncheckedAccount to prove you've thought about security. Without it, `anchor build` will fail. Always explain WHY it's safe.

> Note on `#[account(mut)]` for recipient - it's marked mutable because we're transferring SOL TO it. Any account receiving lamports must be writable.


Now our program knows about the verifier program

---

## Part 4: Update Withdraw to Verify Proofs

Now we update the withdraw function to:
1. Accept the proof as input
2. Encode the public inputs
3. Call the verifier via CPI
4. Only proceed if verification succeeds (CPI failure = transaction reverts)

### 5. Update withdraw function signature


```rust
    pub fn withdraw(
      
        // The ZK proof generated by the client (324 bytes)
        proof: Vec<u8>,
      
    ) -> Result<()> {
```

### 6. Add verification CPI call

This is where the magic happens. We call the verifier program with the proof and public inputs. If the proof is invalid, the CPI fails and the entire transaction reverts - no funds are transferred.

Find:


Add:
```rust

        // === Verify ZK proof via CPI ===
        // Encode public inputs in the format the verifier expects
        let public_inputs = encode_public_inputs(&root, &nullifier_hash, &recipient, amount);

        // .as_slice() converts Vec<u8> to &[u8] (a slice reference)
        // .concat() joins multiple slices into one new Vec
        // Result: proof bytes followed by public input bytes
        let instruction_data = [proof.as_slice(), public_inputs.as_slice()].concat();

        // Why invoke() instead of CpiContext like we used for transfers?
        // - CpiContext is Anchor's helper for calling OTHER Anchor programs
        // - invoke() is the low-level Solana way - works with ANY program
        // - The Sunspot verifier isn't an Anchor program, so we use invoke()
        //
        // invoke() takes:
        // 1. &Instruction - what to call (program + accounts + data)
        // 2. &[AccountInfo] - accounts the called program needs access to
        invoke(
            &Instruction {
                program_id: ctx.accounts.verifier_program.key(),  // WHO to call
                accounts: vec![],  // Verifier needs no accounts, just the instruction data
                data: instruction_data,  // WHAT to send (proof + public inputs)
            },
            &[ctx.accounts.verifier_program.to_account_info()],  // Account infos for the runtime
        )?;

        // If we get here, proof was valid! Mark nullifier and transfer funds
        nullifier_set.mark_nullifier_used(nullifier_hash)?;
```

---

## Solana Deep Dive: CPI Patterns

Cross-Program Invocation (CPI) is how Solana programs talk to each other. Let's understand the patterns:

**`invoke` vs `invoke_signed`:**
- `invoke` - Call another program. The caller's authority is passed through.
- `invoke_signed` - Same, but also sign with PDA seeds. Use this when your PDA needs to authorize something (like transferring from a PDA-owned vault).

We use `invoke` here because the verifier doesn't need any signatures - it just validates math.

**`invoke` vs `CpiContext`:**
- `CpiContext` is Anchor's helper for calling other Anchor programs. It gives you type safety and automatic account validation.
- `invoke` is raw Solana - works with any program, but you build the instruction manually.

The Sunspot verifier isn't an Anchor program (it's generated Rust code), so we use `invoke`. Earlier, when we transferred SOL, we used `CpiContext` because the System Program has Anchor bindings.

**Why the verifier needs no accounts:**

Most CPIs pass accounts - the called program needs to read/write data. But our verifier is purely computational: it takes proof bytes + public inputs, does elliptic curve math, and either succeeds or fails. No state to read, nothing to write. This makes it cheap and simple.

**Atomic failure:**

If `invoke` returns an error, the entire transaction reverts. This is crucial for security - we can't have a situation where verification fails but funds still transfer. The `?` after `invoke(...)` propagates the error, causing the transaction to fail if verification fails.

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
