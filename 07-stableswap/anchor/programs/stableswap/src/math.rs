//! StableSwap invariant math, LP issuance math, and adaptive fee helpers.

use crate::constants::{BASIS_POINTS_DIVISOR, MAX_ITERATIONS};
use crate::errors::StableSwapError;
use anchor_lang::prelude::*;

// ─── Stableswap invariant ────────────────────────────────────────────────────
//
// For a 2-token pool the StableSwap invariant is:
//
//   4·A·(x + y) + D  =  4·A·D + D³/(4·x·y)
//
// Where:
//   A  = amplification coefficient
//   x  = reserve of token 0
//   y  = reserve of token 1
//   D  = total pool value (invariant)
//
// When A → ∞  the curve approaches constant-sum  (x + y = D, zero slippage)
// When A → 0  the curve approaches constant-product (x·y = k, high slippage)
//
// For n=2 tokens the general Curve formula simplifies to this form.
// We use Newton–Raphson iteration to solve for D and y.
// ─────────────────────────────────────────────────────────────────────────────

/// Fully priced swap result returned by the math layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SwapQuote {
    /// Net output that the user receives after fees.
    pub amount_out: u128,
    /// Fee retained by the pool, denominated in the output token.
    pub fee_amount: u128,
    /// Adaptive fee applied to the trade in basis points.
    pub dynamic_fee_bps: u16,
}

/// Compute the StableSwap invariant D given two reserves and amplification A.
///
/// Uses Newton–Raphson iteration starting from D = x + y.
///
/// # Errors
/// Returns [`StableSwapError::EmptyPool`] if either reserve is zero.
/// Returns [`StableSwapError::ConvergenceFailed`] if the method fails to converge.
pub fn compute_d(reserve_a: u128, reserve_b: u128, amp: u128) -> Result<u128> {
    require!(reserve_a > 0 && reserve_b > 0, StableSwapError::EmptyPool);

    let s = reserve_a
        .checked_add(reserve_b)
        .ok_or(StableSwapError::MathOverflow)?;

    // A·n^n  where n=2  →  4·A
    let ann = amp.checked_mul(4).ok_or(StableSwapError::MathOverflow)?;

    let mut d = s;

    for _ in 0..MAX_ITERATIONS {
        let d_prev = d;

        // D_P = D³ / (4·x·y)  accumulated iteratively as:
        //   d_p = d; d_p = d_p·D / (2·x); d_p = d_p·D / (2·y)
        let dp = d
            .checked_mul(d)
            .ok_or(StableSwapError::MathOverflow)?
            .checked_div(
                reserve_a
                    .checked_mul(2)
                    .ok_or(StableSwapError::MathOverflow)?,
            )
            .ok_or(StableSwapError::MathOverflow)?
            .checked_mul(d)
            .ok_or(StableSwapError::MathOverflow)?
            .checked_div(
                reserve_b
                    .checked_mul(2)
                    .ok_or(StableSwapError::MathOverflow)?,
            )
            .ok_or(StableSwapError::MathOverflow)?;

        // Newton step:
        // D = (ann·S + D_P·2) · D / ((ann - 1)·D + 3·D_P)
        let numerator = ann
            .checked_mul(s)
            .ok_or(StableSwapError::MathOverflow)?
            .checked_add(dp.checked_mul(2).ok_or(StableSwapError::MathOverflow)?)
            .ok_or(StableSwapError::MathOverflow)?
            .checked_mul(d)
            .ok_or(StableSwapError::MathOverflow)?;

        let denominator = ann
            .checked_sub(1)
            .ok_or(StableSwapError::MathOverflow)?
            .checked_mul(d)
            .ok_or(StableSwapError::MathOverflow)?
            .checked_add(dp.checked_mul(3).ok_or(StableSwapError::MathOverflow)?)
            .ok_or(StableSwapError::MathOverflow)?;

        d = numerator
            .checked_div(denominator)
            .ok_or(StableSwapError::MathOverflow)?;

        if d.abs_diff(d_prev) <= 1 {
            return Ok(d);
        }
    }

    Err(StableSwapError::ConvergenceFailed.into())
}

