use anchor_lang::prelude::*;

#[error_code]
pub enum StableSwapError {
    #[msg("Amplification parameter must be between 1 and 1,000,000")]
    InvalidAmplification,

    #[msg("Fee exceeds maximum of 100%")]
    InvalidFee,

    #[msg("Slippage exceeded: output less than minimum")]
    SlippageExceeded,

    #[msg("Insufficient liquidity in pool")]
    InsufficientLiquidity,

    #[msg("Math overflow")]
    MathOverflow,

    #[msg("Zero amount not allowed")]
    ZeroAmount,

    #[msg("Convergence failed in Newton's method")]
    ConvergenceFailed,

    #[msg("Pool is empty")]
    EmptyPool,

    #[msg("Initial liquidity too small")]
    InsufficientInitialLiquidity,

    #[msg("Invalid vault account")]
    InvalidVault,

    #[msg("Invalid token mint")]
    InvalidMint,
}
