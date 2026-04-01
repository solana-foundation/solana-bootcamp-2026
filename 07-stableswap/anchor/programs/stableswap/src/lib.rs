//! StableSwap program entrypoint and module wiring.

use anchor_lang::prelude::*;

/// Shared constants used across invariant, fee, and oracle logic.
pub mod constants;
/// Program-specific error codes returned by Anchor instructions.
pub mod errors;
/// Instruction account contexts and handlers.
pub mod instructions;
/// StableSwap invariant and fee math.
pub mod math;
/// Pyth oracle parsing and peg-protection helpers.
pub mod oracle;
/// Persistent on-chain account state definitions.
pub mod state;

use instructions::*;

declare_id!("CorabfeniSyoc4aLcJe7t9b3RaFX5tzVWXdewU1xuA6B");

/// StableSwap AMM — a two-token liquidity pool optimized for stablecoin pairs.
///
/// ## Background
///
/// Constant-product AMMs (Uniswap-style x·y=k) have significant price impact
/// even for modest trades. For assets that should trade at nearly equal value
/// (USDC/USDT, USDh/USDT, etc.) the Curve-style hybrid invariant:
///
///   4·A·(x + y) + D  =  4·A·D + D³/(4·x·y)
///
/// gives dramatically lower slippage while still self-balancing when the peg
/// breaks. The amplification parameter A controls the trade-off:
///
///   - High A (100-2000): very stable, nearly flat curve near peg
///   - Low A (1-10): close to constant-product, handles de-peg better
#[program]
pub mod stableswap {
    use super::*;

    /// Create a new two-token StableSwap pool.
    ///
    /// Initialises the pool state, LP mint, and two token vaults.
    ///
    /// # Arguments
    /// * `amplification` — A parameter (1–1,000,000). Typical: 100–2000.
    /// * `base_fee_bps`         — Minimum swap fee in basis points.
    /// * `max_dynamic_fee_bps`  — Maximum swap fee once the pool drifts away from peg.
    /// * `depeg_threshold_bps`  — Peg band that triggers the oracle pause.
    /// * `max_price_age_sec`    — Maximum acceptable Pyth price age.
    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        amplification: u64,
        base_fee_bps: u16,
        max_dynamic_fee_bps: u16,
        depeg_threshold_bps: u16,
        max_price_age_sec: u64,
    ) -> Result<()> {
        instructions::initialize_pool::initialize_pool_handler(
            ctx,
            amplification,
            base_fee_bps,
            max_dynamic_fee_bps,
            depeg_threshold_bps,
            max_price_age_sec,
        )
    }

    /// Deposit token A and/or token B to receive LP tokens.
    ///
    /// LP tokens represent a proportional share of the pool.
    /// Depositing both tokens in the current pool ratio is most efficient,
    /// but imbalanced deposits are also supported.
    ///
    /// # Arguments
    /// * `amount_a`   — Token A to deposit.
    /// * `amount_b`   — Token B to deposit.
    /// * `min_lp_out` — Minimum LP tokens to receive (slippage guard).
    pub fn add_liquidity(
        ctx: Context<AddLiquidity>,
        amount_a: u64,
        amount_b: u64,
        min_lp_out: u64,
    ) -> Result<()> {
        instructions::add_liquidity::add_liquidity_handler(ctx, amount_a, amount_b, min_lp_out)
    }

    /// Burn LP tokens to withdraw a proportional share of both tokens.
    ///
    /// # Arguments
    /// * `lp_amount` — LP tokens to burn.
    /// * `min_a`     — Minimum token A to receive (slippage guard).
    /// * `min_b`     — Minimum token B to receive (slippage guard).
    pub fn remove_liquidity(
        ctx: Context<RemoveLiquidity>,
        lp_amount: u64,
        min_a: u64,
        min_b: u64,
    ) -> Result<()> {
        instructions::remove_liquidity::remove_liquidity_handler(ctx, lp_amount, min_a, min_b)
    }

    /// Swap token A for token B or token B for token A.
    ///
    /// Uses the StableSwap invariant for extremely low slippage when both
    /// tokens trade near parity, while Pyth oracle checks and adaptive fees
    /// defend LPs when the pool drifts away from fair value.
    ///
    /// # Arguments
    /// * `amount_in`      — Input amount to sell.
    /// * `min_amount_out` — Minimum output amount (slippage guard).
    /// * `a_to_b`         — `true` for A→B, `false` for B→A.
    pub fn swap(
        ctx: Context<Swap>,
        amount_in: u64,
        min_amount_out: u64,
        a_to_b: bool,
    ) -> Result<()> {
        instructions::swap::swap_handler(ctx, amount_in, min_amount_out, a_to_b)
    }
}