/// Given a new reserve for token i, compute the new reserve for token j
/// such that the invariant D is preserved.
///
/// Solves:   y² + c·y = b·y + c  (Newton–Raphson)
/// where c and b are derived from D and the other reserves.
///
/// # Arguments
/// * `reserve_other` - Current reserve of the token we are NOT solving for
///   (but updated with the new value for the input token)
/// * `d`             - Pre-computed invariant
/// * `amp`           - Amplification coefficient
pub fn compute_y(reserve_other: u128, d: u128, amp: u128) -> Result<u128> {
    require!(reserve_other > 0, StableSwapError::EmptyPool);

    // ann = 4·A  (n=2)
    let ann = amp.checked_mul(4).ok_or(StableSwapError::MathOverflow)?;

    // b = reserve_other + D/ann
    let b = reserve_other
        .checked_add(d.checked_div(ann).ok_or(StableSwapError::MathOverflow)?)
        .ok_or(StableSwapError::MathOverflow)?;

    // c = D³ / (4·ann·reserve_other)
    let c = d
        .checked_mul(d)
        .ok_or(StableSwapError::MathOverflow)?
        .checked_mul(d)
        .ok_or(StableSwapError::MathOverflow)?
        .checked_div(
            reserve_other
                .checked_mul(4)
                .ok_or(StableSwapError::MathOverflow)?
                .checked_mul(ann)
                .ok_or(StableSwapError::MathOverflow)?,
        )
        .ok_or(StableSwapError::MathOverflow)?;

    // Newton–Raphson: y = (y² + c) / (2y + b - D)
    let mut y = d;

    for _ in 0..MAX_ITERATIONS {
        let y_prev = y;

        let numerator = y
            .checked_mul(y)
            .ok_or(StableSwapError::MathOverflow)?
            .checked_add(c)
            .ok_or(StableSwapError::MathOverflow)?;

        let denominator = y
            .checked_mul(2)
            .ok_or(StableSwapError::MathOverflow)?
            .checked_add(b)
            .ok_or(StableSwapError::MathOverflow)?
            .checked_sub(d)
            .ok_or(StableSwapError::MathOverflow)?;

        y = numerator
            .checked_div(denominator)
            .ok_or(StableSwapError::MathOverflow)?;

        if y.abs_diff(y_prev) <= 1 {
            return Ok(y);
        }
    }

    Err(StableSwapError::ConvergenceFailed.into())
}

/// Compute post-trade imbalance in basis points for two oracle-valued sides.
fn calculate_value_imbalance_bps(value_a: u128, value_b: u128) -> Result<u128> {
    let total_value = value_a
        .checked_add(value_b)
        .ok_or(StableSwapError::MathOverflow)?;

    if total_value == 0 {
        return Ok(0);
    }

    Ok(value_a
        .abs_diff(value_b)
        .checked_mul(BASIS_POINTS_DIVISOR)
        .ok_or(StableSwapError::MathOverflow)?
        .checked_div(total_value)
        .ok_or(StableSwapError::MathOverflow)?)
}

/// Increase fees as the pool becomes more imbalanced relative to the oracle.
///
/// The fee ramps linearly from `base_fee_bps` to `max_dynamic_fee_bps` as
/// either the oracle-weighted reserve imbalance or the oracle cross-price
/// deviation approaches the configured depeg threshold.
pub fn calculate_dynamic_fee_bps(
    base_fee_bps: u16,
    max_dynamic_fee_bps: u16,
    new_reserve_in: u128,
    new_reserve_out: u128,
    oracle_price_in: u128,
    oracle_price_out: u128,
    depeg_threshold_bps: u16,
) -> Result<u16> {
    require!(
        base_fee_bps <= max_dynamic_fee_bps,
        StableSwapError::InvalidFeeConfig
    );
    require!(
        depeg_threshold_bps > 0,
        StableSwapError::InvalidDepegThreshold
    );

    let post_value_in = new_reserve_in
        .checked_mul(oracle_price_in)
        .ok_or(StableSwapError::MathOverflow)?;
    let post_value_out = new_reserve_out
        .checked_mul(oracle_price_out)
        .ok_or(StableSwapError::MathOverflow)?;

    let imbalance_bps = calculate_value_imbalance_bps(post_value_in, post_value_out)?;
    let oracle_ratio_bps = oracle_price_in
        .checked_mul(BASIS_POINTS_DIVISOR)
        .ok_or(StableSwapError::MathOverflow)?
        .checked_div(oracle_price_out)
        .ok_or(StableSwapError::MathOverflow)?;
    let oracle_deviation_bps = oracle_ratio_bps.abs_diff(BASIS_POINTS_DIVISOR);
    let stress_bps = imbalance_bps.max(oracle_deviation_bps);
    let stress_cap = stress_bps.min(depeg_threshold_bps as u128);
    let dynamic_range = (max_dynamic_fee_bps - base_fee_bps) as u128;

    let effective_fee = (base_fee_bps as u128)
        .checked_add(
            dynamic_range
                .checked_mul(stress_cap)
                .ok_or(StableSwapError::MathOverflow)?
                .checked_div(depeg_threshold_bps as u128)
                .ok_or(StableSwapError::MathOverflow)?,
        )
        .ok_or(StableSwapError::MathOverflow)?;

    Ok(effective_fee.min(max_dynamic_fee_bps as u128) as u16)
}

