use anchor_lang::prelude::*;

#[error_code]
pub enum MarketError {
    #[msg("Resolution time must be in the future")]
    ResolutionTimeInPast,
    #[msg("Betting is closed for this market")]
    BettingClosed,
    #[msg("Bet amount must be greater than zero")]
    InvalidBetAmount,
    #[msg("Market cannot be resolved yet")]
    ResolutionTooEarly,
    #[msg("Market has already been resolved")]
    AlreadyResolved,
    #[msg("Market has not been resolved yet")]
    NotResolved,
    #[msg("Winnings have already been claimed")]
    AlreadyClaimed,
    #[msg("No winnings to claim")]
    NoWinnings,
    #[msg("Arithmetic overflow")]
    Overflow,
}
