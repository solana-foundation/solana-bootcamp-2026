# Part 2: On-Chain Program

**Duration:** 30 min

---

## 2.1 — Account Structures (10 min | ~900 words)

<!-- Screen share code - walk through state.rs -->

Alright, before we write any code, we need to decide what lives on-chain. On Solana, programs are basically pure functions over accounts: each instruction takes accounts in, reads them, and writes them back out. So the account layout is the real API. Get that right and everything else becomes straightforward state transitions.

For this build, we're keeping it minimal. Two accounts: `Market` for global state, and `UserPosition` for a single user's exposure. No order books, no price history, no off-chain references. Just the data we need to validate bets and pay people out. That's the whole idea.

We define these in `state.rs` with `#[account]` and `#[derive(InitSpace)]`. Quick note: `InitSpace` matters because Solana makes you allocate space up front. Too small and the transaction fails, too big and you burn lamports forever. With `InitSpace` and `#[max_len]`, Anchor does the sizing for us, so we don't have to.

**Walkthrough: `anchor/programs/prediction_market/src/state.rs`**

```rust
pub struct Market {
    pub creator: Pubkey,        // 32 bytes - who can resolve
    pub market_id: u64,         // 8 bytes - unique ID
    pub question: String,       // variable - "Will X happen?"
    pub resolution_time: i64,   // 8 bytes - betting deadline
    pub yes_pool: u64,          // 8 bytes - total YES lamports
    pub no_pool: u64,           // 8 bytes - total NO lamports
    pub resolved: bool,         // 1 byte - is outcome set?
    pub outcome: Option<bool>,  // 2 bytes - None, Some(true), Some(false)
    pub bump: u8,               // 1 byte - PDA bump seed
}
```

Let's walk it quickly, and I'll keep it light. `creator` is the authority who can resolve the market. Think of it as the admin. We store it so later instructions can check the signer. It also goes into the PDA seeds, so one creator can run a bunch of markets without collisions.

`market_id` is a u64 we pass in. It's only unique per creator, like a per-creator sequence number. The PDA seed uses `b"market" + creator + market_id`, so two creators can both have market_id 1 and still get different addresses. The cool part is deterministic discovery: given a creator and id, a client can derive the market address without any registry.

`question` is the prompt. We cap it at 200 characters with `MAX_QUESTION_LEN` so the account stays small and predictable. A `String` in Anchor includes a 4 byte length prefix plus the bytes, so max length matters for rent. It also keeps the UI tidy.

`resolution_time` is an i64 unix timestamp. We use it to reject markets created in the past and to close betting after the deadline. On Solana, `Clock::get()?.unix_timestamp` is the source of time. It's not perfectly precise, but it's fine for a tutorial.

`yes_pool` and `no_pool` are just running totals of lamports on each side. When a user bets, we move lamports into the market PDA and increment one of these pools. There is no pricing curve here; the implied probability is just the ratio of the two pools. Simple parimutuel math, nothing fancy.

`resolved` and `outcome` work together. `resolved` is a quick guard that prevents double resolution. `outcome` is an `Option<bool>` so we can represent three states: `None` (unresolved), `Some(true)` (YES wins), and `Some(false)` (NO wins). This avoids confusing "not resolved" with "NO".

`bump` stores the PDA bump seed. We compute it when the account is created and store it on-chain so later instructions can re-derive the PDA without the client passing it. You'll see this in the account constraints for `claim_winnings`, where the PDA is validated by seeds and bump.

Next up is `UserPosition`. This account is created per user per market, and it aggregates their bets over time. Instead of making a new account for every bet, we keep one account and update its totals. That keeps account management and UI logic simple.

```rust
pub struct UserPosition {
    pub market: Pubkey,      // which market this position belongs to
    pub user: Pubkey,        // owner of the position
    pub yes_amount: u64,     // lamports bet on YES
    pub no_amount: u64,      // lamports bet on NO
    pub claimed: bool,       // has the payout been claimed?
    pub bump: u8,            // PDA bump seed
}
```

So `market` and `user` just tie this position to a specific market and owner. Storing both makes the account self-describing and lets us add constraints like `user_position.user == user.key()` later.

`yes_amount` and `no_amount` are cumulative totals. We intentionally allow both to be non-zero, which means a user can hedge or change their mind by placing bets on both sides. We don't net anything out here; when we pay out, only the winning side counts.

`claimed` is a one-way flag. Once a user withdraws their winnings, we set it to true and refuse any further claims. It prevents double spending even if someone resubmits the same transaction. `bump` plays the same role as in the `Market` account: it lets us re-derive the PDA deterministically.

Quick note on space: for a `Market`, the size is the fixed fields plus the 4 byte string length prefix and the 200 byte max question. For `UserPosition`, it is mostly fixed: two pubkeys, two u64 totals, a bool, and a bump. Anchor's `INIT_SPACE` keeps this accurate so we can allocate `8 + Market::INIT_SPACE` and `8 + UserPosition::INIT_SPACE` without hand math.