/// Calculate the quoted output and adaptive fee for a StableSwap trade.
///
/// # Arguments
/// * `reserve_in` - Current reserve of the input token.
/// * `reserve_out` - Current reserve of the output token.
/// * `amount_in` - Token amount being sold into the pool.
/// * `amp` - StableSwap amplification coefficient.
/// * `base_fee_bps` - Minimum fee charged on a healthy, balanced pool.
/// * `max_dynamic_fee_bps` - Maximum fee charged under pool stress.
/// * `oracle_price_in` - Normalized Pyth price for the input asset.
/// * `oracle_price_out` - Normalized Pyth price for the output asset.
/// * `depeg_threshold_bps` - Peg band used to cap fee escalation.
pub fn calculate_swap_output(
    reserve_in: u128,
    reserve_out: u128,
    amount_in: u128,
    amp: u128,
    base_fee_bps: u16,
    max_dynamic_fee_bps: u16,
    oracle_price_in: u128,
    oracle_price_out: u128,
    depeg_threshold_bps: u16,
) -> Result<SwapQuote> {
    require!(amount_in > 0, StableSwapError::ZeroAmount);

    let d = compute_d(reserve_in, reserve_out, amp)?;

    let new_reserve_in = reserve_in
        .checked_add(amount_in)
        .ok_or(StableSwapError::MathOverflow)?;

    let new_reserve_out = compute_y(new_reserve_in, d, amp)?;

    let amount_out_before_fee = reserve_out
        .checked_sub(new_reserve_out)
        .ok_or(StableSwapError::MathOverflow)?;

    let dynamic_fee_bps = calculate_dynamic_fee_bps(
        base_fee_bps,
        max_dynamic_fee_bps,
        new_reserve_in,
        new_reserve_out,
        oracle_price_in,
        oracle_price_out,
        depeg_threshold_bps,
    )?;

    let fee_amount = amount_out_before_fee
        .checked_mul(dynamic_fee_bps as u128)
        .ok_or(StableSwapError::MathOverflow)?
        .checked_div(BASIS_POINTS_DIVISOR)
        .ok_or(StableSwapError::MathOverflow)?;

    let amount_out = amount_out_before_fee
        .checked_sub(fee_amount)
        .ok_or(StableSwapError::MathOverflow)?;

    Ok(SwapQuote {
        amount_out,
        fee_amount,
        dynamic_fee_bps,
    })
}

