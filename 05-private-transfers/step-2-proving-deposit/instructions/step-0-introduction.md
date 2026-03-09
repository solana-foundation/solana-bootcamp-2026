# Step 0: Introduction

## Goal

Open the starter code, understand what it does now, and see exactly what we'll be adding.

---

## The Project Structure

```
noir-solana-private-transfers/
├── anchor/                          # Solana program (YOU EDIT THIS)
│   └── programs/private_transfers/
│       └── src/lib.rs
├── circuits/                        # Noir ZK circuits (ALREADY COMPLETE)
│   ├── hasher/                      # Computes commitment hash
│   ├── merkle-hasher/               # Computes Merkle roots
│   └── withdrawal/                  # Main withdrawal proof circuit
├── backend/                         # Proof generation (ALREADY COMPLETE)
│   └── src/
│       ├── deposit.ts               # Generates secrets, commitment
│       └── withdraw.ts              # Generates ZK proof
└── frontend/                        # React UI (ALREADY COMPLETE)
```

**You'll modify:** `anchor/programs/private_transfers/src/lib.rs`

**You'll read (but not modify):** The circuits in `circuits/` and backend in `backend/` and `frontend`

---

## Run the Frontend

Let's see what the UI looks like:

```bash
# Terminal 1 - Start the backend
cd backend
bun install
bun run dev

# Terminal 2 - Start the frontend
cd frontend
bun install
bun run dev
```

Open `http://localhost:3000`.

You'll see:
- **Connect Wallet** - Links to your Phantom/Solflare
- **Deposit Section** - Enter amount, get a deposit note
- **Withdraw Section** - Paste deposit note, get your funds

The frontend is already wired up to call the backend APIs and build transactions. We just need to add the privacy features to the program.

---

# Open program


Open `anchor/programs/private_transfers/src/lib.rs`.

This is a basic escrow-style program. Walk through how it works

---

## Build the Program

Make sure the starter code compiles:

```bash
cd anchor
anchor build
```

You should see it compile successfully.

---

## Next Step

Let's start by hiding deposit details with commitments.

Continue to [Step 1: Hiding Deposits](./step-1-hiding-deposits.md).
