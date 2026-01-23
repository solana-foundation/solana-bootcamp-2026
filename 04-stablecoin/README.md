# Stablecoin Program

A Solana stablecoin program built with Anchor that implements minting controls, allowance management, and pause functionality.

## Features

- **Controlled Minting**: Only authorized minters can create new tokens, each with a configurable allowance
- **Allowance System**: Track and limit how many tokens each minter can create
- **Pause/Unpause**: Admin can pause all minting operations in emergencies
- **Token Burning**: Users can burn their own tokens

## Prerequisites

- [Rust](https://rustup.rs/)
- [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools)
- [Anchor](https://www.anchor-lang.com/docs/installation) v1.0.0-rc.2+

## Building

```bash
anchor build
```

## Testing

This project uses [litesvm](https://github.com/LiteSVM/litesvm) for fast, in-process testing:

```bash
cargo test
```

## Program Instructions

### `initialize`
Creates the stablecoin mint and config account. The config PDA becomes the mint authority.

### `configure_minter`
Authorizes a minter with a specific allowance. Can also update an existing minter's allowance. Admin only.

### `remove_minter`
Revokes a minter's authorization and closes their config account. Admin only.

### `mint_tokens`
Mints tokens to a destination account. Requires:
- Caller is an authorized minter
- Amount doesn't exceed remaining allowance
- Program is not paused

### `burn_tokens`
Burns tokens from the caller's token account.

### `pause` / `unpause`
Pauses or resumes minting operations. Admin only.

## Accounts

### Config
Stores global stablecoin settings:
- `admin` - Authority that can manage minters and pause
- `mint` - The stablecoin mint address
- `paused` - Whether minting is paused

### MinterConfig
Per-minter settings:
- `minter` - The minter's public key
- `allowance` - Maximum tokens this minter can create
- `amount_minted` - Tokens already minted by this minter

## License

MIT
