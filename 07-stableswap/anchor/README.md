# StableSwap AMM

A two-token liquidity pool optimized for stablecoin pairs, built with [Anchor](https://www.anchor-lang.com/) on Solana. Implements the [Curve StableSwap invariant](https://curve.fi/files/stableswap-paper.pdf) to achieve dramatically lower slippage than constant-product AMMs when swapping similarly-priced assets (e.g. USDC/USDT).

## How It Works

Standard AMMs (Uniswap-style `x·y = k`) have significant price impact even for modest trades. For assets that should trade near 1:1, the StableSwap invariant:

```
4·A·(x + y) + D  =  4·A·D + D³ / (4·x·y)
```

concentrates liquidity around the peg, giving near-zero slippage for balanced swaps while still self-correcting if the peg breaks.

The **amplification parameter A** controls the trade-off:
- **High A (100–2000):** nearly flat curve, minimal slippage near peg — ideal for USDC/USDT
- **Low A (1–10):** approaches constant-product, handles de-peg scenarios better

### Example

Swapping 10% of the pool with A=100:
- **StableSwap:** ~99.9% output efficiency
- **Constant product:** ~90.9% output efficiency

## Program ID

```
CorabfeniSyoc4aLcJe7t9b3RaFX5tzVWXdewU1xuA6B
```

## Prerequisites

- [Rust](https://rustup.rs/)
- [Solana CLI](https://solana.com/developers/guides/getstarted/setup-local-development)
- [Anchor CLI](https://www.anchor-lang.com/docs/installation) v1.0.0-rc.2
- [Node.js](https://nodejs.org/) + [Yarn](https://yarnpkg.com/)

## Building

```bash
anchor build
```

## Testing

```bash
yarn install
anchor test
```

Tests cover the full pool lifecycle with a local validator:
- Pool initialization
- Adding initial and subsequent liquidity
- Swapping A→B and B→A
- Slippage protection on swaps and withdrawals
- Removing liquidity
- StableSwap vs constant-product efficiency comparison

## Instructions

### `initialize_pool`

Creates a new pool for a token A / token B pair.

| Argument | Type | Description |
|----------|------|-------------|
| `amplification` | `u64` | A parameter (1–1,000,000). Typical: 100–2000 for stablecoins. |
| `fee_bps` | `u16` | Swap fee in basis points (e.g. `4` = 0.04%). |

Automatically creates the pool PDA, LP mint, and two vault ATAs.

### `add_liquidity`

Deposits token A and/or token B to receive LP tokens representing a share of the pool.

| Argument | Type | Description |
|----------|------|-------------|
| `amount_a` | `u64` | Token A to deposit. |
| `amount_b` | `u64` | Token B to deposit. |
| `min_lp_out` | `u64` | Minimum LP tokens to receive (slippage guard). |

The first deposit establishes the initial price. Subsequent deposits can be imbalanced but LP tokens are minted proportional to the change in the pool invariant D.

### `swap`

Exchanges one token for the other using the StableSwap invariant.

| Argument | Type | Description |
|----------|------|-------------|
| `amount_in` | `u64` | Input token amount. |
| `min_amount_out` | `u64` | Minimum output amount (slippage guard). |
| `a_to_b` | `bool` | `true` for A→B, `false` for B→A. |

### `remove_liquidity`

Burns LP tokens to withdraw a proportional share of both tokens.

| Argument | Type | Description |
|----------|------|-------------|
| `lp_amount` | `u64` | LP tokens to burn. |
| `min_a` | `u64` | Minimum token A to receive (slippage guard). |
| `min_b` | `u64` | Minimum token B to receive (slippage guard). |

## Account Structure

```
Pool PDA  seeds: ["pool", mint_a, mint_b]
├── token_mint_a   — Token A mint address
├── token_mint_b   — Token B mint address
├── vault_a        — Pool's ATA for token A (owned by pool PDA)
├── vault_b        — Pool's ATA for token B (owned by pool PDA)
├── lp_mint        — LP token mint (authority = pool PDA)
├── amplification  — A parameter
└── fee_bps        — Swap fee
```

## Security

- **LP inflation attack prevention:** the first deposit locks `MINIMUM_LIQUIDITY = 1000` as virtual dead shares, ensuring an attacker cannot profit by donating dust to manipulate the LP price
- **Slippage guards:** all instructions accept user-specified minimums and revert if not met
- **Overflow protection:** all math uses `checked_*` arithmetic; Newton–Raphson convergence is capped at 255 iterations
