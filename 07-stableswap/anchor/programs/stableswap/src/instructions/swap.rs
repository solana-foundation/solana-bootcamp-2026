//! Execute a StableSwap trade using oracle-aware dynamic fees.
//
// ============================================================================
// SWAP INSTRUCTION
// ============================================================================
//
// The core AMM function. This lets users trade one token for another.
//
// HOW A SWAP WORKS
//
// 1. User specifies: "I want to swap X of token A for token B"
// 2. We calculate how much token B they should receive using StableSwap math
// 3. We transfer token A from user to pool vault
// 4. We transfer token B from pool vault to user
//
// The exchange rate comes from the StableSwap invariant formula.
// Unlike Uniswap's `x*y=k`, StableSwap gives much better rates for stablecoins.
//
// SLIPPAGE PROTECTION
//
// The user specifies `min_amount_out`, the minimum they'll accept.
// If market conditions change or there's a bug, the swap reverts.
// This protects against front-running and sandwich attacks.
// ============================================================================

use crate::constants::DEFAULT_MAX_PRICE_AGE_SEC;
use crate::constants::NUM_TOKENS;
use crate::errors::StableSwapError;
use crate::math::calculate_swap_output;
use crate::oracle::load_pair_status;
use crate::state::Pool;
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::get_associated_token_address,
    token::{self, Token, TokenAccount, Transfer},
};

/// Accounts required to execute a swap.
#[derive(Accounts)]
pub struct Swap<'info> {
    /// The pool we're swapping in.
    #[account(mut)]
    pub pool: Account<'info, Pool>,

    /// CHECK: validated against the stored pool config and parsed as a Pyth price account.
    pub oracle_price_feed_a: UncheckedAccount<'info>,

    /// CHECK: validated against the stored pool config and parsed as a Pyth price account.
    pub oracle_price_feed_b: UncheckedAccount<'info>,

    /// The user performing the swap. They sign to authorize token transfers.
    pub user: Signer<'info>,

    /// SPL Token program used for transfer instructions.
    pub token_program: Program<'info, Token>,
}

impl<'info> Swap<'info> {
    /// Read and deserialize a token account from the remaining account list.
    fn read_token_account(&self, account: &AccountInfo<'info>) -> Result<TokenAccount> {
        let data = &mut &account.try_borrow_data()?[..];
        TokenAccount::try_deserialize(data)
            .map_err(|_| error!(StableSwapError::InvalidRemainingAccounts))
    }

