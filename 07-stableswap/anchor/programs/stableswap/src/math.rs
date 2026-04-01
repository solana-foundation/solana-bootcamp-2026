//! StableSwap invariant math and LP issuance helpers.

use crate::constants::{BASIS_POINTS_DIVISOR, MAX_ITERATIONS};
use crate::dynamic_fees::calculate_dynamic_fee_bps;
use crate::errors::StableSwapError;
use anchor_lang::prelude::*;

// ============================================================================
// STABLESWAP MATH
// ============================================================================
//
// This file contains the mathematical core of the StableSwap AMM.
// Understanding this math is the key to understanding how the protocol works.
//
// BACKGROUND: Why StableSwap?
//
// Uniswap uses the "constant product" formula:
//
//   x * y = k
//
// That works well for volatile assets, but it is inefficient for stablecoins.
//
// Example with a constant-product AMM:
// - Pool: 1,000,000 USDC and 1,000,000 USDT
// - k = 1,000,000 * 1,000,000 = 10^12
// - Swap 10,000 USDC for USDT
// - New USDC reserve: 1,010,000
// - New USDT reserve: 10^12 / 1,010,000 = 990,099
// - Output: 1,000,000 - 990,099 = 9,901 USDT
//
// That is roughly 1% slippage for a 1% trade, which is too expensive for
// assets that are expected to remain near the same price.
//
// StableSwap does much better for correlated assets. The curve is:
// - Flat in the middle, where the pair should trade near 1:1
// - Curved at the edges, so the pool still protects itself when imbalanced
//
// You can think of it as blending two extremes:
// - Constant sum near the peg, which minimizes unnecessary slippage
// - Constant product further from the peg, which prevents the pool from being
//   completely drained when the pair moves off balance
//
// THE STABLESWAP INVARIANT
//
// In the general n-token form:
//
//   A * n^n * sum(x_i) + D = A * D * n^n + D^(n+1) / (n^n * prod(x_i))
//
// Where:
// - A = amplification coefficient
// - n = number of tokens
// - x_i = reserve of token i
// - D = the invariant, which represents the pool's normalized total value
//
// For a 2-token pool, the equation simplifies to:
//
//   4A(x + y) + D = 4AD + D^3 / (4xy)
//
// The A parameter blends between two extremes:
// - A -> 0: the product term dominates, so the curve behaves like
//   constant product
// - A -> infinity: the amplified sum term dominates, so the curve behaves
//   like constant sum and D approaches x + y
//
// WHAT IS D?
//
// D is the "total value" of the pool in a normalized way. If all tokens were
// equally priced, D is the amount you would intuitively expect the pool to be
// worth.
//
// For a balanced pool with 1M of each token:
//
//   D ≈ 2,000,000
//
// During swaps, D stays constant. That is what "invariant" means.
// During add/remove liquidity operations, D increases or decreases.
//
// WHY NEWTON'S METHOD?
//
// The invariant is polynomial in D and y, so there is no simple closed-form
// solution we want to use on chain. We instead use Newton-Raphson iteration:
//
// 1. Start with an initial guess
// 2. Measure how far off that guess is
// 3. Compute a better next guess
// 4. Repeat until the value stabilizes
//
// In this implementation:
// - `compute_d` starts with D = x + y
// - `compute_y` starts with y = D
// - Both iterations stop once the answer changes by at most 1 unit
//
// This usually converges quickly for the pool sizes and amplification factors
// used in stablecoin markets.
// ============================================================================

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
/// Conceptually:
/// 1. Guess that the pool's total value is the sum of reserves.
/// 2. Refine that guess using the invariant equation.
/// 3. Repeat until the guess stops moving.
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
        // This is algebraically equivalent to:
        //   D_next = D - f(D) / f'(D)
        // but written in a reduced form that is cheaper and safer to evaluate
        // with checked integer arithmetic on chain.
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
/// Solves:   y² + c = y(2y + b - D)  via Newton–Raphson
/// where c and b are derived from D and the other reserves.
///
/// In swap terms:
/// - the input reserve moves first because the trader adds tokens in
/// - D is kept constant
/// - we solve for the output reserve that keeps the pool on the same curve
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

    // Newton–Raphson update for the output reserve.
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

/// Calculate the quoted output and adaptive fee for a StableSwap trade.
///
/// This function follows the standard StableSwap swap flow:
/// 1. Compute the current invariant D from the existing reserves.
/// 2. Add the trader's input amount to the input-side reserve.
/// 3. Solve for the new output-side reserve that preserves D.
/// 4. The raw output is the amount removed from the output reserve.
/// 5. Apply dynamic fees based on post-trade imbalance and oracle conditions.
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
/// LP issuance is based on how much the invariant D grows, not just on one
/// token balance. That keeps LP accounting aligned with the StableSwap curve.
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

/// Compute the proportional token amounts owed for an LP burn.
///
/// Withdrawals in this pool are intentionally dual-sided only: an LP burns a
/// share of the total supply and receives that same share of each reserve.
/// This prevents the instruction from becoming a single-sided exit path.
///
/// The caller should pass the live reserves for both tokens and the LP supply
/// adjusted for the protocol's virtual dead shares.
pub fn calculate_withdraw_amounts(
    reserves: &[u128],
    lp_amount: u128,
    adjusted_supply: u128,
) -> Result<Vec<u64>> {
    require!(reserves.len() == 2, StableSwapError::InvalidVault);
    require!(lp_amount > 0, StableSwapError::ZeroAmount);
    require!(adjusted_supply > 0, StableSwapError::EmptyPool);
    require!(
        reserves.iter().all(|reserve| *reserve > 0),
        StableSwapError::EmptyPool
    );

    let mut withdraw_amounts = Vec::with_capacity(reserves.len());
    for reserve in reserves {
        let amount = reserve
            .checked_mul(lp_amount)
            .ok_or(StableSwapError::MathOverflow)?
            .checked_div(adjusted_supply)
            .ok_or(StableSwapError::MathOverflow)?;
        withdraw_amounts.push(amount.min(u64::MAX as u128) as u64);
    }

    require!(
        withdraw_amounts.iter().all(|amount| *amount > 0),
        StableSwapError::SingleSidedWithdrawalNotAllowed
    );

    Ok(withdraw_amounts)
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

    /// LP burns must return a proportional share of both reserves.
    #[test]
    fn test_calculate_withdraw_amounts_proportional() {
        let withdraw_amounts =
            calculate_withdraw_amounts(&[1_000_000u128, 1_000_000u128], 100_000, 1_000_000)
                .unwrap();

        assert_eq!(withdraw_amounts, vec![100_000u64, 100_000u64]);
    }

    /// Dust burns that would collapse into a one-sided exit are rejected.
    #[test]
    fn test_calculate_withdraw_amounts_rejects_single_sided_rounding() {
        let err = calculate_withdraw_amounts(&[1u128, 1_000_000u128], 1, 1_000_000).unwrap_err();

        assert!(err
            .to_string()
            .contains("Single-sided withdrawals are not supported"));
    }
}
