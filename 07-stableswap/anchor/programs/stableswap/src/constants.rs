//! Constants that parameterize pool safety checks and fixed-point math.

/// Dead shares locked on first deposit to prevent LP inflation attacks.
/// Equivalent to Uniswap V2's MINIMUM_LIQUIDITY.
pub const MINIMUM_LIQUIDITY: u64 = 1_000;

/// Basis points divisor used throughout fee and peg calculations.
pub const BASIS_POINTS_DIVISOR: u128 = 10_000;

/// Maximum amplification parameter A.
/// Higher A = tighter price curve (less slippage but less flexibility).
/// Curve Finance typically uses 100-2000 for stablecoin pairs.
pub const MAX_AMP: u64 = 1_000_000;

/// Maximum swap fee in basis points (100% = 10_000 bps).
pub const MAX_FEE_BPS: u16 = 10_000;

/// Maximum allowed depeg threshold. Pools tighter than this still provide
/// sensible protection, while avoiding configurations that effectively disable
/// the oracle guard.
pub const MAX_DEPEG_THRESHOLD_BPS: u16 = 5_000;

/// Maximum Newton's method iterations for invariant computation.
pub const MAX_ITERATIONS: u8 = 255;

/// Pyth prices are normalized to 1e9 fixed-point precision inside the program.
pub const ORACLE_PRICE_SCALE: u128 = 1_000_000_000;

/// Target stablecoin price in the normalized oracle precision.
pub const TARGET_STABLE_PRICE: u128 = ORACLE_PRICE_SCALE;

/// Internal exponent associated with ORACLE_PRICE_SCALE.
pub const ORACLE_TARGET_EXPONENT: i32 = -9;
