# Anchor Escrow

A token escrow program built with [Anchor](https://www.anchor-lang.com/) on Solana.

## Why Build an Escrow?

An escrow is one of the best programs to learn when getting started with Solana development. It covers essential concepts including:

- Creating and managing PDAs (Program Derived Addresses)
- Working with SPL tokens and associated token accounts
- Handling CPIs (Cross-Program Invocations)
- Managing program state and account relationships
- Implementing secure token transfers between parties

These patterns form the foundation for more complex DeFi applications like AMMs, lending protocols, and marketplaces.

## Overview

This program enables trustless token swaps between two parties:

- **Make** - Create an escrow by depositing tokens and specifying desired tokens in return
- **Take** - Complete the swap by providing the requested tokens
- **Refund** - Cancel the escrow and reclaim deposited tokens

## Build

```bash
anchor build
```

## Test

This project uses [LiteSVM](https://github.com/LiteSVM/litesvm) for fast, lightweight testing without needing a local validator.

Testing crates used:

- [litesvm](https://crates.io/crates/litesvm) - Lightweight Solana VM for testing
- [litesvm-token](https://crates.io/crates/litesvm-token) - Token helpers for LiteSVM
- [anchor-litesvm](https://crates.io/crates/anchor-litesvm) - Anchor integration for LiteSVM

```bash
cargo test
```

## Resources

- [Anchor Documentation](https://www.anchor-lang.com/)
- [LiteSVM Documentation](https://litesvm.com/)

## Program ID

`8F3byNyXVHzfmjKK9J2cxvVbKzRiVYh8icoprMUqSFmb`
