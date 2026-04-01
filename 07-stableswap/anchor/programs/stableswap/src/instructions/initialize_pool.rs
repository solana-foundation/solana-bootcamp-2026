//! Initialize a new StableSwap pool and store its oracle/risk configuration.

use crate::constants::{MAX_AMP, MAX_DEPEG_THRESHOLD_BPS, MAX_FEE_BPS};
use crate::errors::StableSwapError;
use crate::oracle::load_pair_status;
use crate::state::Pool;
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

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
        mint::decimals = token_mint_a.decimals,
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

    /// CHECK: validated against the stored pool config and parsed as a Pyth price account.
    pub oracle_price_feed_a: UncheckedAccount<'info>,

    /// CHECK: validated against the stored pool config and parsed as a Pyth price account.
    pub oracle_price_feed_b: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
    /// SPL Token program used for mint and vault CPI calls.
    pub token_program: Program<'info, Token>,
    /// Associated Token program used to create the pool vault ATAs.
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// Rent sysvar required by Anchor account initialization.
    pub rent: Sysvar<'info, Rent>,
}

/// Initialize a new two-token StableSwap pool.
///
/// # Arguments
/// * `amplification` — The A parameter. Typical range: 100–2000 for stable pairs.
///   Higher A gives lower slippage when both tokens trade near parity.
/// * `base_fee_bps` — Base swap fee in basis points (e.g. 4 = 0.04%).
/// * `max_dynamic_fee_bps` — Fee cap once the pool drifts away from the peg.
/// * `depeg_threshold_bps` — Maximum tolerated peg drift before swaps/deposits halt.
/// * `max_price_age_sec` — Maximum age accepted from Pyth before halting.
pub fn initialize_pool_handler(
    ctx: Context<InitializePool>,
    amplification: u64,
    base_fee_bps: u16,
    max_dynamic_fee_bps: u16,
    depeg_threshold_bps: u16,
    max_price_age_sec: u64,
) -> Result<()> {
    require!(
        amplification > 0 && amplification <= MAX_AMP,
        StableSwapError::InvalidAmplification
    );
    require!(base_fee_bps <= MAX_FEE_BPS, StableSwapError::InvalidFee);
    require!(
        max_dynamic_fee_bps <= MAX_FEE_BPS,
        StableSwapError::InvalidFee
    );
    require!(
        base_fee_bps <= max_dynamic_fee_bps,
        StableSwapError::InvalidFeeConfig
    );
    require!(
        depeg_threshold_bps > 0 && depeg_threshold_bps <= MAX_DEPEG_THRESHOLD_BPS,
        StableSwapError::InvalidDepegThreshold
    );
    require!(max_price_age_sec > 0, StableSwapError::InvalidOracleAge);
    require!(
        ctx.accounts.token_mint_a.key() != ctx.accounts.token_mint_b.key(),
        StableSwapError::InvalidMint
    );
    require!(
        ctx.accounts.token_mint_a.decimals == ctx.accounts.token_mint_b.decimals,
        StableSwapError::InvalidMintDecimals
    );
    require!(
        ctx.accounts.oracle_price_feed_a.key() != ctx.accounts.oracle_price_feed_b.key(),
        StableSwapError::InvalidOracleAccount
    );

    let oracle_status = load_pair_status(
        &ctx.accounts.oracle_price_feed_a.key(),
        &ctx.accounts.oracle_price_feed_b.key(),
        &ctx.accounts.oracle_price_feed_a.to_account_info(),
        &ctx.accounts.oracle_price_feed_b.to_account_info(),
        max_price_age_sec,
        depeg_threshold_bps,
    )?;
    require!(!oracle_status.should_pause, StableSwapError::PoolPaused);

    ctx.accounts.pool.set_inner(Pool {
        admin: ctx.accounts.admin.key(),
        token_mint_a: ctx.accounts.token_mint_a.key(),
        vault_a: ctx.accounts.vault_a.key(),
        token_mint_b: ctx.accounts.token_mint_b.key(),
        vault_b: ctx.accounts.vault_b.key(),
        lp_mint: ctx.accounts.lp_mint.key(),
        amplification,
        base_fee_bps,
        max_dynamic_fee_bps,
        depeg_threshold_bps,
        max_price_age_sec,
        oracle_price_feed_a: ctx.accounts.oracle_price_feed_a.key(),
        oracle_price_feed_b: ctx.accounts.oracle_price_feed_b.key(),
        bump: ctx.bumps.pool,
    });

    msg!(
        "StableSwap pool initialized: A={}, base_fee={}bps, max_fee={}bps, depeg_threshold={}bps",
        amplification,
        base_fee_bps,
        max_dynamic_fee_bps,
        depeg_threshold_bps
    );
    Ok(())
}