/// Compute how many LP tokens to mint for a deposit.
///
/// On the **first deposit** (lp_supply == 0) we mint `D - MINIMUM_LIQUIDITY`
/// tokens and lock MINIMUM_LIQUIDITY as virtual dead shares to prevent
/// the first-depositor inflation attack.
///
/// On **subsequent deposits** we scale by the change in D:
///   mint = lp_supply * (D_after - D_before) / D_before
pub fn calculate_lp_mint_amount(
    reserve_a_before: u128,
    reserve_b_before: u128,
    reserve_a_after: u128,
    reserve_b_after: u128,
    lp_supply: u128,
    amp: u128,
    minimum_liquidity: u64,
) -> Result<u64> {
    if lp_supply == 0 {
        let d = compute_d(reserve_a_after, reserve_b_after, amp)?;
        require!(
            d > minimum_liquidity as u128,
            StableSwapError::InsufficientInitialLiquidity
        );
        let lp_to_mint = (d - minimum_liquidity as u128).min(u64::MAX as u128) as u64;
        return Ok(lp_to_mint);
    }

    let d_before = compute_d(reserve_a_before, reserve_b_before, amp)?;
    let d_after = compute_d(reserve_a_after, reserve_b_after, amp)?;

    require!(d_after >= d_before, StableSwapError::InsufficientLiquidity);

    let d_diff = d_after
        .checked_sub(d_before)
        .ok_or(StableSwapError::MathOverflow)?;

    let lp_to_mint = lp_supply
        .checked_mul(d_diff)
        .ok_or(StableSwapError::MathOverflow)?
        .checked_div(d_before)
        .ok_or(StableSwapError::MathOverflow)?
        .min(u64::MAX as u128) as u64;

    Ok(lp_to_mint)
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Balanced pool: D should stay very close to the sum of reserves.
    #[test]
    fn test_compute_d_balanced() {
        let reserve = 1_000_000_000u128; // 1000 USDC (6 decimals)
        let d = compute_d(reserve, reserve, 100).unwrap();
        // For balanced pool D ≈ 2 * reserve
        assert!(d > 1_900_000_000u128);
        assert!(d < 2_100_000_000u128);
    }

    /// Swapping 1 USDC in a 1M/1M pool should return nearly 1 USDC out.
    #[test]
    fn test_swap_low_slippage() {
        let reserve = 1_000_000_000_000u128; // 1M USDC (6 decimals)
        let amount_in = 1_000_000u128; // 1 USDC
        let quote = calculate_swap_output(
            reserve,
            reserve,
            amount_in,
            100,
            4,
            100,
            1_000_000_000,
            1_000_000_000,
            500,
        )
        .unwrap();
        // With A=100 and tiny trade vs huge pool: almost 1:1
        assert!(quote.amount_out > 990_000u128);
        assert!(quote.amount_out <= amount_in);
        assert_eq!(quote.dynamic_fee_bps, 4);
    }

    /// A large swap should still have low slippage (stablecoin advantage).
    #[test]
    fn test_swap_large_amount_low_slippage() {
        let reserve = 1_000_000_000_000u128; // 1M USDC
        let amount_in = 100_000_000_000u128; // 100k USDC swap (10% of pool)
        let quote = calculate_swap_output(
            reserve,
            reserve,
            amount_in,
            100,
            4,
            100,
            1_000_000_000,
            1_000_000_000,
            500,
        )
        .unwrap();
        // StableSwap should give >99% output for 10% of pool swap
        let ratio = quote.amount_out * 100 / amount_in;
        assert!(ratio >= 98, "Expected >=98% output, got {}%", ratio);
        assert!(quote.dynamic_fee_bps > 4);
    }

    /// First-deposit LP minting reserves dead shares for inflation protection.
    #[test]
    fn test_lp_mint_first_deposit() {
        let amount = 1_000_000_000u128; // 1000 tokens each
        let lp = calculate_lp_mint_amount(0, 0, amount, amount, 0, 100, 1_000).unwrap();
        // LP ≈ D - MINIMUM_LIQUIDITY ≈ 2_000_000_000 - 1_000
        assert!(lp > 1_999_000_000u64);
    }

    /// Subsequent LP issuance tracks the proportional increase in invariant D.
    #[test]
    fn test_lp_mint_subsequent_deposit() {
        let reserve = 1_000_000_000u128;
        let lp_supply = 2_000_000_000u128;
        // Doubling reserves should double LP supply
        let lp = calculate_lp_mint_amount(
            reserve,
            reserve,
            reserve * 2,
            reserve * 2,
            lp_supply,
            100,
            1_000,
        )
        .unwrap();
        // LP minted should be approximately equal to current supply
        assert!(lp > 1_900_000_000u64);
        assert!(lp < 2_100_000_000u64);
    }

    /// Adaptive fees should remain low near peg and increase when imbalanced.
    #[test]
    fn test_dynamic_fee_scales_with_imbalance() {
        let balanced_fee = calculate_dynamic_fee_bps(
            4,
            100,
            1_000_000,
            1_000_000,
            1_000_000_000,
            1_000_000_000,
            500,
        )
        .unwrap();
        let imbalanced_fee = calculate_dynamic_fee_bps(
            4,
            100,
            1_400_000,
            600_000,
            1_000_000_000,
            1_000_000_000,
            500,
        )
        .unwrap();

        assert_eq!(balanced_fee, 4);
        assert!(imbalanced_fee > balanced_fee);
    }
}
