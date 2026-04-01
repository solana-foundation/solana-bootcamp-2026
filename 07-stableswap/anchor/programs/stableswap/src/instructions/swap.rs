//! Execute a StableSwap trade using oracle-aware dynamic fees.

use crate::errors::StableSwapError;
use crate::math::calculate_swap_output;
use crate::oracle::load_pair_status;
use crate::state::Pool;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

/// Accounts required to execute a swap.
#[derive(Accounts)]
pub struct Swap<'info> {
    /// Token A mint — used together with token_mint_b to derive the pool PDA.
    pub token_mint_a: Box<Account<'info, Mint>>,

    /// Token B mint — used together with token_mint_a to derive the pool PDA.
    pub token_mint_b: Box<Account<'info, Mint>>,

    /// Pool PDA — auto-resolved from [b"pool", token_mint_a, token_mint_b].
    #[account(
        mut,
        seeds = [b"pool", token_mint_a.key().as_ref(), token_mint_b.key().as_ref()],
        bump = pool.bump,
    )]
    pub pool: Account<'info, Pool>,

    /// Pool's token A vault.
    #[account(
        mut,
        constraint = vault_a.key() == pool.vault_a @ StableSwapError::InvalidVault,
    )]
    pub vault_a: Account<'info, TokenAccount>,

    /// Pool's token B vault.
    #[account(
        mut,
        constraint = vault_b.key() == pool.vault_b @ StableSwapError::InvalidVault,
    )]
    pub vault_b: Account<'info, TokenAccount>,

    /// User's input token account (debited).
    #[account(mut)]
    pub user_input: Account<'info, TokenAccount>,

    /// User's output token account (credited).
    #[account(mut)]
    pub user_output: Account<'info, TokenAccount>,

    /// CHECK: validated against the stored pool config and parsed as a Pyth price account.
    pub oracle_price_feed_a: UncheckedAccount<'info>,

    /// CHECK: validated against the stored pool config and parsed as a Pyth price account.
    pub oracle_price_feed_b: UncheckedAccount<'info>,

    /// The swapper.
    pub user: Signer<'info>,

    /// SPL Token program used for token transfer CPI calls.
    pub token_program: Program<'info, Token>,
}

/// Swap tokens using the StableSwap invariant.
///
/// # Arguments
/// * `amount_in`     — Amount of input token to sell.
/// * `min_amount_out` — Minimum output tokens to receive (slippage guard).
/// * `a_to_b`        — `true` to swap A→B, `false` to swap B→A.
pub fn swap_handler(
    ctx: Context<Swap>,
    amount_in: u64,
    min_amount_out: u64,
    a_to_b: bool,
) -> Result<()> {
    require!(amount_in > 0, StableSwapError::ZeroAmount);

    let pool = &ctx.accounts.pool;
    let oracle_status = load_pair_status(
        &pool.oracle_price_feed_a,
        &pool.oracle_price_feed_b,
        &ctx.accounts.oracle_price_feed_a.to_account_info(),
        &ctx.accounts.oracle_price_feed_b.to_account_info(),
        pool.max_price_age_sec,
        pool.depeg_threshold_bps,
    )?;
    require!(!oracle_status.should_pause, StableSwapError::PoolPaused);

    let reserve_a = ctx.accounts.vault_a.amount as u128;
    let reserve_b = ctx.accounts.vault_b.amount as u128;
    let amp = pool.amplification as u128;
    let base_fee_bps = pool.base_fee_bps;
    let max_dynamic_fee_bps = pool.max_dynamic_fee_bps;

    // Both reserves must be non-zero for a valid swap
    require!(reserve_a > 0 && reserve_b > 0, StableSwapError::EmptyPool);

    let expected_input_mint = if a_to_b {
        pool.token_mint_a
    } else {
        pool.token_mint_b
    };
    let expected_output_mint = if a_to_b {
        pool.token_mint_b
    } else {
        pool.token_mint_a
    };
    require_keys_eq!(
        ctx.accounts.user_input.mint,
        expected_input_mint,
        StableSwapError::InvalidMint
    );
    require_keys_eq!(
        ctx.accounts.user_output.mint,
        expected_output_mint,
        StableSwapError::InvalidMint
    );

    // Determine which side is in and which is out
    let (reserve_in, reserve_out, oracle_price_in, oracle_price_out) = if a_to_b {
        (
            reserve_a,
            reserve_b,
            oracle_status.price_a,
            oracle_status.price_b,
        )
    } else {
        (
            reserve_b,
            reserve_a,
            oracle_status.price_b,
            oracle_status.price_a,
        )
    };

    let quote = calculate_swap_output(
        reserve_in,
        reserve_out,
        amount_in as u128,
        amp,
        base_fee_bps,
        max_dynamic_fee_bps,
        oracle_price_in,
        oracle_price_out,
        pool.depeg_threshold_bps,
    )?;

    require!(
        quote.amount_out >= min_amount_out as u128,
        StableSwapError::SlippageExceeded
    );

    let seeds: &[&[u8]] = &[
        b"pool",
        pool.token_mint_a.as_ref(),
        pool.token_mint_b.as_ref(),
        &[pool.bump],
    ];

    if a_to_b {
        // Transfer A in: user → vault_a
        token::transfer(
            CpiContext::new(
                anchor_spl::token::ID,
                Transfer {
                    from: ctx.accounts.user_input.to_account_info(),
                    to: ctx.accounts.vault_a.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount_in,
        )?;

        // Transfer B out: vault_b → user
        token::transfer(
            CpiContext::new_with_signer(
                anchor_spl::token::ID,
                Transfer {
                    from: ctx.accounts.vault_b.to_account_info(),
                    to: ctx.accounts.user_output.to_account_info(),
                    authority: ctx.accounts.pool.to_account_info(),
                },
                &[seeds],
            ),
            quote.amount_out as u64,
        )?;
    } else {
        // Transfer B in: user → vault_b
        token::transfer(
            CpiContext::new(
                anchor_spl::token::ID,
                Transfer {
                    from: ctx.accounts.user_input.to_account_info(),
                    to: ctx.accounts.vault_b.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount_in,
        )?;

        // Transfer A out: vault_a → user
        token::transfer(
            CpiContext::new_with_signer(
                anchor_spl::token::ID,
                Transfer {
                    from: ctx.accounts.vault_a.to_account_info(),
                    to: ctx.accounts.user_output.to_account_info(),
                    authority: ctx.accounts.pool.to_account_info(),
                },
                &[seeds],
            ),
            quote.amount_out as u64,
        )?;
    }

    msg!(
        "Swap {}: {} in → {} out (fee: {}, dynamic_fee={}bps, oracle_a={}bps, oracle_b={}bps)",
        if a_to_b { "A→B" } else { "B→A" },
        amount_in,
        quote.amount_out,
        quote.fee_amount,
        quote.dynamic_fee_bps,
        oracle_status.peg_delta_a_bps,
        oracle_status.peg_delta_b_bps
    );
    Ok(())
}
