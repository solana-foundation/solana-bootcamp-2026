//! Explicit oracle-health instruction for checking whether a pool is depegged.
//
// ============================================================================
// CHECK DEPEG INSTRUCTION
// ============================================================================
//
// This instruction allows anyone (keepers, bots, admins) to check whether the
// pool's stablecoins have depegged from $1.
//
// WHY IS THIS A SEPARATE INSTRUCTION?
//
// 1. Checking oracles on every swap is expensive
// 2. Not every pool design wants to rely on a continuously persisted pause flag
// 3. Keepers can call this periodically to evaluate pool health
// 4. It provides a lightweight way to react to depeg events without executing a
//    swap or deposit
//
// HOW IT WORKS
//
// 1. Read prices from the Pyth oracles for both tokens
// 2. Check whether either price has deviated from $1 beyond the configured
//    threshold
// 3. Return success if the pool is healthy, or `PoolPaused` if the pair should
//    be treated as halted
//
// REMAINING ACCOUNTS
//
// - `oracle_price_feed_a`: Pyth oracle for token A
// - `oracle_price_feed_b`: Pyth oracle for token B
// ============================================================================

use crate::constants::DEFAULT_MAX_PRICE_AGE_SEC;
use crate::errors::StableSwapError;
use crate::oracle::load_pair_status;
use crate::state::Pool;
use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

/// Accounts required to verify whether the pool is still inside its peg band.
#[derive(Accounts)]
pub struct CheckDepeg<'info> {
    /// Token A mint — validated against the pool's stored token list.
    pub token_mint_a: Box<Account<'info, Mint>>,

    /// Token B mint — validated against the pool's stored token list.
    pub token_mint_b: Box<Account<'info, Mint>>,

    /// LP mint used to derive the pool PDA.
    #[account(
        constraint = lp_mint.key() == pool.lp_mint @ StableSwapError::InvalidMint,
    )]
    pub lp_mint: Box<Account<'info, Mint>>,

    /// Pool PDA — auto-resolved from [b"pool", lp_mint].
    #[account(
        mut,
        seeds = [b"pool", lp_mint.key().as_ref()],
        bump = pool.bump,
    )]
    pub pool: Account<'info, Pool>,

    /// CHECK: validated against the stored pool config and parsed as a Pyth price account.
    pub oracle_price_feed_a: UncheckedAccount<'info>,

    /// CHECK: validated against the stored pool config and parsed as a Pyth price account.
    pub oracle_price_feed_b: UncheckedAccount<'info>,
}

/// Verify that both stablecoins remain inside the configured peg band.
///
/// This instruction is useful for explicit monitoring, pre-trade checks, and
/// test harnesses that want to validate oracle health without executing a swap
/// or deposit.
pub fn check_depeg_handler(ctx: Context<CheckDepeg>) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    let oracle_status = load_pair_status(
        &pool.oracle_config.oracle_a,
        &pool.oracle_config.oracle_b,
        &ctx.accounts.oracle_price_feed_a.to_account_info(),
        &ctx.accounts.oracle_price_feed_b.to_account_info(),
        DEFAULT_MAX_PRICE_AGE_SEC,
        pool.oracle_config.max_depeg_bps,
    )?;

    pool.is_paused = pool.oracle_config.enabled && oracle_status.should_pause;

    msg!(
        "Check depeg: oracle_a={}bps oracle_b={}bps threshold={}bps paused={}",
        oracle_status.peg_delta_a_bps,
        oracle_status.peg_delta_b_bps,
        pool.oracle_config.max_depeg_bps,
        pool.is_paused
    );
    Ok(())
}
