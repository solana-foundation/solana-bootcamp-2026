//! Withdraw proportional liquidity from the pool.

use crate::constants::MINIMUM_LIQUIDITY;
use crate::errors::StableSwapError;
use crate::state::Pool;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, Token, TokenAccount, Transfer};

/// Accounts required to remove liquidity from a StableSwap pool.
#[derive(Accounts)]
pub struct RemoveLiquidity<'info> {
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

    /// LP token mint (supply decreases on burn).
    #[account(
        mut,
        constraint = lp_mint.key() == pool.lp_mint @ StableSwapError::InvalidMint,
    )]
    pub lp_mint: Account<'info, Mint>,

    /// Withdrawer's token A account (receives token A).
    #[account(
        mut,
        constraint = user_token_a.mint == pool.token_mint_a @ StableSwapError::InvalidMint,
    )]
    pub user_token_a: Account<'info, TokenAccount>,

    /// Withdrawer's token B account (receives token B).
    #[account(
        mut,
        constraint = user_token_b.mint == pool.token_mint_b @ StableSwapError::InvalidMint,
    )]
    pub user_token_b: Account<'info, TokenAccount>,

    /// Withdrawer's LP token account (LP tokens burned from here).
    #[account(
        mut,
        constraint = user_lp_token.mint == pool.lp_mint @ StableSwapError::InvalidMint,
    )]
    pub user_lp_token: Account<'info, TokenAccount>,

    /// The withdrawer.
    pub user: Signer<'info>,

    /// SPL Token program used for burn and transfer CPI calls.
    pub token_program: Program<'info, Token>,
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
    // Actual on-chain supply: does NOT include the MINIMUM_LIQUIDITY virtual shares.
    // We add them back to compute proportional withdrawal correctly.
    let actual_supply = ctx.accounts.lp_mint.supply;
    let adjusted_supply = (actual_supply as u128)
        .checked_add(MINIMUM_LIQUIDITY as u128)
        .ok_or(StableSwapError::MathOverflow)?;

    require!(adjusted_supply > 0, StableSwapError::EmptyPool);
    require!(
        lp_amount as u128 <= actual_supply as u128,
        StableSwapError::InsufficientLiquidity
    );

    // Proportional withdrawal: amount_i = reserve_i * lp_amount / adjusted_supply
    let amount_a = (reserve_a
        .checked_mul(lp_amount as u128)
        .ok_or(StableSwapError::MathOverflow)?)
    .checked_div(adjusted_supply)
    .ok_or(StableSwapError::MathOverflow)? as u64;

    let amount_b = (reserve_b
        .checked_mul(lp_amount as u128)
        .ok_or(StableSwapError::MathOverflow)?)
    .checked_div(adjusted_supply)
    .ok_or(StableSwapError::MathOverflow)? as u64;

    require!(amount_a >= min_a, StableSwapError::SlippageExceeded);
    require!(amount_b >= min_b, StableSwapError::SlippageExceeded);

    // Burn LP tokens from user
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

    let seeds: &[&[u8]] = &[
        b"pool",
        pool.token_mint_a.as_ref(),
        pool.token_mint_b.as_ref(),
        &[pool.bump],
    ];

    // Transfer token A to user
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

    // Transfer token B to user
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
