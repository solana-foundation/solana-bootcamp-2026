# Step 5: Sunspot Verifier

## Goal

Deploy the Sunspot verifier program that validates ZK proofs on-chain.

---

## What You'll Do

| Task | Purpose |
|------|---------|
| Set up Sunspot environment | Point to verifier template |
| Generate verifier from vk | Create Solana program that verifies proofs |
| Deploy to devnet | Make verifier available on-chain |

---

## Set Up Sunspot Environment

Sunspot needs access to its verifier template to generate programs.

### Clone the Sunspot repo (one-time setup)

```bash
git clone https://github.com/reilabs/sunspot.git ~/.sunspot
```

### Set the environment variable

Add to your shell config (`~/.zshrc` or `~/.bashrc`):

```bash
export GNARK_VERIFIER_BIN="$HOME/.sunspot/gnark-solana/crates/verifier-bin"
```

Then reload:

```bash
source ~/.zshrc  # or ~/.bashrc
```

---

## Generate the Verifier Program

```bash
cd circuit
sunspot deploy target/withdrawal.vk
```

This creates:
- `target/withdrawal.so` - The compiled Solana program
- `target/withdrawal-keypair.json` - Keypair for deployment

---

## What the Verifier Expects

Looking inside the generated verifier (`~/.sunspot/gnark-solana/crates/verifier-bin/src/lib.rs`):

**Instruction data format:** `proof_bytes || public_witness_bytes`

| Component | Size | Description |
|-----------|------|-------------|
| Proof A | 64 bytes | G1 element |
| Proof B | 128 bytes | G2 element |
| Proof C | 64 bytes | G1 element |
| Num commitments | 4 bytes | Big-endian u32 (usually 0) |
| Commitment PoK | 64 bytes | G1 element |
| **Total proof** | **324 bytes** | |
| Header | 12 bytes | num_public (4) + num_private (4) + length (4) |
| Public inputs | NR_INPUTS * 32 bytes | Each input as 32-byte field element |

For our circuit with 4 public inputs:
- Total instruction data = 324 + 12 + (4 * 32) = **464 bytes**

---

## Deploy to Devnet

Make sure you have SOL:

```bash
solana config set --url devnet
solana balance
# If needed: solana airdrop 2
```

Deploy:

```bash
solana program deploy circuit/target/withdrawal.so
```

**Copy the Program ID:**

```
Program Id: Amugr8yL9EQVAgGwqds9gCmjzs8fh6H3wjJ3eB4pBhXV
```

Save this for Step 6.

---

## What You Deployed

The verifier program:
- Has your verification key baked in
- Only accepts proofs from your specific circuit
- Uses BN254 elliptic curve pairings (~1.4M compute units)
- Is stateless - no accounts needed, just CPI

---

## Next Step

Now we'll update our program to call this verifier during withdrawals.

Continue to [Step 6: Verification CPI](./step-6-verification-cpi.md).
