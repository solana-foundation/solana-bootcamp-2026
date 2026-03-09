use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::solana_program::program::invoke;
use anchor_lang::system_program;

declare_id!("HzEfEnt2E6T6gmy9VQi2d15TN5PYAy78iq7WHPF9ddHB");

// Step 2: Add Merkle tree constants here
// Step 5: Add SUNSPOT_VERIFIER_ID here

pub const MIN_DEPOSIT_AMOUNT: u64 = 1_000_000; // 0.001 SOL

#[program]
pub mod private_transfers {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        pool.authority = ctx.accounts.authority.key();
        pool.total_deposits = 0;
        // Step 2: Initialize next_leaf_index, current_root_index, roots[0]
        // Step 3: Initialize nullifier_set.pool

        msg!("Pool initialized");
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, commitment: [u8; 32], amount: u64) -> Result<()> {
        let pool = &mut ctx.accounts.pool;

        require!(
            amount >= MIN_DEPOSIT_AMOUNT,
            PrivateTransfersError::DepositTooSmall
        );
        // Step 2: Add tree full check

        let cpi_context = CpiContext::new(
            *ctx.accounts.system_program.key,
            system_program::Transfer {
                from: ctx.accounts.depositor.to_account_info(),
                to: ctx.accounts.pool_vault.to_account_info(),
            },
        );
        system_program::transfer(cpi_context, amount)?;

        // Step 2: Save leaf_index, update root history

        emit!(DepositEvent {
            commitment,
            amount,
            timestamp: Clock::get()?.unix_timestamp,
        });

        pool.total_deposits += 1;
        // Step 2: Increment next_leaf_index

        msg!("Deposit: {} lamports, commitment: {:?}", amount, commitment);
        Ok(())
    }

    pub fn withdraw(
        ctx: Context<Withdraw>,
        // Step 5: Add proof: Vec<u8>
        // Step 3: Add nullifier_hash: [u8; 32]
        // Step 2: Add root: [u8; 32]
        recipient: Pubkey,
        amount: u64,
    ) -> Result<()> {
        // Step 3: Check nullifier not used
        // Step 2: Validate root is known

        require!(
            ctx.accounts.recipient.key() == recipient,
            PrivateTransfersError::RecipientMismatch
        );

        require!(
            ctx.accounts.pool_vault.lamports() >= amount,
            PrivateTransfersError::InsufficientVaultBalance
        );

        // Step 5: Verify ZK proof via CPI
        // Step 3: Mark nullifier as used

        let pool_key = ctx.accounts.pool.key();
        let seeds = &[
            b"vault".as_ref(),
            pool_key.as_ref(),
            &[ctx.bumps.pool_vault],
        ];
        let signer_seeds = &[&seeds[..]];

        let cpi_context = CpiContext::new_with_signer(
            *ctx.accounts.system_program.key,
            system_program::Transfer {
                from: ctx.accounts.pool_vault.to_account_info(),
                to: ctx.accounts.recipient.to_account_info(),
            },
            signer_seeds,
        );
        system_program::transfer(cpi_context, amount)?;

        emit!(WithdrawEvent {
            recipient: ctx.accounts.recipient.key(),
            amount,
            timestamp: Clock::get()?.unix_timestamp,
            // Step 3: Replace amount with nullifier_hash
        });

        msg!("Public withdrawal: {} lamports to {}", amount, recipient);
        Ok(())
    }
}

// Step 5: Add encode_public_inputs function here

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + Pool::INIT_SPACE,
        seeds = [b"pool"],
        bump
    )]
    pub pool: Account<'info, Pool>,

    // Step 3: Add nullifier_set account here
    #[account(seeds = [b"vault", pool.key().as_ref()], bump)]
    pub pool_vault: SystemAccount<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut, seeds = [b"pool"], bump)]
    pub pool: Account<'info, Pool>,

    #[account(mut, seeds = [b"vault", pool.key().as_ref()], bump)]
    pub pool_vault: SystemAccount<'info>,

    #[account(mut)]
    pub depositor: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(seeds = [b"pool"], bump)]
    pub pool: Account<'info, Pool>,

    // Step 3: Add nullifier_set account here
    #[account(mut, seeds = [b"vault", pool.key().as_ref()], bump)]
    pub pool_vault: SystemAccount<'info>,

    /// CHECK: Validated in instruction logic
    #[account(mut)]
    pub recipient: UncheckedAccount<'info>,

    // Step 5: Add verifier_program account here
    pub system_program: Program<'info, System>,
}

#[account]
#[derive(InitSpace)]
pub struct Pool {
    pub authority: Pubkey,
    pub total_deposits: u64,
    // Step 2: Add next_leaf_index, current_root_index, roots
}

// Step 2: Add is_known_root method to Pool
// Step 3: Add NullifierSet struct with is_nullifier_used and mark_nullifier_used methods

#[event]
pub struct DepositEvent {
    pub commitment: [u8; 32],
    pub amount: u64,
    pub timestamp: i64,
    // Step 2: Add leaf_index: u64, new_root: [u8; 32]
}

#[event]
pub struct WithdrawEvent {
    pub recipient: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
    // Step 3: Replace amount with nullifier_hash: [u8; 32]
}

#[error_code]
pub enum PrivateTransfersError {
    #[msg("Deposit amount too small")]
    DepositTooSmall,
    #[msg("Recipient mismatch")]
    RecipientMismatch,
    #[msg("Insufficient vault balance")]
    InsufficientVaultBalance,
    // Step 2: Add TreeFull, InvalidRoot
    // Step 3: Add NullifierUsed, NullifierSetFull
    // Step 5: Add InvalidVerifier
}
