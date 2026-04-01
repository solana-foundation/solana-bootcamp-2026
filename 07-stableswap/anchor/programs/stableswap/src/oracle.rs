//! Pyth oracle helpers used to enforce peg protection and normalize prices.

use std::mem::size_of;

use anchor_lang::prelude::*;
use bytemuck::{try_from_bytes, Pod, Zeroable};

use crate::constants::{
    BASIS_POINTS_DIVISOR, ORACLE_PRICE_SCALE, ORACLE_TARGET_EXPONENT, TARGET_STABLE_PRICE,
};
use crate::errors::StableSwapError;

/// Pyth account discriminator used to validate raw account data.
const PYTH_MAGIC: u32 = 0xa1b2c3d4;
/// Supported Pyth account version for the embedded layout below.
const PYTH_VERSION_2: u32 = 2;
/// Pyth account type value representing a price account.
const PYTH_ACCOUNT_TYPE_PRICE: u32 = 3;
/// Pyth status value meaning the aggregate price is actively trading.
const PYTH_STATUS_TRADING: u8 = 1;
/// Number of component publisher slots stored in a legacy Pyth price account.
const PYTH_NUM_COMPONENTS: usize = 32;

/// Minimal in-program representation of Pyth's `PriceInfo` struct.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
struct PythPriceInfo {
    /// Aggregate or publisher price value.
    price: i64,
    /// Confidence interval around `price`.
    conf: u64,
    /// Pyth status enum encoded as a byte.
    status: u8,
    /// Corporate action flag from Pyth.
    corp_act: u8,
    /// Padding bytes required by the canonical account layout.
    padding: [u8; 6],
    /// Slot in which the price was published.
    pub_slot: u64,
}

/// Minimal representation of Pyth's rational EMA fields.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
struct PythRational {
    /// Pre-computed integer value for convenience.
    val: i64,
    /// Rational numerator.
    numer: i64,
    /// Rational denominator.
    denom: i64,
}

/// Single publisher contribution entry inside the Pyth price account.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
struct PythPriceComp {
    /// Publisher authority key.
    publisher: Pubkey,
    /// Price contribution used in the current aggregate.
    agg: PythPriceInfo,
    /// Publisher's latest unpublished contribution.
    latest: PythPriceInfo,
}

/// Legacy Solana Pyth price account layout parsed directly from account data.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct PythPriceAccount {
    /// Magic header for account validation.
    magic: u32,
    /// Pyth version number.
    ver: u32,
    /// Pyth account type discriminator.
    atype: u32,
    /// Serialized size recorded by the account itself.
    size: u32,
    /// Price type discriminator.
    ptype: u32,
    /// Base-10 exponent used by all price values in this account.
    expo: i32,
    /// Number of active component prices.
    num: u32,
    /// Number of component prices included in the aggregate.
    num_qt: u32,
    /// Last slot with a valid aggregate.
    last_slot: u64,
    /// Slot threshold used by Pyth for validity.
    valid_slot: u64,
    /// EMA price.
    ema_price: PythRational,
    /// EMA confidence.
    ema_conf: PythRational,
    /// Publish timestamp for the aggregate.
    timestamp: i64,
    /// Minimum publishers required for validity.
    min_pub: u8,
    /// Reserved field from the canonical layout.
    drv2: u8,
    /// Reserved field from the canonical layout.
    drv3: u16,
    /// Reserved field from the canonical layout.
    drv4: u32,
    /// Linked product account.
    prod: Pubkey,
    /// Linked next price account.
    next: Pubkey,
    /// Previous valid slot.
    prev_slot: u64,
    /// Previous valid trading price.
    prev_price: i64,
    /// Previous valid confidence.
    prev_conf: u64,
    /// Previous valid publish timestamp.
    prev_timestamp: i64,
    /// Current aggregate price info.
    agg: PythPriceInfo,
    /// Per-publisher contributions.
    comp: [PythPriceComp; PYTH_NUM_COMPONENTS],
}

/// Normalized pairwise oracle view used by instruction handlers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OracleStatus {
    /// Token A price normalized to the program's fixed-point scale.
    pub price_a: u128,
    /// Token B price normalized to the program's fixed-point scale.
    pub price_b: u128,
    /// Token A deviation from the $1 target in basis points.
    pub peg_delta_a_bps: u128,
    /// Token B deviation from the $1 target in basis points.
    pub peg_delta_b_bps: u128,
    /// Whether swaps and deposits should be halted for this pool state.
    pub should_pause: bool,
}

/// Load, validate, and normalize both Pyth feeds for a stable pair.
pub fn load_pair_status(
    expected_price_feed_a: &Pubkey,
    expected_price_feed_b: &Pubkey,
    price_feed_a: &AccountInfo,
    price_feed_b: &AccountInfo,
    max_price_age_sec: u64,
    depeg_threshold_bps: u16,
) -> Result<OracleStatus> {
    require_keys_eq!(
        *price_feed_a.key,
        *expected_price_feed_a,
        StableSwapError::InvalidOracleAccount
    );
    require_keys_eq!(
        *price_feed_b.key,
        *expected_price_feed_b,
        StableSwapError::InvalidOracleAccount
    );

    let price_a = load_scaled_price(price_feed_a, max_price_age_sec)?;
    let price_b = load_scaled_price(price_feed_b, max_price_age_sec)?;

    let peg_delta_a_bps = calculate_peg_delta_bps(price_a)?;
    let peg_delta_b_bps = calculate_peg_delta_bps(price_b)?;
    let should_pause = peg_delta_a_bps > depeg_threshold_bps as u128
        || peg_delta_b_bps > depeg_threshold_bps as u128;

    Ok(OracleStatus {
        price_a,
        price_b,
        peg_delta_a_bps,
        peg_delta_b_bps,
        should_pause,
    })
}

