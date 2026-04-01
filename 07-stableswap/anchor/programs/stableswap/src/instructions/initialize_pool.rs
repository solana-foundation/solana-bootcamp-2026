//! Initialize a new StableSwap pool and store its oracle/risk configuration.
//
// ============================================================================
// INITIALIZE POOL INSTRUCTION
// ============================================================================
//
// This instruction creates a new StableSwap pool for 2 tokens.
//
// WHAT GETS CREATED?
//
// 1. Pool account (PDA) - stores pool configuration such as amplification,
//    fee settings, token mints, vault addresses, and oracle feeds
// 2. LP mint - an SPL token mint representing ownership shares in the pool
// 3. Vault A - pool-owned token account for token A reserves
// 4. Vault B - pool-owned token account for token B reserves
//
// The pool account is a PDA (Program Derived Address) seeded by the LP mint
// address. Because this seed format is deterministic, anyone can derive the
// pool address off chain.
//
// ACCOUNT STRUCTURE
//
// - admin: pays for account creation and becomes pool admin
// - token_mint_a: mint for token A
// - token_mint_b: mint for token B
// - pool: the pool PDA we are initializing
// - lp_mint: new mint for LP tokens, with the pool PDA as mint authority
// - vault_a: pool-owned ATA for token A
// - vault_b: pool-owned ATA for token B
// - oracle_price_feed_a: Pyth feed for token A
// - oracle_price_feed_b: Pyth feed for token B
// ============================================================================

use crate::constants::{MAX_AMP, MAX_DEPEG_THRESHOLD_BPS, MAX_FEE_BPS, NUM_TOKENS};
use crate::errors::StableSwapError;
use crate::oracle::load_pair_status;
use crate::state::{OracleConfig, Pool};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{get_associated_token_address, AssociatedToken},
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

    /// Pool PDA — derived from [b"pool", lp_mint].
    #[account(
        init,
        payer = admin,
        space = Pool::LEN,
        seeds = [b"pool", lp_mint.key().as_ref()],
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

    /// CHECK: validated against the canonical system program ID in the handler.
    pub system_program: UncheckedAccount<'info>,
    /// SPL Token program used for mint and vault CPI calls.
    /// CHECK: validated against the canonical SPL token program ID in the handler.
    pub token_program: UncheckedAccount<'info>,
    /// Associated Token program used to create the pool vault ATAs.
    /// CHECK: validated against the canonical associated token program ID in the handler.
    pub associated_token_program: UncheckedAccount<'info>,
    /// Rent sysvar required by Anchor account initialization.
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> InitializePool<'info> {
    /// Validate the core pool parameters.
    ///
    /// We check:
    /// 1. Amplification is in the valid range `(0, MAX_AMP]`
    /// 2. The base fee does not exceed 100%
    pub fn validate(&self, amplification: u64, fee_bps: u16) -> Result<()> {
        // A must be positive. A=0 would break the StableSwap invariant because
        // later math divides by `ann = 4 * A`.
        require!(
            amplification > 0 && amplification <= MAX_AMP,
            StableSwapError::InvalidAmplification
        );

        // Fee sanity check. Values above 100% would make swaps nonsensical.
        require!(fee_bps <= MAX_FEE_BPS, StableSwapError::InvalidFee);

        Ok(())
    }
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
    ctx.accounts.validate(amplification, base_fee_bps)?;
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
    require_keys_eq!(
        ctx.accounts.system_program.key(),
        System::id(),
        StableSwapError::InvalidSystemProgram
    );
    require_keys_eq!(
        ctx.accounts.token_program.key(),
        Token::id(),
        StableSwapError::InvalidTokenProgram
    );
    require_keys_eq!(
        ctx.accounts.associated_token_program.key(),
        AssociatedToken::id(),
        StableSwapError::InvalidAssociatedTokenProgram
    );

    // VAULT VALIDATION (Important security check!)
    //
    // The vaults must be the canonical ATAs owned by the pool PDA.
    // If arbitrary token accounts were accepted here, a malicious caller could
    // substitute attacker-controlled accounts and divert pool funds.
    let expected_vault_a =
        get_associated_token_address(&ctx.accounts.pool.key(), &ctx.accounts.token_mint_a.key());
    require!(
        ctx.accounts.vault_a.key() == expected_vault_a,
        StableSwapError::InvalidVault
    );

    let expected_vault_b =
        get_associated_token_address(&ctx.accounts.pool.key(), &ctx.accounts.token_mint_b.key());
    require!(
        ctx.accounts.vault_b.key() == expected_vault_b,
        StableSwapError::InvalidVault
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
        lp_mint: ctx.accounts.lp_mint.key(),
        amplification,
        fee_bps: base_fee_bps,
        token_mints: [
            ctx.accounts.token_mint_a.key(),
            ctx.accounts.token_mint_b.key(),
        ],
        bump: ctx.bumps.pool,
        oracle_config: OracleConfig {
            oracle_a: ctx.accounts.oracle_price_feed_a.key(),
            oracle_b: ctx.accounts.oracle_price_feed_b.key(),
            max_depeg_bps: depeg_threshold_bps,
            emergency_fee_bps: max_dynamic_fee_bps,
            enabled: true,
        },
        is_paused: false,
    });

    msg!(
        "StableSwap pool initialized: tokens={}, A={}, fee={}bps, emergency_fee={}bps, depeg_threshold={}bps",
        NUM_TOKENS,
        amplification,
        base_fee_bps,
        max_dynamic_fee_bps,
        depeg_threshold_bps
    );
    Ok(())
}
