# 90-Minute Video Lecture: Full-Stack Solana Prediction Markets

## Overview

**Format:** Pre-recorded video lecture
**Duration:** 90 minutes
**Style:** Architecture diagrams + function walkthroughs

### Learning Objectives

By the end, viewers will understand:
1. How Solana programs store and manage state
2. The Anchor framework's role in simplifying development
3. How type-safe clients are generated from on-chain code
4. End-to-end data flow in a real dApp

---

## Part 1: Why & What (15 min)

### 1.1 — The Problem (5 min)

- Traditional prediction markets: centralized, custodial, restricted
- Blockchain solution: trustless escrow, global access, transparent odds

### 1.2 — Architecture Overview (10 min)

**Diagram 1: System Layers**

```
┌────────────────────────────────────────────────────────┐
│                   BROWSER                              │
│  ┌──────────────────────────────────────────────────┐  │
│  │  Next.js App                                     │  │
│  │  • MarketsList, MarketCard, PositionsList        │  │
│  │  • Wallet connection via autoDiscover()          │  │
│  └──────────────────────────────────────────────────┘  │
│                         │                              │
│                         ▼                              │
│  ┌──────────────────────────────────────────────────┐  │
│  │  Generated Client (Codama)                       │  │
│  │  • Type-safe instruction builders                │  │
│  │  • Account encoders/decoders                     │  │
│  └──────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────┘
                          │ JSON-RPC
                          ▼
┌────────────────────────────────────────────────────────┐
│                   SOLANA DEVNET                        │
│  ┌─────────────────┐    ┌─────────────────────────┐   │
│  │ Anchor Program  │───▶│ Accounts (State)        │   │
│  │ (stateless)     │    │ • Market PDAs           │   │
│  │                 │    │ • UserPosition PDAs     │   │
│  └─────────────────┘    └─────────────────────────┘   │
└────────────────────────────────────────────────────────┘
```

**Diagram 2: Data Ownership**

```
        ┌─────────────┐
        │   Market    │ ← Holds pooled SOL
        │    PDA      │   (no private key!)
        └─────────────┘
              │
    ┌─────────┴─────────┐
    ▼                   ▼
┌─────────┐       ┌─────────┐
│Position │       │Position │  ← Tracks each user's bets
│ User A  │       │ User B  │
└─────────┘       └─────────┘
```

**Key Insight:** PDAs are addresses with no private key — only the program can sign for them. This makes them perfect for holding funds trustlessly.

---

## Part 2: On-Chain Program (30 min)

### 2.1 — Account Structures (10 min)

**Walkthrough: `state.rs`**

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

**Discussion Points:**
- Why each field exists
- How `Option<bool>` represents three states (unresolved, yes, no)
- Space calculation for rent exemption

---

### 2.2 — Core Functions (20 min)

#### Function 1: `create_market` (5 min)

```rust
pub fn create_market(
    ctx: Context<CreateMarket>,
    market_id: u64,
    question: String,
    resolution_time: i64,
) -> Result<()> {
    // Validation
    require!(resolution_time > Clock::get()?.unix_timestamp,
             ResolutionTimeInPast);
    require!(question.len() <= 200, QuestionTooLong);

    // Initialize state
    let market = &mut ctx.accounts.market;
    market.creator = ctx.accounts.creator.key();
    market.question = question;
    market.yes_pool = 0;
    market.no_pool = 0;
    // ...
}
```

**Key Point:** Validation happens BEFORE state changes

---

#### Function 2: `place_bet` (7 min)

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

```rust
pub fn place_bet(
    ctx: Context<PlaceBet>,
    amount: u64,
    bet_yes: bool,
) -> Result<()> {
    let market = &mut ctx.accounts.market;

    // Time check - can't bet after deadline
    require!(Clock::get()?.unix_timestamp < market.resolution_time,
             BettingClosed);

    // Transfer SOL: user → market PDA
    let transfer_ix = system_instruction::transfer(
        &ctx.accounts.user.key(),
        &ctx.accounts.market.key(),
        amount,
    );
    invoke(/* ... */)?;

    // Update pools
    if bet_yes {
        market.yes_pool = market.yes_pool.checked_add(amount)
            .ok_or(PredictionMarketError::Overflow)?;
    } else {
        market.no_pool = market.no_pool.checked_add(amount)
            .ok_or(PredictionMarketError::Overflow)?;
    }

    // Update user's position...
}
```

