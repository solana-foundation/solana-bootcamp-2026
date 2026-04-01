# StableSwap AMM

A two-token liquidity pool optimized for stablecoin pairs, built with [Anchor](https://www.anchor-lang.com/) on Solana. It uses a Curve-style hybrid invariant with Newton's method, adaptive fees, and Pyth-based depeg protection so the pool behaves like a stable swap rather than a constant-product AMM.

## How It Works

Standard AMMs (Uniswap-style `x·y = k`) have significant price impact even for modest trades. For assets that should trade near 1:1, the StableSwap invariant:

```
4·A·(x + y) + D  =  4·A·D + D³ / (4·x·y)
```

concentrates liquidity around the peg, flattening the curve near 1:1 and eliminating unnecessary slippage for balanced stablecoin trades while still self-correcting as the pool drifts away from parity.

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
anchor build
cargo test -p stableswap --test litesvm
```

The integration suite is written in Rust and runs against `anchor-litesvm`. It provisions local SPL mints, ATAs, and raw Pyth-compatible oracle accounts inside LiteSVM to cover:
- Pool initialization
- Adding initial liquidity
- Swapping through the `remaining_accounts` path
- Oracle-enforced depeg pauses
- Proportional liquidity withdrawal

## Instructions

### `initialize_pool`

Creates a new pool for a token A / token B pair.

| Argument | Type | Description |
|----------|------|-------------|
| `amplification` | `u64` | A parameter (1–1,000,000). Typical: 100–2000 for stablecoins. |
| `base_fee_bps` | `u16` | Minimum swap fee in basis points. |
| `max_dynamic_fee_bps` | `u16` | Fee cap when the pool becomes imbalanced. |
| `depeg_threshold_bps` | `u16` | Maximum allowed drift from $1 before swaps and deposits halt. |
| `max_price_age_sec` | `u64` | Maximum accepted Pyth price age. |

Automatically creates the pool PDA, LP mint, two vault ATAs, and stores the two Pyth feed addresses used to guard the pool.

### `add_liquidity`

Deposits token A and/or token B to receive LP tokens representing a share of the pool.

| Argument | Type | Description |
|----------|------|-------------|
| `amount_a` | `u64` | Token A to deposit. |
| `amount_b` | `u64` | Token B to deposit. |
| `min_lp_out` | `u64` | Minimum LP tokens to receive (slippage guard). |

The first deposit establishes the initial price. Subsequent deposits can be imbalanced but LP tokens are minted proportional to the change in the pool invariant D.

### `swap`

Exchanges one token for the other using the StableSwap invariant. Fees are raised dynamically as the post-trade pool becomes more imbalanced relative to the Pyth oracle prices.

| Argument | Type | Description |
|----------|------|-------------|
| `amount_in` | `u64` | Input token amount. |
| `min_amount_out` | `u64` | Minimum output amount (slippage guard). |
| `input_index` | `u8` | Pool token index to sell. |
| `output_index` | `u8` | Pool token index to receive. |

### `remove_liquidity`

Burns LP tokens to withdraw a proportional share of both tokens. Withdrawals remain available even when swaps/deposits are halted by oracle risk checks.

| Argument | Type | Description |
|----------|------|-------------|
| `lp_amount` | `u64` | LP tokens to burn. |
| `min_a` | `u64` | Minimum token A to receive (slippage guard). |
| `min_b` | `u64` | Minimum token B to receive (slippage guard). |

## Account Structure

```
Pool PDA  seeds: ["pool", mint_a, mint_b]
Pool PDA  seeds: ["pool", lp_mint]
├── token_mint_a   — Token A mint address
├── token_mint_b   — Token B mint address
├── vault_a        — Pool's ATA for token A (owned by pool PDA)
├── vault_b        — Pool's ATA for token B (owned by pool PDA)
├── lp_mint        — LP token mint (authority = pool PDA)
├── amplification        — Hybrid invariant amplification factor
├── base_fee_bps         — Minimum fee
├── max_dynamic_fee_bps  — Adaptive fee cap
├── depeg_threshold_bps  — Oracle pause band
├── max_price_age_sec    — Oracle freshness guard
├── oracle_price_feed_a  — Pyth price feed for token A
└── oracle_price_feed_b  — Pyth price feed for token B
```

## Security

- **LP inflation attack prevention:** the first deposit locks `MINIMUM_LIQUIDITY = 1000` as virtual dead shares, ensuring an attacker cannot profit by donating dust to manipulate the LP price
- **Slippage guards:** all instructions accept user-specified minimums and revert if not met
- **Dynamic fee protection:** fees rise as the pool moves away from the oracle-implied balance, which reduces pure arbitrage extraction against LPs
- **Pyth oracle protection:** swaps and deposits halt when either stablecoin moves outside the configured peg band or the oracle price goes stale
- **Overflow protection:** all math uses `checked_*` arithmetic; Newton–Raphson convergence is capped at 255 iterations
