/// Dead shares locked on first deposit to prevent LP inflation attacks.
/// Equivalent to Uniswap V2's MINIMUM_LIQUIDITY.
pub const MINIMUM_LIQUIDITY: u64 = 1_000;

/// Maximum amplification parameter A.
/// Higher A = tighter price curve (less slippage but less flexibility).
/// Curve Finance typically uses 100-2000 for stablecoin pairs.
pub const MAX_AMP: u64 = 1_000_000;

/// Maximum swap fee in basis points (100% = 10_000 bps).
pub const MAX_FEE_BPS: u16 = 10_000;

/// Maximum Newton's method iterations for invariant computation.
pub const MAX_ITERATIONS: u8 = 255;
