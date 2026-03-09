# Private Transfers on Solana with Noir ZK

## From Escrow to Private Pool

You just built an escrow program. It holds funds and releases them when conditions are met. Now we're going to build something technically kind of similar. It allows users to make private transfers using a pool. The sender deposits into the pool, and then the receiver withdraws from the pool, and using zero-knowledge cryptography the sender and the reciever are not linked.

## What You'll Build

By the end of this tutorial, you'll have:

- A Solana program that accepts private deposits
- ZK circuits that prove withdrawal rights without revealing identity
- onchain verification using Sunspot (Groth16 proofs on Solana)
- A working frontend for the complete flow

## How It Works

The system uses cryptographic primitives working together:

1. **Commitments** - Hide deposit details in a hash (like a sealed envelope)
2. **Nullifiers** - Prevent double-spending without revealing which deposit
3. **Merkle Trees** - Efficiently prove a deposit exists in the pool
4. **ZK Circuits** - Prove everything without revealing anything
5. **Sunspot Verification** - Verify Groth16 proofs onchain via CPI

## Prerequisites

**Solana knowledge** (from previous projects):

- Accounts and PDAs (from Escrow project)
- CPI basics (from Escrow project)
- Anchor program structure

**Tools you should have**:

- Anchor v1.0.0-rc.2, Solana CLI, Bun (from previous projects)

**Tools we'll install during the tutorial**:

- [Noir/Nargo v1.0.0-beta.13](https://noir-lang.org/docs) - ZK circuit compiler (installed in Step 4)
- [Sunspot CLI](https://github.com/reilabs/sunspot) - Groth16 proving for Solana (installed in Step 4)
- Go 1.24+ - Required for building Sunspot

## Tutorial Steps

| Step | Topic                                                      | What You'll Learn                                      |
| ---- | ---------------------------------------------------------- | ------------------------------------------------------ |
| 0    | [Introduction](./step-0-introduction.md)                   | Understanding the problem and starter code             |
| 1    | [Hiding Deposits](./step-1-hiding-deposits.md)             | Hiding deposit details with hashes                     |
| 2    | [Proving Membership](./step-2-proving-membership.md)       | Efficient membership proofs with Merkle trees          |
| 3    | [Preventing Double-Spend](./step-3-preventing-double-spend.md) | Preventing double-spending with nullifiers         |
| 4    | [The ZK Circuit](./step-4-zk-circuit.md)                   | Understanding the withdrawal proof                     |
| 5    | [On-chain Verification](./step-5-onchain-verification.md)  | onchain proof verification                             |
| 6    | [Demo](./step-6-demo.md)                                   | Running the demo + seeing it all work                  |

## Quick Start

```bash
# Clone the repo and install dependencies
bun install

# Build the Anchor program
cd anchor
anchor build

# Start the frontend (for later testing)
cd ../frontend && bun run dev
```

> **Note**: Circuit compilation (`nargo compile`) and Sunspot setup are covered in Step 4 when we get to ZK proofs. You don't need these tools until then!

## Architecture Overview

```
┌─────────────────┐      ┌─────────────────┐      ┌─────────────────┐
│                 │      │                 │      │                 │
│    Frontend     │─────▶│    Backend      │─────▶│  Noir Circuits  │
│    (React)      │      │   (Express)     │      │   (nargo CLI)   │
│                 │◀─────│                 │◀─────│                 │
└─────────────────┘      └─────────────────┘      └─────────────────┘
        │                                                  │
        │                                                  │
        ▼                                                  ▼
┌─────────────────┐                              ┌─────────────────┐
│                 │                              │                 │
│  Solana RPC     │◀────────────────────────────│ Sunspot Verifier│
│                 │                              │   (onchain)    │
└─────────────────┘                              └─────────────────┘
```

## onchain Accounts

| Account          | Type    | Seeds                  | Purpose                                                |
| ---------------- | ------- | ---------------------- | ------------------------------------------------------ |
| **Pool**         | PDA     | `["pool"]`             | Stores Merkle root history, leaf index, deposit count  |
| **PoolVault**    | PDA     | `["vault", pool]`      | Holds deposited SOL (like escrow vault)                |
| **NullifierSet** | PDA     | `["nullifiers", pool]` | Tracks used nullifier hashes to prevent double-spend   |
| **Verifier**     | Program | N/A                    | Sunspot-generated program that verifies Groth16 proofs |

### Why These Accounts?

**Pool** - The "state" of the privacy pool. Stores:

- `roots[10]`: Ring buffer of recent Merkle roots (allows proof timing flexibility)
- `next_leaf_index`: Where the next deposit goes in the tree
- `current_root_index`: Which root is newest

**PoolVault** - Holds the actual SOL. Separate from Pool because:

- Pool stores data, Vault holds lamports
- Vault is a PDA so the program can sign transfers out

**NullifierSet** - Prevents double-spending:

- When you withdraw, your `nullifier_hash` is added here
- Future withdrawals check: "Is this nullifier_hash already used?"
- Stored separately because it grows (Vec) while Pool is fixed-size

> **Scaling Note:** With 256 max nullifiers, `contains()` is fine. But if you scale up, consider one PDA per nullifier pattern (hash as seed) for O(1) lookup.

**Verifier** - External program generated by Sunspot:

- Contains the verification key baked in
- Our program calls it via CPI
- Returns success/error based on proof validity

## Project Structure

```
noir-solana-private-transfers/
├── circuits/                    # Noir ZK circuits (already complete)
│   ├── hasher/                  # Computes commitment & nullifier_hash
│   ├── merkle-hasher/           # Computes Merkle roots
│   └── withdrawal/              # Main withdrawal proof circuit
├── anchor/                      # Solana program
│   └── programs/private_transfers/src/lib.rs  # You'll modify this
├── backend/                     # Proof generation server
├── frontend/                    # React UI
└── instructions/                # This tutorial
```

## Learning Path

This tutorial focuses on **Solana integration** with ZK proofs. The Noir circuits are already complete - you'll walk through them to understand what they do, then implement the Solana program that uses them.

**You'll modify:** `anchor/programs/private_transfers/src/lib.rs`

**You'll understand:** The Noir circuits in `circuits/`

Ready? Start with [Step 0: Introduction](./step-0-introduction.md).