Net result: a small, stable account model. It's also easy to extend: you could add a `fee_bps` field, an `oracle` pubkey, or a `category` enum without touching the core flow. For the tutorial, these two accounts are enough. Now we can move on to the instruction logic that mutates them.

**Discussion Points:**
- Why each field exists
- How `Option<bool>` represents three states (unresolved, yes, no)
- Why we keep one `UserPosition` per user per market
- How PDA seeds give deterministic addresses
- Space calculation for rent exemption


---

## 2.2a — Function: create_market (5 min | ~400 words)

<!-- Code walkthrough - highlight validation pattern -->

Alright, `create_market` is where everything starts. Think of it as the setup step. It allocates the market PDA, stores the metadata, and zeros the pools. In Anchor, most of the setup lives in the accounts struct rather than the instruction body.

Take a quick look at the `CreateMarket` accounts in `lib.rs`: we use `init`, `payer = creator`, `space = 8 + Market::INIT_SPACE`, and the seeds `b"market"`, the creator pubkey, and the `market_id` bytes. That gives us a deterministic address. Same creator + id means the same PDA every time. Try to create it twice and the second transaction fails because the account already exists.

The inputs are simple: `market_id`, the question string, and the resolution time. We validate both before touching state. The question length check enforces `MAX_QUESTION_LEN` so the account fits the space we allocated. The time check ensures the market is in the future; otherwise you'd create a market that is already closed.

**Walkthrough: `anchor/programs/prediction_market/src/lib.rs`**

```rust
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
```

After validation, we fill every field explicitly. No surprises. That keeps the account predictable and avoids relying on defaults. We also store the bump from `ctx.bumps.market` so later instructions can validate the PDA without the client supplying the bump each time.

By the way, we store `market_id` on the account even though it is already part of the PDA seeds. It's handy for the UI and it lets us re-derive the PDA later in `claim_winnings` without asking the user to pass it. We also store `resolution_time` as an i64 because the clock sysvar uses i64, so we avoid casting. In production you'd probably add more validation here - a minimum duration, a sanity check on the question content, or a small creation fee to prevent spam. For the tutorial, the two guards are enough.

One quick design choice: we don't store a global list of markets. On Solana, enumerating all accounts is an RPC or indexer problem, not a program responsibility. Keeping the program focused makes it cheaper to run and easier to audit. Clients can discover markets by scanning PDAs or by indexing off-chain if they need a search experience.

**Key Point:** Validation happens BEFORE state changes


---

## 2.2b — Function: place_bet (7 min | ~500 words)

<!-- Code + money flow diagram - slower pace -->

Alright, `place_bet` is the hot path. Every YES/NO click hits this, so we keep it lean. Think of it as three steps: validate, move lamports into the market PDA, and update accounting in both the market and the user's position.

First guard: `amount > 0`. It sounds trivial, but it prevents no-op transactions that could still create position accounts and waste rent. Next guard checks the deadline: if `Clock::get()?.unix_timestamp` is greater than or equal to `resolution_time`, betting is closed. We do this before any transfer so we never move funds after the cutoff.

Then we do the transfer. Nothing fancy: we use the system program to move native lamports from the user to the market PDA. The user signs the transaction, and the market account simply receives funds. Because the PDA is program-owned, it does not need to sign to receive lamports. If you wanted to bet with an SPL token like USDC, this is where you'd CPI into the token program instead.

**Diagram: Money Flow**

```
   User Wallet                    Market PDA
       │                              │
       │    transfer(amount)          │
       ├─────────────────────────────►│
       │                              │
       ▼                              ▼
  balance -= amt              yes_pool += amt
                                  (or no_pool)
```

**Walkthrough: `anchor/programs/prediction_market/src/lib.rs`**

```rust
pub fn place_bet(
    ctx: Context<PlaceBet>,
    amount: u64,
    bet_yes: bool,
) -> Result<()> {
    require!(amount > 0, MarketError::InvalidBetAmount);

    let clock = Clock::get()?;
    let market = &ctx.accounts.market;
    require!(
        clock.unix_timestamp < market.resolution_time,
        MarketError::BettingClosed
    );

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

    // Update pools
    let market = &mut ctx.accounts.market;
    if bet_yes {
        market.yes_pool = market.yes_pool.checked_add(amount)
            .ok_or(MarketError::Overflow)?;
    } else {
        market.no_pool = market.no_pool.checked_add(amount)
            .ok_or(MarketError::Overflow)?;
    }

    // Update user's position...
}
```

Only after the transfer succeeds do we update the pool totals. We use `checked_add` on u64 to prevent overflow. Overflow is rare in normal usage, but it is a common attack surface: if someone could wrap a pool to zero, they could distort the implied price or steal funds. Defensive math keeps the accounting sane.

Next we update the user's position. The `user_position` account is created with `init_if_needed`, which means the first bet pays rent and subsequent bets reuse the same account. On the first bet we set `market` and `user`, and we store the bump so we can verify the PDA later. On every bet we increment either `yes_amount` or `no_amount`.

Yep, a user can bet both sides over time. That's intentional. Some traders want to hedge, or to change their mind without closing an account. We don't net these positions out. When the market resolves, only the winning side counts. The losing side remains in the pool and is distributed to winners.

