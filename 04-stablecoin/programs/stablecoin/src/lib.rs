use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount, mint_to, burn, MintTo, Burn},
};

declare_id!("5inXknRB7Vz9yXYme7mycL2z3GYQ5frRgozMciUmn7Bd");

#[program]
pub mod stablecoin {
    use super::*;

    /// Initialize the stablecoin mint and config
    /// This creates a new token mint with the program PDA as the mint authority
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let config = &mut ctx.accounts.config;
        config.admin = ctx.accounts.admin.key();
        config.mint = ctx.accounts.mint.key();
        config.paused = false;
        config.bump = ctx.bumps.config;
        config.mint_bump = ctx.bumps.mint;

        Ok(())
    }

    /// Configure a minter with a specific allowance
    /// Only the admin can call this instruction
    /// If the minter already exists, this updates their allowance
    pub fn configure_minter(ctx: Context<ConfigureMinter>, allowance: u64) -> Result<()> {
        let minter_config = &mut ctx.accounts.minter_config;

        // If not initialized, set the minter address
        if !minter_config.is_initialized {
            minter_config.minter = ctx.accounts.minter.key();
            minter_config.amount_minted = 0;
            minter_config.is_initialized = true;
            minter_config.bump = ctx.bumps.minter_config;
        }

        minter_config.allowance = allowance;

        msg!("Configured minter {} with allowance {}", ctx.accounts.minter.key(), allowance);

        Ok(())
    }

    /// Remove a minter's authorization
    /// Only the admin can call this instruction
    /// This closes the minter config account and returns rent to admin
    pub fn remove_minter(_ctx: Context<RemoveMinter>) -> Result<()> {
        msg!("Minter removed");
        Ok(())
    }

    /// Mint new stablecoins to a user
    /// Only authorized minters can call this instruction
    /// The minter must have sufficient allowance remaining
    pub fn mint_tokens(ctx: Context<MintTokens>, amount: u64) -> Result<()> {
        let config = &ctx.accounts.config;

        // Check not paused
        require!(!config.paused, StablecoinError::Paused);

        // Check and update minter allowance
        let minter_config = &mut ctx.accounts.minter_config;
        let remaining = minter_config.allowance.checked_sub(minter_config.amount_minted)
            .ok_or(StablecoinError::ExceedsAllowance)?;
        require!(amount <= remaining, StablecoinError::ExceedsAllowance);

        minter_config.amount_minted = minter_config.amount_minted.checked_add(amount)
            .ok_or(StablecoinError::Overflow)?;

        // Create the signer seeds for the mint authority PDA
        let signer_seeds: &[&[&[u8]]] = &[&[b"config", &[config.bump]]];

        // Mint tokens to the destination account
        mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.key(),
                MintTo {
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.destination.to_account_info(),
                    authority: ctx.accounts.config.to_account_info(),
                },
                signer_seeds,
            ),
            amount,
        )?;

        msg!("Minted {} tokens to {}", amount, ctx.accounts.destination.key());

        Ok(())
    }

    /// Burn stablecoins from the caller's account
    /// Anyone can burn their own tokens
    /// In a real stablecoin, this would be called when users redeem for fiat
    pub fn burn_tokens(ctx: Context<BurnTokens>, amount: u64) -> Result<()> {
        burn(
            CpiContext::new(
                ctx.accounts.token_program.key(),
                Burn {
                    mint: ctx.accounts.mint.to_account_info(),
                    from: ctx.accounts.token_account.to_account_info(),
                    authority: ctx.accounts.owner.to_account_info(),
                },
            ),
            amount,
        )?;

        msg!("Burned {} tokens from {}", amount, ctx.accounts.token_account.key());

        Ok(())
    }

    /// Pause all minting operations
    /// Only the admin can call this instruction
    pub fn pause(ctx: Context<Pause>) -> Result<()> {
        ctx.accounts.config.paused = true;
        msg!("Stablecoin paused");
        Ok(())
    }

    /// Unpause minting operations
    /// Only the admin can call this instruction
    pub fn unpause(ctx: Context<Unpause>) -> Result<()> {
        ctx.accounts.config.paused = false;
        msg!("Stablecoin unpaused");
        Ok(())
    }
}

// ============================================================================
// Account Structures
// ============================================================================

/// Config account that stores the stablecoin configuration
#[account]
#[derive(InitSpace)]
pub struct Config {
    /// The admin who can configure minters and blacklist accounts
    pub admin: Pubkey,
    /// The mint address of the stablecoin
    pub mint: Pubkey,
    /// Whether minting is paused
    pub paused: bool,
    /// Bump seed for the config PDA
    pub bump: u8,
    /// Bump seed for the mint PDA
    pub mint_bump: u8,
}

