use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

mod errors;
mod state;

// Tests temporarily disabled - see tests.rs for LiteSVM tests
// #[cfg(test)]
// mod tests;

use errors::MarketError;
use state::{Market, UserPosition, MAX_QUESTION_LEN};

declare_id!("33ijjeqroG7Syj3fqJn7oebG6iAwk1pGq5deBNYzkr1h");

#[program]
pub mod prediction_market {
    use super::*;

    /// Create a new prediction market
    pub fn create_market(
        ctx: Context<CreateMarket>,
        market_id: u64,
        question: String,
        resolution_time: i64,
    ) -> Result<()> {
        require!(question.len() <= MAX_QUESTION_LEN, MarketError::Overflow);

        let clock = Clock::get()?;
        require!(
            resolution_time > clock.unix_timestamp,
            MarketError::ResolutionTimeInPast
        );

        let market = &mut ctx.accounts.market;
        market.creator = ctx.accounts.creator.key();
        market.market_id = market_id;
        market.question = question;
        market.resolution_time = resolution_time;
        market.yes_pool = 0;
        market.no_pool = 0;
        market.resolved = false;
        market.outcome = None;
        market.bump = ctx.bumps.market;

        Ok(())
    }

    /// Place a bet on YES or NO
    pub fn place_bet(ctx: Context<PlaceBet>, amount: u64, bet_yes: bool) -> Result<()> {
        require!(amount > 0, MarketError::InvalidBetAmount);

        let clock = Clock::get()?;
        let market = &ctx.accounts.market;
        require!(
            clock.unix_timestamp < market.resolution_time,
            MarketError::BettingClosed
        );

        // Transfer SOL from user to market PDA
        transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user.to_account_info(),
                    to: ctx.accounts.market.to_account_info(),
                },
            ),
            amount,
        )?;

        // Update market pools
        let market = &mut ctx.accounts.market;
        if bet_yes {
            market.yes_pool = market.yes_pool.checked_add(amount).ok_or(MarketError::Overflow)?;
        } else {
            market.no_pool = market.no_pool.checked_add(amount).ok_or(MarketError::Overflow)?;
        }

        // Update user position - set initial values only if not already set
        let position = &mut ctx.accounts.user_position;
        if position.market == Pubkey::default() {
            position.market = market.key();
            position.user = ctx.accounts.user.key();
            // Calculate bump for the position PDA
            let (_, bump) = Pubkey::find_program_address(
                &[b"position", market.key().as_ref(), ctx.accounts.user.key().as_ref()],
                ctx.program_id,
            );
            position.bump = bump;
        }

        if bet_yes {
            position.yes_amount = position
                .yes_amount
                .checked_add(amount)
                .ok_or(MarketError::Overflow)?;
        } else {
            position.no_amount = position
                .no_amount
                .checked_add(amount)
                .ok_or(MarketError::Overflow)?;
        }

        Ok(())
    }

    /// Resolve the market with the winning outcome
    pub fn resolve_market(ctx: Context<ResolveMarket>, outcome: bool) -> Result<()> {
        let clock = Clock::get()?;
        let market = &ctx.accounts.market;

        require!(
            clock.unix_timestamp >= market.resolution_time,
            MarketError::ResolutionTooEarly
        );
        require!(!market.resolved, MarketError::AlreadyResolved);

        let market = &mut ctx.accounts.market;
        market.resolved = true;
        market.outcome = Some(outcome);

        Ok(())
    }

    /// Claim winnings after market resolution
    pub fn claim_winnings(ctx: Context<ClaimWinnings>) -> Result<()> {
        let market = &ctx.accounts.market;
        let position = &ctx.accounts.user_position;

        require!(market.resolved, MarketError::NotResolved);
        require!(!position.claimed, MarketError::AlreadyClaimed);

        let outcome = market.outcome.unwrap();

        // Calculate winnings
        let (user_winning_bet, total_winning_pool, total_losing_pool) = if outcome {
            (position.yes_amount, market.yes_pool, market.no_pool)
        } else {
            (position.no_amount, market.no_pool, market.yes_pool)
        };

        require!(user_winning_bet > 0, MarketError::NoWinnings);

        // Calculate payout: original bet + share of losing pool
        // winnings = (user_bet / winning_pool) * losing_pool
        let winnings = (user_winning_bet as u128)
            .checked_mul(total_losing_pool as u128)
            .ok_or(MarketError::Overflow)?
            .checked_div(total_winning_pool as u128)
            .ok_or(MarketError::Overflow)? as u64;

        let total_payout = user_winning_bet
            .checked_add(winnings)
            .ok_or(MarketError::Overflow)?;

        // Transfer lamports directly from market PDA to user
        // (Cannot use system_program::transfer because market account is program-owned)
        let market_account_info = ctx.accounts.market.to_account_info();
        let user_account_info = ctx.accounts.user.to_account_info();

        **market_account_info.try_borrow_mut_lamports()? -= total_payout;
        **user_account_info.try_borrow_mut_lamports()? += total_payout;

        // Mark as claimed
        let position = &mut ctx.accounts.user_position;
        position.claimed = true;

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(market_id: u64, question: String)]
pub struct CreateMarket<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
        init,
        payer = creator,
        space = 8 + Market::INIT_SPACE,
        seeds = [b"market", creator.key().as_ref(), &market_id.to_le_bytes()],
        bump,
    )]
    pub market: Account<'info, Market>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct PlaceBet<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub market: Account<'info, Market>,

    #[account(
        init_if_needed,
        payer = user,
        space = 8 + UserPosition::INIT_SPACE,
        seeds = [b"position", market.key().as_ref(), user.key().as_ref()],
        bump,
    )]
    pub user_position: Account<'info, UserPosition>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ResolveMarket<'info> {
    #[account(
        constraint = creator.key() == market.creator
    )]
    pub creator: Signer<'info>,

    #[account(mut)]
    pub market: Account<'info, Market>,
}

#[derive(Accounts)]
pub struct ClaimWinnings<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"market", market.creator.as_ref(), &market.market_id.to_le_bytes()],
        bump = market.bump,
    )]
    pub market: Account<'info, Market>,

    #[account(
        mut,
        seeds = [b"position", market.key().as_ref(), user.key().as_ref()],
        bump = user_position.bump,
        constraint = user_position.user == user.key(),
    )]
    pub user_position: Account<'info, UserPosition>,
}