This is a simple parimutuel model, not an AMM. No curve, no slippage, no price protection. The "price" is just the ratio of pools at any given moment. That makes the logic easy to reason about and perfect for a tutorial. Once you understand this flow, you can swap in more advanced pricing if you want.

We also don't store an explicit price; the UI derives it on the fly from pool ratios. This keeps the on-chain state minimal and avoids extra rounding logic.

**Key Insight:** `checked_add` returns `None` on overflow instead of wrapping. This prevents attacks where someone could overflow the pool to zero.


---

## 2.2c — Function: claim_winnings (8 min | ~550 words)

<!-- Payout diagram + code - let math sink in -->

Alright, `claim_winnings` is the settlement step. It's the only instruction that moves lamports out of the market PDA, so we slow down and double-check everything here. The flow is: verify the market is resolved, verify the user has not claimed, compute the user's share, transfer lamports, and mark the claim as complete.

Before this happens, someone has to call `resolve_market` to set the outcome. In this tutorial, only the market creator can do that, and they can only do it after the resolution time. That's a trust assumption we're making to keep the program simple. In production you'd typically replace this with an oracle or a multi-sig to avoid a single point of failure.

**Diagram: Payout Calculation**

```
Example Market:
┌─────────────────────────────────────┐
│  YES Pool: 100 SOL                  │
│  NO Pool:   50 SOL                  │
│  Outcome: YES wins                  │
└─────────────────────────────────────┘

User bet 10 SOL on YES:
┌─────────────────────────────────────┐
│  Share of winners: 10/100 = 10%     │
│  Claim from losers: 50 × 10% = 5    │
│  Total payout: 10 + 5 = 15 SOL      │
└─────────────────────────────────────┘
```

**Walkthrough: `anchor/programs/prediction_market/src/lib.rs`**

```rust
pub fn claim_winnings(ctx: Context<ClaimWinnings>) -> Result<()> {
    let market = &ctx.accounts.market;
    let position = &ctx.accounts.user_position;

    // Guards
    require!(market.resolved, MarketError::NotResolved);
    require!(!position.claimed, MarketError::AlreadyClaimed);

    // Determine winning side
    let outcome = market.outcome.unwrap();
    let (user_winning_bet, total_winning_pool, total_losing_pool) = if outcome {
        (position.yes_amount, market.yes_pool, market.no_pool)
    } else {
        (position.no_amount, market.no_pool, market.yes_pool)
    };

    require!(user_winning_bet > 0, MarketError::NoWinnings);

    // Calculate payout
    let winnings = (user_winning_bet as u128)
        .checked_mul(total_losing_pool as u128)
        .ok_or(MarketError::Overflow)?
        .checked_div(total_winning_pool as u128)
        .ok_or(MarketError::Overflow)? as u64;
    let total_payout = user_winning_bet
        .checked_add(winnings)
        .ok_or(MarketError::Overflow)?;

    // Transfer from PDA to user
    let market_account_info = ctx.accounts.market.to_account_info();
    let user_account_info = ctx.accounts.user.to_account_info();

    **market_account_info.try_borrow_mut_lamports()? -= total_payout;
    **user_account_info.try_borrow_mut_lamports()? += total_payout;

    let position = &mut ctx.accounts.user_position;
    position.claimed = true;

    Ok(())
}
```

Pretty straightforward: the first guards are `market.resolved` and `!position.claimed`. Simple checks, but important. Without them, a user could claim before resolution or claim multiple times. We also verify that the user has a winning bet by checking that `user_winning_bet > 0`. If you only bet on the losing side, there is nothing to claim.

We determine the winning side by reading `market.outcome`. Because the market is resolved, this option should be `Some(true)` or `Some(false)`. We then pick the user's winning amount and the total winning and losing pools. That is all we need for payout; we don't store any per-bet history.

The payout formula is parimutuel: `winnings = (user_bet / winning_pool) * losing_pool`. In the actual code we do this with u128 math to avoid overflow when multiplying large numbers. Integer division floors the result, so there may be a few lamports left in the market PDA. That is normal in integer arithmetic and keeps the program deterministic.

To transfer lamports, we can't call the system program. The market PDA is program-owned, and the system program only moves lamports from signer accounts. Instead we directly mutate the lamports field on the market and user accounts. This is safe because the program owns the market account and we have already validated the user and the seeds in the account constraints.

We also rely on those account constraints to make sure the `user_position` PDA matches the signer.

After the transfer, we flip `position.claimed` to true. That makes the claim idempotent: if the user submits the transaction again, it will fail at the guard. We don't close the position account in this version, but we could add a `close = user` constraint to reclaim rent once claimed.

One last nuance worth calling out: if a user bet on both sides, only the winning side is paid out. The losing side stays in the pool and is distributed to winners, which includes them if they have a winning bet. This keeps the math consistent even when users hedge. The overall totals still balance because all lamports entering the market are either paid out or remain as dust.

This wraps the on-chain flow. We now have a full cycle: create a market, place bets, resolve it, and claim winnings. Next we'll take this program interface and generate a TypeScript client so the frontend can call these instructions with type safety.
