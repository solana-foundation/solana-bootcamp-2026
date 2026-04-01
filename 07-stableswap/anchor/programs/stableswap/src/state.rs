//! On-chain account state for the StableSwap pool.

use anchor_lang::prelude::*;

/// A two-token StableSwap liquidity pool.
///
/// Designed for asset pairs expected to trade near 1:1 (e.g. USDC/USDT).
/// Uses the Curve StableSwap invariant to provide extremely low slippage
/// for balanced swaps while still supporting imbalanced positions.
#[account]
pub struct Pool {
    /// Pool creator / admin authority
    pub admin: Pubkey,
    /// Mint address of token A
    pub token_mint_a: Pubkey,
    /// Vault (ATA) that holds the pool's token A reserves
    pub vault_a: Pubkey,
    /// Mint address of token B
    pub token_mint_b: Pubkey,
    /// Vault (ATA) that holds the pool's token B reserves
    pub vault_b: Pubkey,
    /// LP token mint — minted on deposit, burned on withdraw
    pub lp_mint: Pubkey,
    /// StableSwap amplification coefficient (A).
    /// Controls how "stable" the curve is — higher = lower slippage near peg.
    pub amplification: u64,
    /// Baseline swap fee in basis points (1 bps = 0.01%)
    pub base_fee_bps: u16,
    /// Upper bound for the adaptive fee schedule.
    pub max_dynamic_fee_bps: u16,
    /// Maximum allowed deviation from $1.00 before swaps/deposits are halted.
    pub depeg_threshold_bps: u16,
    /// Maximum oracle age tolerated for swap/deposit operations.
    pub max_price_age_sec: u64,
    /// Pyth price feed for token A.
    pub oracle_price_feed_a: Pubkey,
    /// Pyth price feed for token B.
    pub oracle_price_feed_b: Pubkey,
    /// PDA bump seed
    pub bump: u8,
}

impl Pool {
    /// Total account size used when initializing the pool PDA.
    pub const LEN: usize = 8   // discriminator
        + 32   // admin
        + 32   // token_mint_a
        + 32   // vault_a
        + 32   // token_mint_b
        + 32   // vault_b
        + 32   // lp_mint
        + 8    // amplification
        + 2    // base_fee_bps
        + 2    // max_dynamic_fee_bps
        + 2    // depeg_threshold_bps
        + 8    // max_price_age_sec
        + 32   // oracle_price_feed_a
        + 32   // oracle_price_feed_b
        + 1; // bump
}
