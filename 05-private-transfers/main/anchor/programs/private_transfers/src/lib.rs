use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::solana_program::program::invoke;
use anchor_lang::system_program;

declare_id!("8g5mRSVbbZ28fXcHNTwjvA66ZpRRQLkRb6scndFTm5UG");

pub const SUNSPOT_VERIFIER_ID: Pubkey = pubkey!("G6ZGZt9iJqejZBdSh4EMXrYiUXkCtedrzxsjorwSm7Un");
pub const TREE_DEPTH: usize = 10;
pub const MAX_LEAVES: u64 = 1 << TREE_DEPTH;
pub const MIN_DEPOSIT_AMOUNT: u64 = 1_000_000;
pub const ROOT_HISTORY_SIZE: usize = 10;

// Empty tree root using Poseidon2 (noir-lang/poseidon compatible with @zkpassport/poseidon2)
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
        pool.next_leaf_index = 0;
        pool.total_deposits = 0;
        pool.current_root_index = 0;
        pool.roots[0] = EMPTY_ROOT;

        let nullifiers = &mut ctx.accounts.nullifier_set;
        nullifiers.pool = pool.key();

        msg!("Pool initialized");
        Ok(())
    }

    /// Client computes commitment and new_root offchain.
    /// Invalid roots will cause withdrawal proofs to fail.
    pub fn deposit(
        ctx: Context<Deposit>,
        commitment: [u8; 32],
        new_root: [u8; 32],
        amount: u64,
    ) -> Result<()> {
        let pool = &mut ctx.accounts.pool;

        require!(
            pool.next_leaf_index < MAX_LEAVES,
            PrivateTransfersError::TreeFull
        );

        require!(
            amount >= MIN_DEPOSIT_AMOUNT,
            PrivateTransfersError::DepositTooSmall
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
        pool.current_root_index = new_root_index as u64;
        pool.roots[new_root_index] = new_root;

        emit!(DepositEvent {
            commitment,
            leaf_index,
            timestamp: Clock::get()?.unix_timestamp,
            new_root,
        });

        pool.next_leaf_index += 1;
        pool.total_deposits += 1;

        msg!(
            "Deposit successful: {} lamports at leaf index {}",
            amount,
            leaf_index
        );
        Ok(())
    }

    pub fn withdraw(
        ctx: Context<Withdraw>,
        proof: Vec<u8>,
        nullifier_hash: [u8; 32],
        root: [u8; 32],
        recipient: Pubkey,
        amount: u64,
    ) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        let nullifier_set = &mut ctx.accounts.nullifier_set;

        require!(
            !nullifier_set.is_nullifier_used(&nullifier_hash),
            PrivateTransfersError::NullifierUsed
        );

        require!(
            pool.is_known_root(&root),
            PrivateTransfersError::InvalidRoot
        );

        // Prevents front-running by binding proof to recipient
        require!(
            ctx.accounts.recipient.key() == recipient,
            PrivateTransfersError::RecipientMismatch
        );

        require!(
            ctx.accounts.pool_vault.lamports() >= amount,
            PrivateTransfersError::InsufficientVaultBalance
        );

        // Verify ZK proof via CPI to Sunspot
        let public_inputs = encode_public_inputs(&root, &nullifier_hash, &recipient, amount);
        let instruction_data = [proof.as_slice(), public_inputs.as_slice()].concat();

        invoke(
            &Instruction {
                program_id: ctx.accounts.verifier_program.key(),
                accounts: vec![],
                data: instruction_data,
            },
            &[ctx.accounts.verifier_program.to_account_info()],
        )?;

        nullifier_set.mark_nullifier_used(nullifier_hash)?;

        let pool_key = pool.key();
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
            nullifier_hash,
            recipient: ctx.accounts.recipient.key(),
            timestamp: Clock::get()?.unix_timestamp,
        });

        msg!("Withdrawal: {} lamports to {}", amount, recipient);
        Ok(())
    }
}

