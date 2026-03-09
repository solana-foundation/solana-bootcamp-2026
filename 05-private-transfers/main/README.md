# Private Transfers on Solana

Privacy-preserving SOL transfers using Noir ZK circuits and onchain Groth16 verification via Sunspot.

This is for demo purposes only to explain zero-knowledge proofs on Solana. It is not intended to be used or deployed to production.

## How it Works

1. **Deposit**: User deposits SOL into a shared pool. A commitment `hash(nullifier, secret, amount)` is added to a Merkle tree.

2. **Withdraw**: User generates a ZK proof showing they know a valid commitment without revealing which one. The proof is verified onchain via Sunspot.

3. **Privacy**: The link between deposit and withdrawal is broken. Only the amount is visible (variable amounts trade privacy for flexibility).

## Project Structure

```
├── circuits/
│   ├── hasher/          # Computes commitment and nullifier hash
│   ├── merkle-hasher/   # Computes Merkle root for a leaf
│   └── withdrawal/      # Main ZK proof circuit
├── anchor/
│   └── programs/
│       └── private_transfers/  # Solana program
├── backend/             # API for proof generation
└── frontend/            # React UI
```

## Requirements

| Tool       | Version       |
| ---------- | ------------- |
| Noir/Nargo | 1.0.0-beta.13 |
| Sunspot    | latest        |
| Anchor     | 1.0.0-rc.2    |
| Solana CLI | stable        |
| Go         | 1.24+         |
| Bun        | latest        |

This project uses `bun` because I love `bun` but you can use `pnpm` or whichever you'd like.

## Installation

### 1. Install Noir

```bash
curl -L https://raw.githubusercontent.com/noir-lang/noirup/main/install | bash
noirup -v 1.0.0-beta.13
```

### 2. Install Sunspot

```bash
git clone https://github.com/reilabs/sunspot.git
cd sunspot/go
go build -o sunspot .
sudo mv sunspot /usr/local/bin/
```

For the `sunspot deploy` command, set the environment variable:

```bash
export GNARK_VERIFIER_BIN=/path/to/sunspot/gnark-solana/crates/verifier-bin
```

### 3. Install Anchor

For this demo we use `v1.0.0-rc.2`. When Anchor v1 is officially released use that!

```bash
cargo install --git https://github.com/coral-xyz/anchor --tag v1.0.0-rc.2 anchor-cli
```

### 4. Install Dependencies

```bash
bun install
```

## Quick Start

### 1. Build Circuits

```bash
cd circuits/withdrawal
nargo compile
nargo test
```

### 2. Generate Verification Keys

```bash
cd circuits/withdrawal
sunspot compile target/withdrawal.json
sunspot setup target/withdrawal.ccs
```

### 3. Deploy Verifier to Solana

```bash
cd circuits/withdrawal
sunspot deploy target/withdrawal.vk
solana program deploy target/withdrawal.so --url devnet
```

Update the verifier program ID in:

- `anchor/programs/private_transfers/src/lib.rs`
- `anchor/Anchor.toml`
- `frontend/src/constants.ts`

### 4. Build & Deploy Program

```bash
cd anchor
anchor build
anchor deploy --provider.cluster devnet
```

### 5. Run

```bash
# Start backend (generates proofs)
cd backend && bun run dev

# Start frontend
cd frontend && bun run dev
```

## Testing

```bash
# Circuit tests (uses BB not groth16, but doesn't matter for testing)
cd circuits/withdrawal && nargo test

# LiteSVM tests (only tests deposit logic)
cd anchor && cargo test

# E2E test with ZK proof verification (requires backend running)
cd anchor
ANCHOR_PROVIDER_URL=https://api.devnet.solana.com \
ANCHOR_WALLET=~/.config/solana/id.json \
npx ts-mocha -p ./tsconfig.json -t 1000000 tests/e2e.ts
```

## Architecture

### Circuit (Noir)

**Public inputs:**

- `root` - Merkle tree root
- `nullifier_hash` - Hash of nullifier (prevents double-spend)
- `recipient` - Withdrawal address
- `amount` - Withdrawal amount

**Private inputs:**

- `nullifier`, `secret` - Commitment preimage
- `merkle_proof`, `is_even` - Merkle path

### Solana Program

- **Pool**: Stores Merkle root history (10 roots)
- **NullifierSet**: Tracks used nullifiers
- **Vault**: Holds deposited SOL

Withdrawal verifies the ZK proof via CPI to Sunspot's onchain verifier.

## Limitations

This is a demo project only and should not be used in production.

- **Not audited** - educational project
- **Variable amounts reduce privacy** - deposits/withdrawals can be correlated by amount
- **256 nullifier limit** - would need sharding for production

## Resources

- [Noir](https://noir-lang.org)
- [Sunspot](https://github.com/reilabs/sunspot)
- [Anchor](https://anchor-lang.com)
