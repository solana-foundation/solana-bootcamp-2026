//! Error codes emitted by the StableSwap program.

use anchor_lang::prelude::*;

/// Domain-specific failures surfaced by instruction handlers and helpers.
#[error_code]
pub enum StableSwapError {
    /// The configured amplification coefficient is outside the supported range.
    #[msg("Amplification parameter must be between 1 and 1,000,000")]
    InvalidAmplification,

    /// A fee input exceeded the universal basis-point ceiling.
    #[msg("Fee exceeds maximum of 100%")]
    InvalidFee,

    /// The fee schedule is internally inconsistent.
    #[msg("Dynamic fee configuration is invalid")]
    InvalidFeeConfig,

    /// The caller configured an unsupported peg deviation threshold.
    #[msg("Depeg threshold is outside the supported range")]
    InvalidDepegThreshold,

    /// Oracle freshness windows must be positive.
    #[msg("Oracle price maximum age must be greater than zero")]
    InvalidOracleAge,

    /// User-provided slippage bounds were not met.
    #[msg("Slippage exceeded: output less than minimum")]
    SlippageExceeded,

    /// The pool does not have enough liquidity for the requested action.
    #[msg("Insufficient liquidity in pool")]
    InsufficientLiquidity,

    /// Checked arithmetic overflowed or divided by zero.
    #[msg("Math overflow")]
    MathOverflow,

    /// Zero-value operations are rejected to avoid ambiguous behavior.
    #[msg("Zero amount not allowed")]
    ZeroAmount,

    /// Newton iteration failed to converge within the configured limit.
    #[msg("Convergence failed in Newton's method")]
    ConvergenceFailed,

    /// The pool must hold reserves on both sides before pricing swaps.
    #[msg("Pool is empty")]
    EmptyPool,

    /// Initial liquidity was too small after subtracting locked LP shares.
    #[msg("Initial liquidity too small")]
    InsufficientInitialLiquidity,

    /// Liquidity withdrawals must redeem a proportional share of both assets.
    #[msg("Single-sided withdrawals are not supported; LP exits must stay proportional")]
    SingleSidedWithdrawalNotAllowed,

    /// The caller referenced a token slot outside the supported token list.
    #[msg("Invalid token index")]
    InvalidTokenIndex,

    /// Swapping a token into itself is always nonsensical.
    #[msg("Input and output token must be different")]
    SameTokenSwap,

    /// The swap instruction requires the expected remaining account layout.
    #[msg("Invalid remaining accounts for swap instruction")]
    InvalidRemainingAccounts,

    /// A supplied token vault does not match the pool configuration.
    #[msg("Invalid vault account")]
    InvalidVault,

    /// A supplied mint does not match the pool configuration.
    #[msg("Invalid token mint")]
    InvalidMint,

    /// Stable pairs are expected to share the same token precision.
    #[msg("Both token mints must use the same decimals for this StableSwap pool")]
    InvalidMintDecimals,

    /// The provided Pyth account failed key or layout validation.
    #[msg("Invalid oracle account")]
    InvalidOracleAccount,

    /// The provided system program account does not match the canonical ID.
    #[msg("Invalid system program account")]
    InvalidSystemProgram,

    /// The provided SPL token program account does not match the canonical ID.
    #[msg("Invalid token program account")]
    InvalidTokenProgram,

    /// The provided associated token program account does not match the canonical ID.
    #[msg("Invalid associated token program account")]
    InvalidAssociatedTokenProgram,

    /// The oracle value is older than the configured freshness threshold.
    #[msg("Oracle price is stale")]
    StaleOraclePrice,

    /// The oracle reported a non-positive or otherwise unusable price.
    #[msg("Oracle price is invalid")]
    InvalidOraclePrice,

    /// Trading and deposits are halted while a stablecoin is off peg.
    #[msg("Pool is paused because one of the stablecoins is outside the peg band")]
    PoolPaused,
}