/// Gnark witness format: 12-byte header + 4x32-byte public inputs
fn encode_public_inputs(
    root: &[u8; 32],
    nullifier_hash: &[u8; 32],
    recipient: &Pubkey,
    amount: u64,
) -> Vec<u8> {
    const NR_PUBLIC_INPUTS: u32 = 4;
    let mut inputs = Vec::with_capacity(12 + 128);

    // Header: num_public (4) | num_private (4) | vector_len (4)
    inputs.extend_from_slice(&NR_PUBLIC_INPUTS.to_be_bytes());
    inputs.extend_from_slice(&0u32.to_be_bytes());
    inputs.extend_from_slice(&NR_PUBLIC_INPUTS.to_be_bytes());

    inputs.extend_from_slice(root);
    inputs.extend_from_slice(nullifier_hash);
    inputs.extend_from_slice(recipient.as_ref());

    let mut amount_bytes = [0u8; 32];
    amount_bytes[24..32].copy_from_slice(&amount.to_be_bytes());
    inputs.extend_from_slice(&amount_bytes);

    inputs
}

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

    #[account(
        init,
        payer = authority,
        space = 8 + NullifierSet::INIT_SPACE,
        seeds = [b"nullifiers", pool.key().as_ref()],
        bump
    )]
    pub nullifier_set: Account<'info, NullifierSet>,

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
    #[account(mut, seeds = [b"pool"], bump)]
    pub pool: Account<'info, Pool>,

    #[account(mut, seeds = [b"nullifiers", pool.key().as_ref()], bump)]
    pub nullifier_set: Account<'info, NullifierSet>,

    #[account(mut, seeds = [b"vault", pool.key().as_ref()], bump)]
    pub pool_vault: SystemAccount<'info>,

    /// CHECK: Validated in instruction logic
    #[account(mut)]
    pub recipient: UncheckedAccount<'info>,

    /// CHECK: Validated by constraint
    #[account(constraint = verifier_program.key() == SUNSPOT_VERIFIER_ID @ PrivateTransfersError::InvalidVerifier)]
    pub verifier_program: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

#[account]
#[derive(InitSpace)]
pub struct Pool {
    pub authority: Pubkey,
    pub next_leaf_index: u64,
    pub total_deposits: u64,
    pub current_root_index: u64,
    pub roots: [[u8; 32]; ROOT_HISTORY_SIZE],
}

impl Pool {
    pub fn is_known_root(&self, root: &[u8; 32]) -> bool {
        self.roots.iter().any(|r| r == root)
    }
}

#[account]
#[derive(InitSpace)]
pub struct NullifierSet {
    pub pool: Pubkey,
    #[max_len(256)]
    pub nullifiers: Vec<[u8; 32]>,
}

impl NullifierSet {
    pub fn is_nullifier_used(&self, nullifier_hash: &[u8; 32]) -> bool {
        self.nullifiers.contains(nullifier_hash)
    }

    pub fn mark_nullifier_used(&mut self, nullifier_hash: [u8; 32]) -> Result<()> {
        require!(
            self.nullifiers.len() < 256,
            PrivateTransfersError::NullifierSetFull
        );
        self.nullifiers.push(nullifier_hash);
        Ok(())
    }
}

#[event]
pub struct DepositEvent {
    pub commitment: [u8; 32],
    pub leaf_index: u64,
    pub timestamp: i64,
    pub new_root: [u8; 32],
}

#[event]
pub struct WithdrawEvent {
    pub nullifier_hash: [u8; 32],
    pub recipient: Pubkey,
    pub timestamp: i64,
}

#[error_code]
pub enum PrivateTransfersError {
    #[msg("Merkle tree is full")]
    TreeFull,
    #[msg("Invalid Merkle root")]
    InvalidRoot,
    #[msg("Nullifier already used")]
    NullifierUsed,
    #[msg("Deposit amount too small (minimum 0.001 SOL)")]
    DepositTooSmall,
    #[msg("Nullifier set is full")]
    NullifierSetFull,
    #[msg("Recipient account does not match recipient parameter")]
    RecipientMismatch,
    #[msg("Invalid verifier program")]
    InvalidVerifier,
    #[msg("Insufficient vault balance for withdrawal")]
    InsufficientVaultBalance,
}
