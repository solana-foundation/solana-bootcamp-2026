**~7 min**

# Step 4.2: Sunspot Verifier

## Goal

Deploy the Sunspot verifier program that validates ZK proofs on-chain.

---

* Tell Sunspot where our verifier template is
* Generate verifier from verifier key, which is a Solana program
* Deploy the verifier to devnet

---

### Set the GNARK_VERIFIER_KEY environment variable

Wherever you installed sunspot, inside that directory you'll find `gnark-solana/crates/verifier-bin`. This needs to be exported as a variable

```bash
export GNARK_VERIFIER_BIN="$HOME/.sunspot/gnark-solana/crates/verifier-bin"
```

---

## Generate the Verifier Program

```bash
cd circuit
sunspot deploy target/withdrawal.vk
```

> Sunspot converts our Noir circuit (compiled to CCS - Customizable Constraint System format) into a Groth16 verifier using Gnark. The .vk file contains the cryptographic parameters specific to our circuit.

This creates:
- `target/withdrawal.so` - The compiled Solana program
- `target/withdrawal-keypair.json` - Keypair for deployment (like a wallet for the program - lets us deploy to a specific address and redeploy to the same address later)

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

Save this for frontend step

---

## What You Deployed

The verifier program:
- Has your verification key baked in
- Only accepts proofs from your specific circuit
- Uses BN254 elliptic curve pairings (~1.4M compute units)
- Is stateless - no accounts needed, just CPI

---

## Solana Deep Dive: Why Groth16?

Not all proof systems work well on Solana. Here's why Groth16 is ideal:

**Tiny proofs:** Groth16 proofs are ~256 bytes. Solana transactions max out at 1232 bytes, so small proofs are essential. STARKs can be 50-200KB - they literally don't fit in a transaction.

**Fast verification:** Groth16 verification uses elliptic curve pairings - expensive math, but predictable. On Solana, it costs ~1.4M compute units. The default transaction budget is 200K CUs, but you can request up to 1.4M with `SetComputeUnitLimit`. Our verification fits.

**The trade-off:** Groth16 requires a trusted setup (those `.pk` and `.vk` files we generated). If someone knew the randomness used during setup, they could forge proofs. For production, you'd use a multi-party computation ceremony where the randomness is destroyed. Sunspot handles this for development.

**Why a separate verifier program?** 

The verifier is ~100KB of compiled code - too big to include in your main program. By deploying it separately:
- Multiple programs can share the same verifier
- You can upgrade the verifier without redeploying your main program
- The verification logic is auditable in isolation

This pattern of "call external verifier via CPI" is becoming standard for ZK on Solana.

---

## Next Step

Now we'll update our program to call this verifier during withdrawals.

Continue to [Step 6: Verification CPI](./step-6-verification-cpi.md).
