//! Deposit or withdraw liquidity from the pool.
//
// ============================================================================
// MODIFY LIQUIDITY INSTRUCTIONS
// ============================================================================
//
// This file handles adding and removing liquidity from the pool.
//
// LIQUIDITY PROVIDER (LP) BASICS
//
// - LPs deposit tokens into the pool to earn fees from swaps
// - In return, they receive LP tokens representing their share of the pool
// - When they withdraw, they burn LP tokens to get back their share of reserves
//
// KEY CONCEPTS
//
// - D (invariant): represents the normalized total value of the pool
// - LP tokens: your ownership share of D
// - First deposit: special handling to prevent LP inflation attacks
// ============================================================================

use crate::constants::DEFAULT_MAX_PRICE_AGE_SEC;
use crate::constants::MINIMUM_LIQUIDITY;
use crate::errors::StableSwapError;
use crate::math::{calculate_lp_mint_amount, calculate_withdraw_amounts};
use crate::oracle::load_pair_status;
use crate::state::Pool;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, MintTo, Token, TokenAccount, Transfer};

/// Accounts required to add liquidity to a StableSwap pool.
#[derive(Accounts)]
pub struct AddLiquidity<'info> {
    /// Token A mint — validated against the pool's stored token list.
    pub token_mint_a: Box<Account<'info, Mint>>,

    /// Token B mint — validated against the pool's stored token list.
    pub token_mint_b: Box<Account<'info, Mint>>,

    /// Pool PDA — auto-resolved from [b"pool", lp_mint].
    #[account(
        mut,
        seeds = [b"pool", lp_mint.key().as_ref()],
        bump = pool.bump,
    )]
    pub pool: Box<Account<'info, Pool>>,

    /// Pool's token A vault.
    #[account(
        mut,
        constraint = vault_a.mint == token_mint_a.key() @ StableSwapError::InvalidVault,
        constraint = vault_a.owner == pool.key() @ StableSwapError::InvalidVault,
    )]
    pub vault_a: Box<Account<'info, TokenAccount>>,

    /// Pool's token B vault.
    #[account(
        mut,
        constraint = vault_b.mint == token_mint_b.key() @ StableSwapError::InvalidVault,
        constraint = vault_b.owner == pool.key() @ StableSwapError::InvalidVault,
    )]
    pub vault_b: Box<Account<'info, TokenAccount>>,

    /// LP token mint.
    #[account(
        mut,
        constraint = lp_mint.key() == pool.lp_mint @ StableSwapError::InvalidMint,
    )]
    pub lp_mint: Box<Account<'info, Mint>>,

    /// Depositor's token A account.
    #[account(
        mut,
        constraint = user_token_a.mint == token_mint_a.key() @ StableSwapError::InvalidMint,
    )]
    pub user_token_a: Box<Account<'info, TokenAccount>>,

    /// Depositor's token B account.
    #[account(
        mut,
        constraint = user_token_b.mint == token_mint_b.key() @ StableSwapError::InvalidMint,
    )]
    pub user_token_b: Box<Account<'info, TokenAccount>>,

    /// Depositor's LP token account (receives newly minted LP tokens).
    #[account(
        mut,
        constraint = user_lp_token.mint == pool.lp_mint @ StableSwapError::InvalidMint,
    )]
    pub user_lp_token: Box<Account<'info, TokenAccount>>,

    /// CHECK: validated against the stored pool config and parsed as a Pyth price account.
    pub oracle_price_feed_a: UncheckedAccount<'info>,

    /// CHECK: validated against the stored pool config and parsed as a Pyth price account.
    pub oracle_price_feed_b: UncheckedAccount<'info>,

    /// The depositor.
    pub user: Signer<'info>,

    /// SPL Token program used for transfers and LP minting.
    pub token_program: Program<'info, Token>,
}

