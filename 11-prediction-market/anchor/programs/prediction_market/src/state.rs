use anchor_lang::prelude::*;

/// Maximum length of a market question
pub const MAX_QUESTION_LEN: usize = 200;

/// Market account storing prediction market state
#[account]
#[derive(InitSpace)]
pub struct Market {
    /// Creator who can resolve the market
    pub creator: Pubkey,
    /// Unique market ID (per creator)
    pub market_id: u64,
    /// The prediction question
    #[max_len(MAX_QUESTION_LEN)]
    pub question: String,
    /// Unix timestamp when betting closes and resolution can occur
    pub resolution_time: i64,
    /// Total lamports bet on YES
    pub yes_pool: u64,
    /// Total lamports bet on NO
    pub no_pool: u64,
    /// Whether the market has been resolved
    pub resolved: bool,
    /// The winning outcome (None until resolved, Some(true) = YES won)
    pub outcome: Option<bool>,
    /// PDA bump seed
    pub bump: u8,
}

/// User position in a specific market
#[account]
#[derive(InitSpace)]
pub struct UserPosition {
    /// The market this position is for
    pub market: Pubkey,
    /// The user who owns this position
    pub user: Pubkey,
    /// Lamports bet on YES
    pub yes_amount: u64,
    /// Lamports bet on NO
    pub no_amount: u64,
    /// Whether winnings have been claimed
    pub claimed: bool,
    /// PDA bump seed
    pub bump: u8,
}
