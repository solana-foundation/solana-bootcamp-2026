//! Adaptive fee helpers for oracle-aware stablecoin swaps.

use anchor_lang::prelude::*;

use crate::constants::BASIS_POINTS_DIVISOR;
use crate::errors::StableSwapError;

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

#[cfg(test)]
mod tests {
    use super::*;

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