    /// Read reserve balances from the provided vault accounts.
    pub fn read_reserves(&self, vaults: &[&AccountInfo<'info>]) -> Result<Vec<u128>> {
        let mut reserves = Vec::with_capacity(vaults.len());

        for vault in vaults {
            let account = self.read_token_account(vault)?;
            reserves.push(account.amount as u128);
        }

        Ok(reserves)
    }

    /// Validate the fixed remaining-account layout used by the swap instruction.
    ///
    /// Expected order for a 2-token pool:
    /// - `[0]` Token A vault (pool ATA)
    /// - `[1]` Token B vault (pool ATA)
    /// - `[2]` User's input token account
    /// - `[3]` User's output token account
    pub fn validate_remaining_accounts(
        &self,
        remaining: &[AccountInfo<'info>],
        input_index: u8,
        output_index: u8,
    ) -> Result<()> {
        require!(
            input_index < NUM_TOKENS as u8,
            StableSwapError::InvalidTokenIndex
        );
        require!(
            output_index < NUM_TOKENS as u8,
            StableSwapError::InvalidTokenIndex
        );
        require!(input_index != output_index, StableSwapError::SameTokenSwap);
        require!(
            remaining.len() == NUM_TOKENS + 2,
            StableSwapError::InvalidRemainingAccounts
        );

        let vault_a = self.read_token_account(&remaining[0])?;
        let vault_b = self.read_token_account(&remaining[1])?;
        let user_input = self.read_token_account(&remaining[2])?;
        let user_output = self.read_token_account(&remaining[3])?;

        let expected_vault_a =
            get_associated_token_address(&self.pool.key(), &self.pool.token_mints[0]);
        let expected_vault_b =
            get_associated_token_address(&self.pool.key(), &self.pool.token_mints[1]);

        require_keys_eq!(
            remaining[0].key(),
            expected_vault_a,
            StableSwapError::InvalidVault
        );
        require_keys_eq!(
            remaining[1].key(),
            expected_vault_b,
            StableSwapError::InvalidVault
        );
        require_keys_eq!(
            vault_a.owner,
            self.pool.key(),
            StableSwapError::InvalidVault
        );
        require_keys_eq!(
            vault_b.owner,
            self.pool.key(),
            StableSwapError::InvalidVault
        );
        require_keys_eq!(
            vault_a.mint,
            self.pool.token_mints[0],
            StableSwapError::InvalidVault
        );
        require_keys_eq!(
            vault_b.mint,
            self.pool.token_mints[1],
            StableSwapError::InvalidVault
        );

        require_keys_eq!(
            user_input.owner,
            self.user.key(),
            StableSwapError::InvalidRemainingAccounts
        );
        require_keys_eq!(
            user_output.owner,
            self.user.key(),
            StableSwapError::InvalidRemainingAccounts
        );
        require_keys_eq!(
            user_input.mint,
            self.pool.token_mints[input_index as usize],
            StableSwapError::InvalidMint
        );
        require_keys_eq!(
            user_output.mint,
            self.pool.token_mints[output_index as usize],
            StableSwapError::InvalidMint
        );

        Ok(())
    }

    /// Transfer tokens FROM the user TO the pool vault.
    ///
    /// CPI (Cross-Program Invocation) EXPLANATION:
    ///
    /// We're calling the SPL Token program to do the actual transfer.
    /// The user must have signed the transaction, so they are the authority.
    pub fn transfer_in(
        &self,
        from: &AccountInfo<'info>,
        vault: &AccountInfo<'info>,
        amount: u64,
    ) -> Result<()> {
        token::transfer(
            CpiContext::new(
                self.token_program.key(),
                Transfer {
                    from: from.clone(),
                    to: vault.clone(),
                    authority: self.user.to_account_info(),
                },
            ),
            amount,
        )
    }

    /// Transfer tokens FROM the pool vault TO the user's account.
    ///
    /// PDA SIGNING EXPLANATION:
    ///
    /// The vault is owned by the pool PDA. PDAs cannot sign with a private key,
    /// but the program can "sign" on behalf of a PDA using the seeds that
    /// derive it.
    ///
    /// `signer_seeds` contains: `[b"pool", lp_mint_pubkey, bump]`
    ///
    /// The Solana runtime verifies these seeds derive the pool address.
    pub fn transfer_out(
        &self,
        vault: &AccountInfo<'info>,
        user_ata: &AccountInfo<'info>,
        amount: u64,
        signer_seeds: &[&[&[u8]]],
    ) -> Result<()> {
        token::transfer(
            CpiContext::new_with_signer(
                self.token_program.key(),
                Transfer {
                    from: vault.clone(),
                    to: user_ata.clone(),
                    authority: self.pool.to_account_info(),
                },
                signer_seeds,
            ),
            amount,
        )
    }
}

/// Swap tokens using the StableSwap invariant.
///
/// # Arguments
/// * `amount_in`     — Amount of input token to sell.
/// * `min_amount_out` — Minimum output tokens to receive (slippage guard).
/// * `input_index`   — Pool token index to sell.
/// * `output_index`  — Pool token index to receive.
pub fn swap_handler<'info>(
    ctx: Context<'_, '_, '_, 'info, Swap<'info>>,
    amount_in: u64,
    min_amount_out: u64,
    input_index: u8,
    output_index: u8,
) -> Result<()> {
    // Handler for the swap instruction.
    //
    // REMAINING ACCOUNTS for a 2-token pool:
    // - `[0]`: Token A vault (pool's ATA)
    // - `[1]`: Token B vault (pool's ATA)
    // - `[2]`: User's input token account
    // - `[3]`: User's output token account
    require!(amount_in > 0, StableSwapError::ZeroAmount);

    let pool = &ctx.accounts.pool;
    let remaining = ctx.remaining_accounts;

    // STEP 1: Validate indices and remaining-account layout.
    ctx.accounts
        .validate_remaining_accounts(remaining, input_index, output_index)?;

    require!(!pool.is_paused, StableSwapError::PoolPaused);
    let oracle_status = load_pair_status(
        &pool.oracle_config.oracle_a,
        &pool.oracle_config.oracle_b,
        &ctx.accounts.oracle_price_feed_a.to_account_info(),
        &ctx.accounts.oracle_price_feed_b.to_account_info(),
        DEFAULT_MAX_PRICE_AGE_SEC,
        pool.oracle_config.max_depeg_bps,
    )?;

    let reserves = ctx
        .accounts
        .read_reserves(&[&remaining[0], &remaining[1]])?;
    let reserve_a = reserves[0];
    let reserve_b = reserves[1];
    let amp = pool.amplification as u128;
    let base_fee_bps = pool.fee_bps;
    let max_dynamic_fee_bps = if pool.oracle_config.enabled {
        pool.oracle_config.emergency_fee_bps
    } else {
        pool.fee_bps
    };

    // Both reserves must be non-zero for a valid swap
    require!(reserve_a > 0 && reserve_b > 0, StableSwapError::EmptyPool);

    // Determine which side is in and which is out
    let reserve_by_index = [reserve_a, reserve_b];
    let price_by_index = [oracle_status.price_a, oracle_status.price_b];
    let reserve_in = reserve_by_index[input_index as usize];
    let reserve_out = reserve_by_index[output_index as usize];
    let oracle_price_in = price_by_index[input_index as usize];
    let oracle_price_out = price_by_index[output_index as usize];

    let quote = calculate_swap_output(
        reserve_in,
        reserve_out,
        amount_in as u128,
        amp,
        base_fee_bps,
        max_dynamic_fee_bps,
        oracle_price_in,
        oracle_price_out,
        pool.oracle_config.max_depeg_bps,
    )?;

    require!(
        quote.amount_out >= min_amount_out as u128,
        StableSwapError::SlippageExceeded
    );

    let seeds: &[&[u8]] = &[b"pool", pool.lp_mint.as_ref(), &[pool.bump]];

    ctx.accounts
        .transfer_in(&remaining[2], &remaining[input_index as usize], amount_in)?;
    ctx.accounts.transfer_out(
        &remaining[output_index as usize],
        &remaining[3],
        quote.amount_out as u64,
        &[seeds],
    )?;

    msg!(
        "Swap {}→{}: {} in → {} out (fee: {}, dynamic_fee={}bps, oracle_a={}bps, oracle_b={}bps)",
        input_index,
        output_index,
        amount_in,
        quote.amount_out,
        quote.fee_amount,
        quote.dynamic_fee_bps,
        oracle_status.peg_delta_a_bps,
        oracle_status.peg_delta_b_bps
    );
    Ok(())
}
