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
    /// Swap fee in basis points (1 bps = 0.01%)
    pub fee_bps: u16,
    /// PDA bump seed
    pub bump: u8,
}

impl Pool {
    pub const LEN: usize = 8   // discriminator
        + 32   // admin
        + 32   // token_mint_a
        + 32   // vault_a
        + 32   // token_mint_b
        + 32   // vault_b
        + 32   // lp_mint
        + 8    // amplification
        + 2    // fee_bps
        + 1;   // bump
}
