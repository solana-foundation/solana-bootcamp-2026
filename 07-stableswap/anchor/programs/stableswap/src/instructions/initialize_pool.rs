use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};
use crate::constants::{MAX_AMP, MAX_FEE_BPS};
use crate::errors::StableSwapError;
use crate::state::Pool;

/// Accounts required to initialize a new StableSwap pool.
#[derive(Accounts)]
pub struct InitializePool<'info> {
    /// Pool creator; pays for account creation.
    #[account(mut)]
    pub admin: Signer<'info>,

    /// Token A mint (e.g. USDC).
    pub token_mint_a: Account<'info, Mint>,

    /// Token B mint (e.g. USDT).
    pub token_mint_b: Account<'info, Mint>,

    /// Pool PDA — derived from [b"pool", token_mint_a, token_mint_b].
    #[account(
        init,
        payer = admin,
        space = Pool::LEN,
        seeds = [b"pool", token_mint_a.key().as_ref(), token_mint_b.key().as_ref()],
        bump,
    )]
    pub pool: Account<'info, Pool>,

    /// LP token mint — authority set to pool PDA.
    #[account(
        init,
        payer = admin,
        mint::decimals = 6,
        mint::authority = pool,
    )]
    pub lp_mint: Account<'info, Mint>,

    /// Pool's token A vault (ATA owned by pool PDA).
    #[account(
        init,
        payer = admin,
        associated_token::mint = token_mint_a,
        associated_token::authority = pool,
    )]
    pub vault_a: Account<'info, TokenAccount>,

    /// Pool's token B vault (ATA owned by pool PDA).
    #[account(
        init,
        payer = admin,
        associated_token::mint = token_mint_b,
        associated_token::authority = pool,
    )]
    pub vault_b: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

/// Initialize a new two-token StableSwap pool.
///
/// # Arguments
/// * `amplification` — The A parameter. Typical range: 100–2000 for stable pairs.
///   Higher A gives lower slippage when both tokens trade near parity.
/// * `fee_bps` — Swap fee in basis points (e.g. 4 = 0.04%).
pub fn initialize_pool_handler(
    ctx: Context<InitializePool>,
    amplification: u64,
    fee_bps: u16,
) -> Result<()> {
    require!(
        amplification > 0 && amplification <= MAX_AMP,
        StableSwapError::InvalidAmplification
    );
    require!(fee_bps <= MAX_FEE_BPS, StableSwapError::InvalidFee);
    require!(
        ctx.accounts.token_mint_a.key() != ctx.accounts.token_mint_b.key(),
        StableSwapError::InvalidMint
    );

    ctx.accounts.pool.set_inner(Pool {
        admin: ctx.accounts.admin.key(),
        token_mint_a: ctx.accounts.token_mint_a.key(),
        vault_a: ctx.accounts.vault_a.key(),
        token_mint_b: ctx.accounts.token_mint_b.key(),
        vault_b: ctx.accounts.vault_b.key(),
        lp_mint: ctx.accounts.lp_mint.key(),
        amplification,
        fee_bps,
        bump: ctx.bumps.pool,
    });

    msg!(
        "StableSwap pool initialized: A={}, fee={}bps",
        amplification,
        fee_bps
    );
    Ok(())
}
