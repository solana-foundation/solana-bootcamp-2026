# Solana RWA Demo - Labubu

A Solana mystery box NFT lottery system featuring 11 limited edition Labubu characters using Token-2022 standard.

## Features

- **11 Unique Labubu Types**: 10 normal types (120 supply each) + 1 rare type (6 supply) = 1,206 total NFTs
- **Mystery Box Mechanism**: Weighted random minting based on remaining supply
- **Token-2022 NFTs**: Using Solana's token-2022 standard for token mint
- **Anchor Program**: On-chain collection state and minting logic
- **Next.js Frontend**: Interactive mystery box UI with wallet integration

## Prerequisites

- Node.js 18+ and pnpm
- Rust and Anchor CLI (0.31.1+)
- Solana CLI configured for devnet
- Devnet SOL ([faucet](https://faucet.solana.com/))

## Quick Start

```bash
# 1. Install dependencies
cd anchor && pnpm install
cd ../app && pnpm install

# 2. Build and deploy program
cd anchor
anchor build
solana config set --url https://api.devnet.solana.com
anchor deploy --provider.cluster devnet

# 3. Generate TypeScript client
pnpm codegen

# 4. Initialize collection
anchor run initialize

# 5. Run frontend
cd ../app && pnpm dev
```

Open [http://localhost:3000](http://localhost:3000)

## Program Architecture

Three main instructions:
1. `initialize_collection` - Creates collection account with supply counters
2. `create_labubu_mint` - Creates Token-2022 mint for each Labubu type
3. `mint_random` - Mints one NFT to user's associated token account


## Resources

- [Solana Documentation](https://solana.com/docs)
- [Anchor Framework](https://www.anchor-lang.com/)
- [Token-2022 Guide](https://spl.solana.com/token-2022)
- [@solana/react-hooks](https://github.com/solana-foundation/framework-kit/tree/main/packages/react-hooks)

## License

MIT
