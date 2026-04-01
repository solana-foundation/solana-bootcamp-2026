//! On-chain account state for the StableSwap pool.

use crate::constants::NUM_TOKENS;
use anchor_lang::prelude::*;

/// Oracle configuration used for depeg detection and emergency fee escalation.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct OracleConfig {
    /// Pyth feed used for token A.
    pub oracle_a: Pubkey,
    /// Pyth feed used for token B.
    pub oracle_b: Pubkey,
    /// Maximum allowed deviation from $1 before the pool should pause.
    pub max_depeg_bps: u16,
    /// Emergency fee ceiling used when oracle protection is enabled.
    pub emergency_fee_bps: u16,
    /// Whether oracle-based depeg protection is enabled for this pool.
    pub enabled: bool,
}

impl OracleConfig {
    /// Serialized size of the oracle config inside the pool account.
    pub const LEN: usize = 32 + 32 + 2 + 2 + 1;
}

/// A two-token StableSwap liquidity pool.
///
/// Designed for asset pairs expected to trade near 1:1 (e.g. USDC/USDT).
/// Uses the Curve StableSwap invariant to provide extremely low slippage
/// for balanced swaps while still supporting imbalanced positions.
#[account]
pub struct Pool {
    /// Pool creator / admin authority
    pub admin: Pubkey,
    /// LP token mint — minted on deposit, burned on withdraw
    pub lp_mint: Pubkey,
    /// StableSwap amplification coefficient (A).
    /// Controls how "stable" the curve is — higher = lower slippage near peg.
    pub amplification: u64,
    /// Baseline swap fee in basis points (1 bps = 0.01%).
    pub fee_bps: u16,
    /// Token mint addresses for the pair.
    pub token_mints: [Pubkey; NUM_TOKENS],
    /// PDA bump seed
    pub bump: u8,
    /// Oracle settings used for depeg detection.
    pub oracle_config: OracleConfig,
    /// Persisted pause state updated by the dedicated `check_depeg` instruction.
    pub is_paused: bool,
}

impl Pool {
    /// Total account size used when initializing the pool PDA.
    pub const LEN: usize = 8   // discriminator
        + 32   // admin
        + 32   // lp_mint
        + 8    // amplification
        + 2    // fee_bps
        + (32 * NUM_TOKENS) // token_mints
        + 1    // bump
        + OracleConfig::LEN
        + 1; // is_paused

    /// Return the pool's token mints in canonical order.
    pub fn mints(&self) -> &[Pubkey; NUM_TOKENS] {
        &self.token_mints
    }

    /// Find the index of a mint inside `token_mints`.
    pub fn find_mint_index(&self, mint: &Pubkey) -> Option<usize> {
        self.token_mints
            .iter()
            .position(|stored_mint| stored_mint == mint)
    }
}