/// Accounts required to remove liquidity from a StableSwap pool.
#[derive(Accounts)]
pub struct RemoveLiquidity<'info> {
    /// Token A mint — validated against the pool's stored token list.
    pub token_mint_a: Box<Account<'info, Mint>>,

    /// Token B mint — validated against the pool's stored token list.
    pub token_mint_b: Box<Account<'info, Mint>>,

    /// Pool PDA — auto-resolved from [b"pool", lp_mint].
    #[account(
        mut,
        seeds = [b"pool", lp_mint.key().as_ref()],
        bump = pool.bump,
    )]
    pub pool: Box<Account<'info, Pool>>,

    /// Pool's token A vault.
    #[account(
        mut,
        constraint = vault_a.mint == token_mint_a.key() @ StableSwapError::InvalidVault,
        constraint = vault_a.owner == pool.key() @ StableSwapError::InvalidVault,
    )]
    pub vault_a: Box<Account<'info, TokenAccount>>,

    /// Pool's token B vault.
    #[account(
        mut,
        constraint = vault_b.mint == token_mint_b.key() @ StableSwapError::InvalidVault,
        constraint = vault_b.owner == pool.key() @ StableSwapError::InvalidVault,
    )]
    pub vault_b: Box<Account<'info, TokenAccount>>,

    /// LP token mint (supply decreases on burn).
    #[account(
        mut,
        constraint = lp_mint.key() == pool.lp_mint @ StableSwapError::InvalidMint,
    )]
    pub lp_mint: Box<Account<'info, Mint>>,

    /// Withdrawer's token A account (receives token A).
    #[account(
        mut,
        constraint = user_token_a.mint == token_mint_a.key() @ StableSwapError::InvalidMint,
    )]
    pub user_token_a: Box<Account<'info, TokenAccount>>,

    /// Withdrawer's token B account (receives token B).
    #[account(
        mut,
        constraint = user_token_b.mint == token_mint_b.key() @ StableSwapError::InvalidMint,
    )]
    pub user_token_b: Box<Account<'info, TokenAccount>>,

    /// Withdrawer's LP token account (LP tokens burned from here).
    #[account(
        mut,
        constraint = user_lp_token.mint == pool.lp_mint @ StableSwapError::InvalidMint,
    )]
    pub user_lp_token: Box<Account<'info, TokenAccount>>,

    /// The withdrawer.
    pub user: Signer<'info>,

    /// SPL Token program used for burn and transfer CPI calls.
    pub token_program: Program<'info, Token>,
}