**Key Insight:** `checked_add` returns `None` on overflow instead of wrapping. This prevents attacks where someone could overflow the pool to zero.

---

#### Function 3: `claim_winnings` (8 min)

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

```rust
pub fn claim_winnings(ctx: Context<ClaimWinnings>) -> Result<()> {
    let market = &ctx.accounts.market;
    let position = &mut ctx.accounts.position;

    // Guards
    require!(market.resolved, NotResolved);
    require!(!position.claimed, AlreadyClaimed);

    // Determine winning side
    let (user_bet, winning_pool, losing_pool) = match market.outcome {
        Some(true) => (position.yes_amount, market.yes_pool, market.no_pool),
        Some(false) => (position.no_amount, market.no_pool, market.yes_pool),
        None => return err!(NotResolved),
    };

    require!(user_bet > 0, NoWinnings);

    // Calculate payout
    let winnings = user_bet
        .checked_mul(losing_pool).ok_or(Overflow)?
        .checked_div(winning_pool).ok_or(Overflow)?;
    let total_payout = user_bet.checked_add(winnings).ok_or(Overflow)?;

    // Transfer from PDA to user...
    position.claimed = true;
}
```

---

## Part 3: The Codegen Layer (15 min)

### 3.1 — The Pipeline (5 min)

**Diagram: Code Generation Flow**

```
┌─────────────┐      ┌─────────────┐      ┌─────────────┐
│   lib.rs    │      │  IDL.json   │      │ TypeScript  │
│   (Rust)    │─────▶│ (Schema)    │─────▶│  Client     │
└─────────────┘      └─────────────┘      └─────────────┘
     anchor build         codama:js

   Source of           Intermediate        What frontend
   truth               representation      imports
```

### 3.2 — Generated Code Walkthrough (10 min)

**From IDL to TypeScript:**

```json
// IDL snippet
{
  "name": "placeBet",
  "args": [
    { "name": "amount", "type": "u64" },
    { "name": "betYes", "type": "bool" }
  ]
}
```

```typescript
// Generated: placeBet.ts
export async function getPlaceBetInstructionAsync(
  input: PlaceBetAsyncInput
): Promise<PlaceBetInstruction> {
  // Auto-derives PDAs from seeds
  const marketAddress = await findMarketPda(input.creator, input.marketId);
  const positionAddress = await findPositionPda(marketAddress, input.user);

  // Encodes args to bytes
  const data = getPlaceBetInstructionDataEncoder().encode({
    amount: input.amount,
    betYes: input.betYes,
  });

  return { keys: [...], data, programId };
}
```

**Key Insight:** The generated client handles two hard problems: **PDA derivation** (calculating deterministic addresses) and **serialization** (encoding args to bytes). You never write this by hand.

---

## Part 4: Frontend Architecture (20 min)

### 4.1 — Component Hierarchy (5 min)

**Diagram:**

```
App (layout.tsx)
 └─ Providers (Solana client + wallet)
     ├─ HomePage (page.tsx)
     │   ├─ CreateMarketForm
     │   └─ MarketsList
     │       └─ MarketCard (×N)
     │
     └─ ActivityPage (activity/page.tsx)
         └─ PositionsList
             └─ PositionCard (×N)
```

### 4.2 — Data Fetching Pattern (8 min)

**Walkthrough: `markets-list.tsx`**

