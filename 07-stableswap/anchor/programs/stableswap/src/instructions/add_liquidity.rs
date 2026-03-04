use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount, Transfer};
use crate::constants::MINIMUM_LIQUIDITY;
use crate::errors::StableSwapError;
use crate::math::calculate_lp_mint_amount;
use crate::state::Pool;

/// Accounts required to add liquidity to a StableSwap pool.
#[derive(Accounts)]
pub struct AddLiquidity<'info> {
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

    /// LP token mint.
    #[account(
        mut,
        constraint = lp_mint.key() == pool.lp_mint @ StableSwapError::InvalidMint,
    )]
    pub lp_mint: Account<'info, Mint>,

    /// Depositor's token A account.
    #[account(mut)]
    pub user_token_a: Account<'info, TokenAccount>,

    /// Depositor's token B account.
    #[account(mut)]
    pub user_token_b: Account<'info, TokenAccount>,

    /// Depositor's LP token account (receives newly minted LP tokens).
    #[account(mut)]
    pub user_lp_token: Account<'info, TokenAccount>,

    /// The depositor.
    pub user: Signer<'info>,

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
    let reserve_a = ctx.accounts.vault_a.amount as u128;
    let reserve_b = ctx.accounts.vault_b.amount as u128;
    let lp_supply = ctx.accounts.lp_mint.supply as u128;
    let amp = pool.amplification as u128;

    let new_reserve_a = reserve_a + amount_a as u128;
    let new_reserve_b = reserve_b + amount_b as u128;

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

    // Transfer token A from user to vault
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

    // Transfer token B from user to vault
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

    // Mint LP tokens to user (pool PDA is mint authority)
    let seeds: &[&[u8]] = &[
        b"pool",
        pool.token_mint_a.as_ref(),
        pool.token_mint_b.as_ref(),
        &[pool.bump],
    ];
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
        "Added liquidity: a={} b={} lp_minted={}",
        amount_a,
        amount_b,
        lp_to_mint
    );
    Ok(())
}
