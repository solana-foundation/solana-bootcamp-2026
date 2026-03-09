# Step 4: The ZK Circuit

## Goal

Create the Noir circuit that proves deposit ownership and generate the proving/verification keys.

---

## What You'll Create

| File | Purpose |
|------|---------|
| `circuit/Nargo.toml` | Noir project config |
| `circuit/src/main.nr` | Withdrawal circuit |
| `circuit/src/merkle_tree.nr` | Merkle proof verification |

---

## Install Nargo

Nargo is the Noir compiler.

```bash
# Install noirup (Noir version manager)
curl -L https://raw.githubusercontent.com/noir-lang/noirup/main/install | bash

# Source your shell or restart terminal
source ~/.bashrc  # or ~/.zshrc

# Install the version compatible with Sunspot
noirup -v 1.0.0-beta.3

# Verify
nargo --version
```

---

## Look through circuit

Withdrawal circuit

## Test circuit

`nargo test`

---

## Compile the Circuit

```bash
cd circuit
nargo compile
```

This creates `target/withdrawal.json` - the compiled circuit.

---

## Install Sunspot

Sunspot converts Noir circuits to Groth16 proofs verifiable on Solana.

Requires **Go 1.24+**:

```bash
go version  # Should show 1.24+
```

If needed, install from [go.dev/dl](https://go.dev/dl/).

Install Sunspot:

```bash
git clone https://github.com/reilabs/sunspot.git
cd sunspot/go
go build -o sunspot .

# Add to PATH
sudo mv sunspot /usr/local/bin/

# Verify
sunspot --help
```

---

## Generate Proving Keys

Groth16 proofs require a **trusted setup** - a one-time process that generates cryptographic keys specific to your circuit.

**Why do we need this?**

Groth16 achieves its tiny proof size (~200 bytes) and fast verification by "baking in" the circuit structure into special keys during setup.
The setup produces two keys:

- **Proving Key (pk)**: Contains the cryptographic parameters needed to generate proofs.

- **Verification Key (vk)**: A small (~1KB) key that contains just enough information to verify proofs. This is what gets deployed on-chain

The "trusted" part refers to the randomness used during setup - if someone knew this randomness, they could forge proofs. Sunspot handles this securely for development, and production systems use multi-party computation ceremonies where the randomness is destroyed.

```bash
cd circuit

# Convert to CCS format
sunspot compile target/withdrawal.json

# Generate proving and verification keys
sunspot setup target/withdrawal.ccs
```

**Output files:**

| File | Size | Purpose |
|------|------|---------|
| `target/withdrawal.ccs` | ~100KB | Circuit in CCS format |
| `target/withdrawal.pk` | ~2MB | Proving key (for generating proofs) |
| `target/withdrawal.vk` | ~1KB | Verification key (for on-chain verifier) |

---
---

## What You Built

- **Withdrawal circuit** - Proves you own a deposit without revealing which one
- **Proving key** - Backend will use this to generate proofs
- **Verification key** - Will be deployed as an on-chain verifier

---

## Next Step

We have the circuit and keys. Now we need to deploy a verifier program to Solana.

Continue to [Step 5: Sunspot Verifier](./step-5-sunspot-verifier.md).
