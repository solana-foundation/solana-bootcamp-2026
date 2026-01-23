# Part 5: Security & Trade-offs

**Duration:** 5 min

---

<!-- Tables + discussion - moderate pace -->

Alright, quick but important section. This is a tutorial build, so we made a bunch of choices that keep the code simple. Some of those choices trade off decentralization or product features. That's fine for learning, but you should know exactly where the shortcuts are.

We'll go through the main design decisions, the protections we did add, and the obvious upgrades you would make for a real product.

## Design Decisions

| Choice | Trade-off |
|--------|-----------|
| Creator resolves market | Simple but centralized trust |
| Polling vs WebSockets | Simpler code, slightly delayed updates |
| All-or-nothing bets | No partial positions, simpler math |
| No fees | No protocol revenue, pure market |

The biggest one is resolution. In our build, the creator decides the outcome. That keeps the program tiny and avoids oracle integration, but it introduces trust. In production, you'd want an oracle or multi-sig so no single party can manipulate results.

Polling is another trade-off. It's easy to implement and easy to understand, but it adds latency and extra RPC load. If you want real-time UI, you'd use WebSockets or an indexer.

All-or-nothing bets keep the math clean. We don't do partial exits, cash-out, or liquidity. That makes the program easier to reason about, but it limits what traders can do.

And finally, we skipped fees. That keeps the math pure and the example clear, but it also means there is no revenue or sustainability built in. In production, you'd likely add a small fee on bets or on winnings.

## Security Protections

| Attack Vector | Protection |
|---------------|------------|
| Overflow attacks | `checked_add/mul/div` |
| Double claims | `position.claimed` flag |
| Late bets | Time window validation |
| Unauthorized resolution | Creator-only check |

These are the minimum protections you want in any prediction market.

The arithmetic checks are there to prevent overflow attacks. In a parimutuel system, overflowing a pool can change implied prices or even allow free bets. Using checked math is boring but critical.

We also gate claims with `position.claimed`. That makes claims idempotent and prevents double spending. The resolution time check prevents late bets, and the creator check prevents random signers from resolving the market.

Notice what's missing: we do not prevent a creator from resolving early with the "wrong" outcome, and we don't prevent griefing with spam markets. Those are product and governance problems more than pure code problems, but they matter.

We also don't do anything about front-running or transaction ordering. On Solana, that usually shows up as users racing to bet near the deadline. For a tutorial it's fine, but if this were production you'd think about more explicit cutoff logic and maybe an on-chain "finalization" window.

## What Could Be Added

- Decentralized oracles (Switchboard, Pyth) for trustless resolution
- Fee mechanism for protocol sustainability
- Partial position exits / liquidity
- Multi-outcome markets (not just YES/NO)

If you wanted to keep going, oracles are the first big upgrade. You'd also add some kind of fee or treasury to sustain the system. Partial exits and liquidity are a bigger design shift, but they make the market feel more like a trading product.

So the takeaway is simple: the core is solid, but this is a learning-focused version. If you treat it like production, you will want to harden it. That's normal.

Even a tiny fee changes the payout math, so it deserves careful thought.
