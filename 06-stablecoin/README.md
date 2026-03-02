# Stablecoin Program

A Solana stablecoin program built with [Anchor](https://www.anchor-lang.com/) that demonstrates how to issue and manage a Token-2022 (Token Extensions) token with controlled minting, allowance management, and emergency pause functionality.

## Overview

The program creates a Token-2022 mint whose authority is a PDA owned by the program itself, so no single private key controls issuance. An admin account manages a set of authorized minters, each with an individual allowance that caps how many tokens they can mint in total. This pattern mirrors how real-world stablecoins (e.g. USDC) segregate the admin role from individual mint operators.

## Features

- **Token-2022 mint** — uses the Token Extensions program (`TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb`)
- **Controlled minting** — only authorized minters can create new tokens
- **Per-minter allowances** — each minter has a cap on their cumulative mint volume
- **Emergency pause** — admin can halt all minting instantly
- **Token burning** — any user can burn their own tokens (e.g. for fiat redemption)
- **Rent reclamation** — removing a minter closes their config account and returns rent to the admin

## Program ID

```
rYXfi25x9JMgau82aGMJMVUokq7JzueqehiJUmwR97Q
```

## Prerequisites

- [Rust](https://rustup.rs/) (see `rust-toolchain.toml` for the required version)
- [Solana CLI](https://solana.com/developers/guides/getstarted/setup-local-development)
- [Anchor CLI](https://www.anchor-lang.com/docs/installation) v1.0.0-rc.2

## Building

```bash
anchor build
```

## Testing

Tests use [LiteSVM](https://github.com/LiteSVM/litesvm) — an in-process Solana VM that runs tests without a local validator, making them fast and deterministic.

```bash
cargo test -p stablecoin
```

All 18 tests should pass.

## Instructions

### `initialize`

Creates the stablecoin mint and config account. Must be called once before any other instruction.

- Creates a `Config` PDA at seeds `["config"]`
- Creates a Token-2022 mint PDA at seeds `["mint"]`
- Sets the `Config` PDA as both mint authority and freeze authority

### `configure_minter`

Authorizes a new minter or updates an existing minter's allowance. Admin only.

```
allowance: u64  — cumulative token cap for this minter
```

### `remove_minter`

Revokes a minter's authorization. Closes the `MinterConfig` account and returns rent to the admin. Admin only.

### `mint_tokens`

Mints tokens to an arbitrary destination account (created as an ATA if it doesn't exist). Reverts if:

- The program is paused
- The caller has no `MinterConfig`
- The requested amount exceeds the minter's remaining allowance (`allowance - amount_minted`)

### `burn_tokens`

Burns tokens from the caller's own token account. Anyone can call this instruction.

### `pause` / `unpause`

Toggles the global `paused` flag on the `Config` account. When paused, all `mint_tokens` calls revert. Admin only.

## Account Structure

### `Config` — PDA seeds: `["config"]`

| Field       | Type   | Description                              |
|-------------|--------|------------------------------------------|
| `admin`     | Pubkey | Can manage minters and pause/unpause     |
| `mint`      | Pubkey | The Token-2022 mint address              |
| `paused`    | bool   | Minting disabled when `true`             |
| `bump`      | u8     | PDA bump seed                            |
| `mint_bump` | u8     | Mint PDA bump seed                       |

### `MinterConfig` — PDA seeds: `["minter", minter_pubkey]`

| Field            | Type   | Description                              |
|------------------|--------|------------------------------------------|
| `minter`         | Pubkey | The authorized minter's public key       |
| `allowance`      | u64    | Total tokens this minter may ever mint   |
| `amount_minted`  | u64    | Tokens minted so far                     |
| `is_initialized` | bool   | Set on first `configure_minter` call     |
| `bump`           | u8     | PDA bump seed                            |

## Token-2022 Notes

The mint is created using the **Token Extensions program** (`Token-2022`). Associated token accounts for this mint use a different ATA derivation path than legacy SPL Token accounts — the token program ID is included as a seed:

```
ATA = find_program_address(
    [wallet, TOKEN_2022_PROGRAM_ID, mint],
    ATA_PROGRAM_ID
)
```

All CPI calls (mint, burn) target `anchor_spl::token_2022::ID` directly, which is the pattern required by Anchor v1.0.0-rc.2.

## License

MIT