/// Minter configuration account
/// Each authorized minter has their own config with an allowance
#[account]
#[derive(InitSpace)]
pub struct MinterConfig {
    /// The minter's public key
    pub minter: Pubkey,
    /// Maximum amount the minter can mint (total)
    pub allowance: u64,
    /// Amount already minted by this minter
    pub amount_minted: u64,
    /// Whether this account has been initialized
    pub is_initialized: bool,
    /// Bump seed for this PDA
    pub bump: u8,
}

// ============================================================================
// Instruction Contexts
// ============================================================================

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    /// The config account that stores stablecoin settings
    #[account(
        init,
        payer = admin,
        space = 8 + Config::INIT_SPACE,
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, Config>,

    /// The stablecoin mint
    /// The config PDA is set as both mint authority and freeze authority
    #[account(
        init,
        payer = admin,
        mint::decimals = 6,
        mint::authority = config,
        mint::freeze_authority = config,
        seeds = [b"mint"],
        bump
    )]
    pub mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ConfigureMinter<'info> {
    /// Only the admin can configure minters
    #[account(
        mut,
        constraint = admin.key() == config.admin @ StablecoinError::Unauthorized
    )]
    pub admin: Signer<'info>,

    #[account(
        seeds = [b"config"],
        bump = config.bump
    )]
    pub config: Account<'info, Config>,

    /// The minter being configured
    /// CHECK: This can be any account that will be authorized to mint
    pub minter: UncheckedAccount<'info>,

    /// The minter's configuration account
    #[account(
        init_if_needed,
        payer = admin,
        space = 8 + MinterConfig::INIT_SPACE,
        seeds = [b"minter", minter.key().as_ref()],
        bump
    )]
    pub minter_config: Account<'info, MinterConfig>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RemoveMinter<'info> {
    /// Only the admin can remove minters
    #[account(
        mut,
        constraint = admin.key() == config.admin @ StablecoinError::Unauthorized
    )]
    pub admin: Signer<'info>,

    #[account(
        seeds = [b"config"],
        bump = config.bump
    )]
    pub config: Account<'info, Config>,

    /// The minter being removed
    /// CHECK: This is the minter whose config is being closed
    pub minter: UncheckedAccount<'info>,

    /// The minter's configuration account to close
    #[account(
        mut,
        close = admin,
        seeds = [b"minter", minter.key().as_ref()],
        bump = minter_config.bump
    )]
    pub minter_config: Account<'info, MinterConfig>,
}

#[derive(Accounts)]
pub struct MintTokens<'info> {
    /// The minter calling this instruction
    #[account(mut)]
    pub minter: Signer<'info>,

    /// The config account
    #[account(
        seeds = [b"config"],
        bump = config.bump
    )]
    pub config: Account<'info, Config>,

    /// The minter's configuration - verifies they are authorized
    #[account(
        mut,
        seeds = [b"minter", minter.key().as_ref()],
        bump = minter_config.bump,
        constraint = minter_config.is_initialized @ StablecoinError::NotMinter
    )]
    pub minter_config: Account<'info, MinterConfig>,

    /// The stablecoin mint
    #[account(
        mut,
        seeds = [b"mint"],
        bump = config.mint_bump
    )]
    pub mint: Account<'info, Mint>,

    /// The destination token account to mint to
    #[account(
        init_if_needed,
        payer = minter,
        associated_token::mint = mint,
        associated_token::authority = destination_owner,
    )]
    pub destination: Account<'info, TokenAccount>,

    /// CHECK: The owner of the destination token account
    pub destination_owner: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct BurnTokens<'info> {
    /// The owner of the token account burning tokens
    pub owner: Signer<'info>,

    /// The config account
    #[account(
        seeds = [b"config"],
        bump = config.bump
    )]
    pub config: Account<'info, Config>,

    /// The stablecoin mint
    #[account(
        mut,
        seeds = [b"mint"],
        bump = config.mint_bump
    )]
    pub mint: Account<'info, Mint>,

    /// The token account to burn from
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = owner,
    )]
    pub token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}


#[derive(Accounts)]
pub struct Pause<'info> {
    /// Only the admin can pause
    #[account(
        constraint = admin.key() == config.admin @ StablecoinError::Unauthorized
    )]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [b"config"],
        bump = config.bump
    )]
    pub config: Account<'info, Config>,
}

#[derive(Accounts)]
pub struct Unpause<'info> {
    /// Only the admin can unpause
    #[account(
        constraint = admin.key() == config.admin @ StablecoinError::Unauthorized
    )]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [b"config"],
        bump = config.bump
    )]
    pub config: Account<'info, Config>,
}

// ============================================================================
// Error Codes
// ============================================================================

#[error_code]
pub enum StablecoinError {
    #[msg("You are not authorized to perform this action")]
    Unauthorized,
    #[msg("Minting is currently paused")]
    Paused,
    #[msg("Mint amount exceeds minter's remaining allowance")]
    ExceedsAllowance,
    #[msg("Account is not an authorized minter")]
    NotMinter,
    #[msg("Arithmetic overflow")]
    Overflow,
}