/// Read a Pyth price account and normalize its price to the shared 1e9 scale.
fn load_scaled_price(price_account_info: &AccountInfo, max_price_age_sec: u64) -> Result<u128> {
    let clock = Clock::get()?;
    let price_account = load_price_account(price_account_info)?;
    let price = select_recent_price(&price_account, clock.unix_timestamp, max_price_age_sec)?;

    scale_price(price.price, price_account.expo)
}

/// Parse a raw account into the embedded Pyth price-account layout.
fn load_price_account(price_account_info: &AccountInfo) -> Result<PythPriceAccount> {
    let data = price_account_info
        .try_borrow_data()
        .map_err(|_| error!(StableSwapError::InvalidOracleAccount))?;
    let bytes = data
        .get(..size_of::<PythPriceAccount>())
        .ok_or_else(|| error!(StableSwapError::InvalidOracleAccount))?;
    let price_account = *try_from_bytes::<PythPriceAccount>(bytes)
        .map_err(|_| error!(StableSwapError::InvalidOracleAccount))?;

    require!(
        price_account.magic == PYTH_MAGIC,
        StableSwapError::InvalidOracleAccount
    );
    require!(
        price_account.ver == PYTH_VERSION_2,
        StableSwapError::InvalidOracleAccount
    );
    require!(
        price_account.atype == PYTH_ACCOUNT_TYPE_PRICE,
        StableSwapError::InvalidOracleAccount
    );

    Ok(price_account)
}

/// Select the newest usable price from the account and enforce freshness.
fn select_recent_price(
    price_account: &PythPriceAccount,
    current_time: i64,
    max_price_age_sec: u64,
) -> Result<PythPrice> {
    let aggregate_price = if price_account.agg.status == PYTH_STATUS_TRADING {
        PythPrice {
            price: price_account.agg.price,
            publish_time: price_account.timestamp,
        }
    } else {
        PythPrice {
            price: price_account.prev_price,
            publish_time: price_account.prev_timestamp,
        }
    };

    let age = aggregate_price.publish_time.abs_diff(current_time);
    require!(age <= max_price_age_sec, StableSwapError::StaleOraclePrice);
    require!(
        aggregate_price.price > 0,
        StableSwapError::InvalidOraclePrice
    );

    Ok(aggregate_price)
}

/// Normalize a Pyth fixed-point price to the program's 1e9 precision.
fn scale_price(price: i64, exponent: i32) -> Result<u128> {
    require!(price > 0, StableSwapError::InvalidOraclePrice);

    let mut normalized = price as u128;

    if exponent > ORACLE_TARGET_EXPONENT {
        let scale = pow10((exponent - ORACLE_TARGET_EXPONENT) as u32)?;
        normalized = normalized
            .checked_mul(scale)
            .ok_or(StableSwapError::MathOverflow)?;
    } else if exponent < ORACLE_TARGET_EXPONENT {
        let scale = pow10((ORACLE_TARGET_EXPONENT - exponent) as u32)?;
        normalized = normalized
            .checked_div(scale)
            .ok_or(StableSwapError::InvalidOraclePrice)?;
    }

    Ok(normalized)
}

/// Convert a normalized stablecoin price into basis points off the $1 peg.
pub fn calculate_peg_delta_bps(price: u128) -> Result<u128> {
    Ok(price
        .abs_diff(TARGET_STABLE_PRICE)
        .checked_mul(BASIS_POINTS_DIVISOR)
        .ok_or(StableSwapError::MathOverflow)?
        .checked_div(ORACLE_PRICE_SCALE)
        .ok_or(StableSwapError::MathOverflow)?)
}

/// Compute `10^exponent` using checked integer arithmetic.
fn pow10(exponent: u32) -> Result<u128> {
    let mut value = 1u128;
    for _ in 0..exponent {
        value = value.checked_mul(10).ok_or(StableSwapError::MathOverflow)?;
    }
    Ok(value)
}

/// Lightweight selected Pyth price used after freshness validation.
#[derive(Debug, Clone, Copy)]
struct PythPrice {
    /// Raw price value reported by Pyth.
    price: i64,
    /// Publish time associated with `price`.
    publish_time: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A perfect peg should report zero deviation and a 2% depeg should report 200 bps.
    #[test]
    fn test_calculate_peg_delta_bps() {
        assert_eq!(calculate_peg_delta_bps(1_000_000_000).unwrap(), 0);
        assert_eq!(calculate_peg_delta_bps(980_000_000).unwrap(), 200);
    }

    /// Price normalization should preserve a 1.0 value across common Pyth exponents.
    #[test]
    fn test_scale_price_handles_positive_and_negative_exponents() {
        assert_eq!(scale_price(1_000_000, -6).unwrap(), 1_000_000_000);
        assert_eq!(scale_price(1_000_000_000, -9).unwrap(), 1_000_000_000);
    }
}