/// Deposit token A and token B into the pool to receive LP tokens.
///
/// # Arguments
/// * `amount_a`     — Token A amount to deposit.
/// * `amount_b`     — Token B amount to deposit.
/// * `min_lp_out`   — Minimum LP tokens to receive (slippage guard).
pub fn add_liquidity_handler(
    ctx: Context<AddLiquidity>,
    amount_a: u64,
    amount_b: u64,
    min_lp_out: u64,
) -> Result<()> {
    require!(amount_a > 0 || amount_b > 0, StableSwapError::ZeroAmount);

    let pool = &ctx.accounts.pool;
    require!(!pool.is_paused, StableSwapError::PoolPaused);
    let oracle_status = load_pair_status(
        &pool.oracle_config.oracle_a,
        &pool.oracle_config.oracle_b,
        &ctx.accounts.oracle_price_feed_a.to_account_info(),
        &ctx.accounts.oracle_price_feed_b.to_account_info(),
        DEFAULT_MAX_PRICE_AGE_SEC,
        pool.oracle_config.max_depeg_bps,
    )?;
    require!(
        !pool.oracle_config.enabled || !oracle_status.should_pause,
        StableSwapError::PoolPaused
    );

    let reserve_a = ctx.accounts.vault_a.amount as u128;
    let reserve_b = ctx.accounts.vault_b.amount as u128;
    let lp_supply = ctx.accounts.lp_mint.supply as u128;
    let amp = pool.amplification as u128;

    let new_reserve_a = reserve_a
        .checked_add(amount_a as u128)
        .ok_or(StableSwapError::MathOverflow)?;
    let new_reserve_b = reserve_b
        .checked_add(amount_b as u128)
        .ok_or(StableSwapError::MathOverflow)?;

    let lp_to_mint = calculate_lp_mint_amount(
        reserve_a,
        reserve_b,
        new_reserve_a,
        new_reserve_b,
        lp_supply,
        amp,
        MINIMUM_LIQUIDITY,
    )?;

    require!(lp_to_mint >= min_lp_out, StableSwapError::SlippageExceeded);
    require!(lp_to_mint > 0, StableSwapError::ZeroAmount);

    if amount_a > 0 {
        token::transfer(
            CpiContext::new(
                anchor_spl::token::ID,
                Transfer {
                    from: ctx.accounts.user_token_a.to_account_info(),
                    to: ctx.accounts.vault_a.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount_a,
        )?;
    }

    if amount_b > 0 {
        token::transfer(
            CpiContext::new(
                anchor_spl::token::ID,
                Transfer {
                    from: ctx.accounts.user_token_b.to_account_info(),
                    to: ctx.accounts.vault_b.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount_b,
        )?;
    }

    let seeds: &[&[u8]] = &[b"pool", pool.lp_mint.as_ref(), &[pool.bump]];
    token::mint_to(
        CpiContext::new_with_signer(
            anchor_spl::token::ID,
            MintTo {
                mint: ctx.accounts.lp_mint.to_account_info(),
                to: ctx.accounts.user_lp_token.to_account_info(),
                authority: ctx.accounts.pool.to_account_info(),
            },
            &[seeds],
        ),
        lp_to_mint,
    )?;

    msg!(
        "Added liquidity: a={} b={} lp_minted={} oracle_a={}bps oracle_b={}bps",
        amount_a,
        amount_b,
        lp_to_mint,
        oracle_status.peg_delta_a_bps,
        oracle_status.peg_delta_b_bps
    );
    Ok(())
}

/// Burn LP tokens to withdraw a proportional share of both token A and token B.
///
/// Withdrawals intentionally remain oracle-independent so LPs can always exit,
/// even when swaps and deposits are halted by depeg protection.
///
/// # Arguments
/// * `lp_amount` — LP tokens to burn.
/// * `min_a`     — Minimum token A to receive (slippage guard).
/// * `min_b`     — Minimum token B to receive (slippage guard).
pub fn remove_liquidity_handler(
    ctx: Context<RemoveLiquidity>,
    lp_amount: u64,
    min_a: u64,
    min_b: u64,
) -> Result<()> {
    require!(lp_amount > 0, StableSwapError::ZeroAmount);

    let pool = &ctx.accounts.pool;
    let reserve_a = ctx.accounts.vault_a.amount as u128;
    let reserve_b = ctx.accounts.vault_b.amount as u128;
    let actual_supply = ctx.accounts.lp_mint.supply;
    let adjusted_supply = (actual_supply as u128)
        .checked_add(MINIMUM_LIQUIDITY as u128)
        .ok_or(StableSwapError::MathOverflow)?;

    require!(adjusted_supply > 0, StableSwapError::EmptyPool);
    require!(
        lp_amount as u128 <= actual_supply as u128,
        StableSwapError::InsufficientLiquidity
    );

    // Withdrawals are always proportional across both pool assets.
    // This pool intentionally does not support single-sided LP exits.
    let withdraw_amounts =
        calculate_withdraw_amounts(&[reserve_a, reserve_b], lp_amount as u128, adjusted_supply)?;
    let amount_a = withdraw_amounts[0];
    let amount_b = withdraw_amounts[1];

    // Slippage protection: both token outputs must clear the user's minimums.
    require!(amount_a >= min_a, StableSwapError::SlippageExceeded);
    require!(amount_b >= min_b, StableSwapError::SlippageExceeded);

    token::burn(
        CpiContext::new(
            anchor_spl::token::ID,
            Burn {
                mint: ctx.accounts.lp_mint.to_account_info(),
                from: ctx.accounts.user_lp_token.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        lp_amount,
    )?;

    let seeds: &[&[u8]] = &[b"pool", pool.lp_mint.as_ref(), &[pool.bump]];

    if amount_a > 0 {
        token::transfer(
            CpiContext::new_with_signer(
                anchor_spl::token::ID,
                Transfer {
                    from: ctx.accounts.vault_a.to_account_info(),
                    to: ctx.accounts.user_token_a.to_account_info(),
                    authority: ctx.accounts.pool.to_account_info(),
                },
                &[seeds],
            ),
            amount_a,
        )?;
    }

    if amount_b > 0 {
        token::transfer(
            CpiContext::new_with_signer(
                anchor_spl::token::ID,
                Transfer {
                    from: ctx.accounts.vault_b.to_account_info(),
                    to: ctx.accounts.user_token_b.to_account_info(),
                    authority: ctx.accounts.pool.to_account_info(),
                },
                &[seeds],
            ),
            amount_b,
        )?;
    }

    msg!(
        "Removed liquidity: lp_burned={} a_out={} b_out={}",
        lp_amount,
        amount_a,
        amount_b
    );
    Ok(())
}
