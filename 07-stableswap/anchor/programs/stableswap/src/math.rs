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
    let ann = amp
        .checked_mul(4)
        .ok_or(StableSwapError::MathOverflow)?;

    let mut d = s;

    for _ in 0..255u8 {
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
    let ann = amp
        .checked_mul(4)
        .ok_or(StableSwapError::MathOverflow)?;

    // b = reserve_other + D/ann
    let b = reserve_other
        .checked_add(
            d.checked_div(ann).ok_or(StableSwapError::MathOverflow)?,
        )
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

    for _ in 0..255u8 {
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

/// Calculate the output amount for a swap, after applying the fee.
///
/// # Returns
/// `(amount_out, fee_amount)`
pub fn calculate_swap_output(
    reserve_in: u128,
    reserve_out: u128,
    amount_in: u128,
    amp: u128,
    fee_bps: u16,
) -> Result<(u128, u128)> {
    require!(amount_in > 0, StableSwapError::ZeroAmount);

    let d = compute_d(reserve_in, reserve_out, amp)?;

    let new_reserve_in = reserve_in
        .checked_add(amount_in)
        .ok_or(StableSwapError::MathOverflow)?;

    let new_reserve_out = compute_y(new_reserve_in, d, amp)?;

    let amount_out_before_fee = reserve_out
        .checked_sub(new_reserve_out)
        .ok_or(StableSwapError::MathOverflow)?;

    let fee = amount_out_before_fee
        .checked_mul(fee_bps as u128)
        .ok_or(StableSwapError::MathOverflow)?
        .checked_div(10_000)
        .ok_or(StableSwapError::MathOverflow)?;

    let amount_out = amount_out_before_fee
        .checked_sub(fee)
        .ok_or(StableSwapError::MathOverflow)?;

    Ok((amount_out, fee))
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
        let lp_to_mint = (d - minimum_liquidity as u128)
            .min(u64::MAX as u128) as u64;
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

    /// Balanced pool: D should equal sum of reserves.
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
        let amount_in = 1_000_000u128;       // 1 USDC
        let (out, _fee) = calculate_swap_output(reserve, reserve, amount_in, 100, 4).unwrap();
        // With A=100 and tiny trade vs huge pool: almost 1:1
        assert!(out > 990_000u128);
        assert!(out <= amount_in);
    }

    /// A large swap should still have low slippage (stablecoin advantage).
    #[test]
    fn test_swap_large_amount_low_slippage() {
        let reserve = 1_000_000_000_000u128; // 1M USDC
        let amount_in = 100_000_000_000u128; // 100k USDC swap (10% of pool)
        let (out, _fee) = calculate_swap_output(reserve, reserve, amount_in, 100, 4).unwrap();
        // StableSwap should give >99% output for 10% of pool swap
        let ratio = out * 100 / amount_in;
        assert!(ratio > 98, "Expected >98% output, got {}%", ratio);
    }

    /// First-deposit LP minting.
    #[test]
    fn test_lp_mint_first_deposit() {
        let amount = 1_000_000_000u128; // 1000 tokens each
        let lp = calculate_lp_mint_amount(0, 0, amount, amount, 0, 100, 1_000).unwrap();
        // LP ≈ D - MINIMUM_LIQUIDITY ≈ 2_000_000_000 - 1_000
        assert!(lp > 1_999_000_000u64);
    }

    /// LP amount grows proportionally on subsequent deposits.
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
}