```typescript
// Fetch all Market accounts from the program
const fetchMarkets = async () => {
  const response = await fetch(RPC_URL, {
    method: 'POST',
    body: JSON.stringify({
      method: 'getProgramAccounts',
      params: [
        PROGRAM_ID,
        {
          filters: [{
            memcmp: {
              offset: 0,
              bytes: "dkokXHR3DTw"  // Market discriminator
            }
          }]
        }
      ]
    })
  });

  // Decode each account
  const markets = accounts.map(acc => {
    const data = base64ToBytes(acc.data);
    return getMarketDecoder().decode(data);
  });
};

// Poll every 3 seconds
useEffect(() => {
  const interval = setInterval(fetchMarkets, 3000);
  return () => clearInterval(interval);
}, []);
```

**Diagram: Discriminator Filtering**

```
Program owns many accounts:
┌──────────────────┐
│ [disc: Market]   │ ← matches filter ✓
│ question: "..."  │
└──────────────────┘
┌──────────────────┐
│ [disc: Position] │ ← doesn't match ✗
│ user: 0x...      │
└──────────────────┘
┌──────────────────┐
│ [disc: Market]   │ ← matches filter ✓
│ question: "..."  │
└──────────────────┘
```

### 4.3 — Transaction Flow (7 min)

**Walkthrough: Betting in `market-card.tsx`**

```typescript
const handleBet = async (betYes: boolean) => {
  // 1. Build instruction using generated client
  const instruction = await getPlaceBetInstructionAsync({
    market: marketAddress,
    user: wallet.address,
    amount: BigInt(solAmount * LAMPORTS_PER_SOL),
    betYes,
  });

  // 2. Send transaction
  await sendTransaction({
    instructions: [instruction],
  });

  // 3. UI updates on next poll (3s)
};
```

**Diagram: Full Round Trip**

```
User clicks          Build            Sign &           Program          Account
"Bet YES"      →   Instruction   →    Send       →   Executes     →    Updated
                        │                               │
                   Generated               Validation + state change
                   client                  (checked_add, time check)
```

---

## Part 5: Security & Trade-offs (5 min)

### Design Decisions

| Choice | Trade-off |
|--------|-----------|
| Creator resolves market | Simple but centralized trust |
| Polling vs WebSockets | Simpler code, slightly delayed updates |
| All-or-nothing bets | No partial positions, simpler math |
| No fees | No protocol revenue, pure market |

### Security Protections

| Attack Vector | Protection |
|---------------|------------|
| Overflow attacks | `checked_add/mul/div` |
| Double claims | `position.claimed` flag |
| Late bets | Time window validation |
| Unauthorized resolution | Creator-only check |

---

## Part 6: Recap (5 min)

### The Full Picture

```
┌─────────────────────────────────────────────────────────────┐
│  1. User interacts with React component                     │
│  2. Component calls generated instruction builder           │
│  3. Wallet signs, transaction sent to Solana                │
│  4. Anchor program validates & mutates account state        │
│  5. Frontend polls for updated state, re-renders            │
└─────────────────────────────────────────────────────────────┘
```

### Key Takeaways

1. **Programs are stateless** — accounts hold all state
2. **PDAs enable trustless escrow** — no private keys hold funds
3. **Codegen eliminates serialization bugs** — type-safe by construction
4. **The IDL is the contract** — between on-chain and off-chain

---

## Timing Summary

| Section | Duration | Content Type |
|---------|----------|--------------|
| Part 1: Why & What | 15 min | Diagrams |
| Part 2: On-Chain Program | 30 min | Code walkthrough |
| Part 3: Codegen Layer | 15 min | Pipeline + code |
| Part 4: Frontend Architecture | 20 min | Diagrams + code |
| Part 5: Security & Trade-offs | 5 min | Discussion |
| Part 6: Recap | 5 min | Summary |
| **Total** | **90 min** | |

---

## File References

| Topic | File Path |
|-------|-----------|
| Account structures | `anchor/programs/prediction_market/src/state.rs` |
| Program instructions | `anchor/programs/prediction_market/src/lib.rs` |
| Error definitions | `anchor/programs/prediction_market/src/errors.rs` |
| Generated client | `app/generated/prediction_market/` |
| Markets list component | `app/components/markets-list.tsx` |
| Market card component | `app/components/market-card.tsx` |
| Wallet providers | `app/components/providers.tsx` |
| Codama config | `codama.json` |
