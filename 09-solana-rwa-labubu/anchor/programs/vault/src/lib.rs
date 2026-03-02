use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::Token2022,
    token_interface::TokenAccount,
};

#[cfg(test)]
mod tests;

declare_id!("6RSKyunojsMYHZQWzeP6SPGXnmZwjHAcPcbEumk7Cw2g");

const TOTAL_LABUBU_TYPES: usize = 11;
const NORMAL_SUPPLY: u16 = 120;
const RARE_SUPPLY: u16 = 6;

#[program]
pub mod vault {
    use super::*;
    use anchor_spl::token_interface;

    /// Initialize the Labubu Collection
    pub fn initialize_collection(ctx: Context<InitializeCollection>) -> Result<()> {
        let collection = &mut ctx.accounts.collection;

        for i in 0..TOTAL_LABUBU_TYPES {
            collection.remaining_supply[i] = if i < 10 { NORMAL_SUPPLY } else { RARE_SUPPLY };
        }

        collection.total_minted = 0;
        collection.authority = ctx.accounts.authority.key();

        msg!("Labubu Collection initialized!");
        Ok(())
    }

    /// Create a Token Mint for a specific Labubu ID
    pub fn create_labubu_mint(ctx: Context<CreateLabubuMint>, labubu_id: u8) -> Result<()> {
        require!(labubu_id >= 1 && labubu_id <= 11, LabubuError::InvalidLabubuId);

        let rent = Rent::get()?;
        let mint_size = 82;
        let lamports = rent.minimum_balance(mint_size);

        // Step 1: Create mint account
        anchor_lang::system_program::create_account(
            CpiContext::new_with_signer(
                ctx.accounts.system_program.to_account_info(),
                anchor_lang::system_program::CreateAccount {
                    from: ctx.accounts.authority.to_account_info(),
                    to: ctx.accounts.mint.to_account_info(),
                },
                &[&[b"labubu_mint".as_ref(), &[labubu_id], &[ctx.bumps.mint]]],
            ),
            lamports,
            mint_size as u64,
            &ctx.accounts.token_program.key(),
        )?;

        // Step 2: Initialize Token Mint
        let cpi_context = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token_interface::InitializeMint2 {
                mint: ctx.accounts.mint.to_account_info(),
            },
        );

        token_interface::initialize_mint2(
            cpi_context,
            0,  // decimals = 0 (NFT)
            &ctx.accounts.collection.key(),  // mint_authority = collection PDA
            None,  // freeze_authority = None
        )?;

        msg!("Created Labubu #{} mint: {}", labubu_id, ctx.accounts.mint.key());
        Ok(())
    }

    /// Mint a Labubu NFT to a user
    pub fn mint_random(ctx: Context<MintRandom>, labubu_id: u8) -> Result<()> {
        let collection = &mut ctx.accounts.collection;

        require!(labubu_id >= 1 && labubu_id <= 11, LabubuError::InvalidLabubuId);
        let index = (labubu_id - 1) as usize;

        require!(collection.remaining_supply[index] > 0, LabubuError::SoldOut);
        collection.remaining_supply[index] -= 1;

        // PDA signing: collection PDA acts as mint authority
        let seeds = &[b"collection".as_ref(), &[ctx.bumps.collection]];
        let signer_seeds = &[&seeds[..]];

        let cpi_context = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token_interface::MintTo {
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.user_token_account.to_account_info(),
                authority: collection.to_account_info(),
            },
            signer_seeds,
        );

        token_interface::mint_to(cpi_context, 1)?;
        collection.total_minted += 1;

        msg!(
            "User {} minted Labubu #{} ({})",
            ctx.accounts.user.key(),
            labubu_id,
            ctx.accounts.mint.key()
        );

        Ok(())
    }
}

#[account]
pub struct LabubuCollection {
    pub authority: Pubkey,
    pub remaining_supply: [u16; TOTAL_LABUBU_TYPES],
    pub total_minted: u32,
}

#[derive(Accounts)]
pub struct InitializeCollection<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 22 + 4,
        seeds = [b"collection"],
        bump
    )]
    pub collection: Account<'info, LabubuCollection>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(labubu_id: u8)]
pub struct CreateLabubuMint<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        seeds = [b"collection"],
        bump,
        has_one = authority
    )]
    pub collection: Account<'info, LabubuCollection>,

    /// CHECK: Safe because we create and initialize it
    #[account(
        mut,
        seeds = [b"labubu_mint", &[labubu_id]],
        bump
    )]
    pub mint: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(labubu_id: u8)]
pub struct MintRandom<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"collection"],
        bump,
    )]
    pub collection: Account<'info, LabubuCollection>,

    /// CHECK: Verified by seeds
    #[account(
        mut,
        seeds = [b"labubu_mint", &[labubu_id]],
        bump,
    )]
    pub mint: UncheckedAccount<'info>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = mint,
        associated_token::authority = user,
        associated_token::token_program = token_program
    )]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[error_code]
pub enum LabubuError {
    #[msg("All Labubu sold out")]
    SoldOut,
    #[msg("Invalid Labubu ID (must be 1-11)")]
    InvalidLabubuId,
}
