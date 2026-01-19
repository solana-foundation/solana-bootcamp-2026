# Solana Prediction Markets

A minimal full-stack prediction market example on Solana using Anchor + framework-kit. Users can create binary (YES/NO) markets, place bets with SOL, and claim winnings after manual resolution.

## Getting Started

```shell
npm install
npm run setup   # Builds program and generates client
npm run dev
```

Open [http://localhost:3000](http://localhost:3000), connect your wallet, and interact with prediction markets on devnet.

## Features

- **Create markets** - Ask any yes/no question and set a resolution deadline
- **Place bets** - Bet SOL on YES or NO outcomes before the deadline
- **Resolve markets** - Market creators manually resolve after the deadline
- **Claim winnings** - Winners receive proportional payouts from the losing pool

## Stack

| Layer          | Technology                              |
| -------------- | --------------------------------------- |
| Frontend       | Next.js 16, React 19, TypeScript        |
| Styling        | Tailwind CSS v4                         |
| Solana Client  | `@solana/client`, `@solana/react-hooks` |
| Program Client | Codama-generated, `@solana/kit`         |
| Program        | Anchor 0.31 (Rust)                      |

## Project Structure

```
├── app/
│   ├── components/
│   │   ├── providers.tsx           # Solana client setup
│   │   ├── create-market-form.tsx  # Market creation UI
│   │   ├── market-card.tsx         # Market betting/resolution UI
│   │   └── markets-list.tsx        # Fetch and display all markets
│   ├── generated/prediction_market/ # Codama-generated client
│   └── page.tsx                    # Main page
├── anchor/
│   └── programs/prediction_market/ # Prediction market program (Rust)
└── codama.json                     # Codama client generation config
```

## How It Works

### Program Architecture

**Accounts:**
- `Market` (PDA) - Stores question, pools, resolution state, creator authority
- `UserPosition` (PDA) - Tracks each user's bets per market

**Instructions:**
1. `create_market` - Initialize market with question and deadline
2. `place_bet` - Transfer SOL to market pool (YES or NO)
3. `resolve_market` - Creator sets winning outcome after deadline
4. `claim_winnings` - Winners withdraw proportional share

**Payout formula:**
```
winnings = (user_bet / winning_pool) * losing_pool
total = user_bet + winnings
```

### Security

- PDA-based pool management (no admin keys holding funds)
- Time-based betting window enforcement
- Creator-only resolution after deadline
- Double-claim prevention via position flag
- Checked math for overflow protection

## Deploy Your Own

### Prerequisites

- [Rust](https://rustup.rs/)
- [Solana CLI](https://solana.com/docs/intro/installation)
- [Anchor](https://www.anchor-lang.com/docs/installation)

### Steps

1. **Configure Solana CLI for devnet**
   ```bash
   solana config set --url devnet
   ```

2. **Create a wallet and fund it**
   ```bash
   solana-keygen new
   solana airdrop 2
   ```

3. **Build and deploy**
   ```bash
   cd anchor
   anchor build
   anchor keys sync    # Updates program ID in source
   anchor build        # Rebuild with new ID
   anchor deploy
   cd ..
   npm run setup       # Regenerate client
   npm run dev
   ```

## Testing

The program includes LiteSVM-based tests in `anchor/programs/prediction_market/src/tests.rs`.

```bash
npm run anchor-build   # Build first
npm run anchor-test    # Run tests
```

## Learn More

- [Solana Docs](https://solana.com/docs) - core concepts
- [Anchor Docs](https://www.anchor-lang.com/docs) - program framework
- [framework-kit](https://github.com/solana-foundation/framework-kit) - React hooks
- [Codama](https://github.com/codama-idl/codama) - client generation
- [solana-dev-skill](https://github.com/GuiBibeau/solana-dev-skill) - Claude Code skill for Solana development
