use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::solana_program::program::invoke;
use anchor_lang::system_program;

declare_id!("HzEfEnt2E6T6gmy9VQi2d15TN5PYAy78iq7WHPF9ddHB");

pub const MIN_DEPOSIT_AMOUNT: u64 = 1_000_000; // 0.001 SOL

pub const TREE_DEPTH: usize = 10;
pub const MAX_LEAVES: u64 = 1 << TREE_DEPTH; // 1024
pub const ROOT_HISTORY_SIZE: usize = 10;

// Empty tree root using Poseidon2
pub const EMPTY_ROOT: [u8; 32] = [
    0x2a, 0x77, 0x5e, 0xa7, 0x61, 0xd2, 0x04, 0x35, 0xb3, 0x1f, 0xa2, 0xc3, 0x3f, 0xf0, 0x76, 0x63,
    0xe2, 0x45, 0x42, 0xff, 0xb9, 0xe7, 0xb2, 0x93, 0xdf, 0xce, 0x30, 0x42, 0xeb, 0x10, 0x46, 0x86,
];

#[program]
pub mod private_transfers {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        pool.authority = ctx.accounts.authority.key();
        pool.total_deposits = 0;
        pool.next_leaf_index = 0;
        pool.current_root_index = 0;
        pool.roots[0] = EMPTY_ROOT;

        msg!("Pool initialized");
        Ok(())
    }

    pub fn deposit(
        ctx: Context<Deposit>,
        commitment: [u8; 32],
        new_root: [u8; 32],
        amount: u64,
    ) -> Result<()> {
        let pool = &mut ctx.accounts.pool;

        require!(
            amount >= MIN_DEPOSIT_AMOUNT,
            PrivateTransfersError::DepositTooSmall
        );
        require!(
            pool.next_leaf_index < MAX_LEAVES,
            PrivateTransfersError::TreeFull
        );

        let cpi_context = CpiContext::new(
            *ctx.accounts.system_program.key,
            system_program::Transfer {
                from: ctx.accounts.depositor.to_account_info(),
                to: ctx.accounts.pool_vault.to_account_info(),
            },
        );
        system_program::transfer(cpi_context, amount)?;
        let leaf_index = pool.next_leaf_index;
        let new_root_index = ((pool.current_root_index + 1) % ROOT_HISTORY_SIZE as u64) as usize;
        pool.roots[new_root_index] = new_root;
        pool.current_root_index = new_root_index as u64;

        emit!(DepositEvent {
            commitment,
            leaf_index,
            timestamp: Clock::get()?.unix_timestamp,
            new_root,
        });

        pool.total_deposits += 1;
        pool.next_leaf_index += 1;

        msg!("Deposit: {} lamports, commitment: {:?}", amount, commitment);
        Ok(())
    }

    pub fn withdraw(
        ctx: Context<Withdraw>,
        root: [u8; 32],
        recipient: Pubkey,
        amount: u64,
    ) -> Result<()> {
        require!(
            ctx.accounts.pool.is_known_root(&root),
            PrivateTransfersError::InvalidRoot
        );
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
    pub next_leaf_index: u64,
    pub current_root_index: u64,
    pub roots: [[u8; 32]; ROOT_HISTORY_SIZE], // array of 10 poseidon hashes
}

impl Pool {
    pub fn is_known_root(&self, root: &[u8; 32]) -> bool {
        self.roots.iter().any(|r| r == root)
    }
}

#[event]
pub struct DepositEvent {
    commitment: [u8; 32],
    pub leaf_index: u64,
    pub new_root: [u8; 32],
    pub timestamp: i64,
}

#[event]
pub struct WithdrawEvent {
    pub recipient: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[error_code]
pub enum PrivateTransfersError {
    #[msg("Deposit amount too small")]
    DepositTooSmall,
    #[msg("Recipient mismatch")]
    RecipientMismatch,
    #[msg("Insufficient vault balance")]
    InsufficientVaultBalance,
    #[msg("Merkle tree is full")]
    TreeFull,
    #[msg("Invalid root")]
    InvalidRoot,
}
