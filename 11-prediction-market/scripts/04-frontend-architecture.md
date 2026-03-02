# Part 4: Frontend Architecture

**Duration:** 20 min

---

## 4.1 — Component Hierarchy (5 min | ~300 words)

<!-- Tree diagram - mostly visual -->

Alright, let's zoom out and look at the frontend structure. We're using the Next.js App Router, so the top-level layout wraps everything in providers. That keeps the Solana connection and wallet state available across the app.

The mental model is simple: a layout with a Providers wrapper, then two pages. The home page is where you create and browse markets. The activity page is where you see your past positions. Everything else hangs off those two roots.

Keeping the tree shallow helps a lot. It makes it clear where data flows and where side effects live. You'll see that anything wallet-related or RPC-related lives inside `Providers`, while everything else is just UI components.

**Diagram: Component Tree**

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

This is not fancy, and that's the point. A small tree keeps the tutorial focused and makes it easy to find where a given behavior lives. If you ever need to add more pages later, you already have a clean starting point.

One practical detail: `Providers` is a client component. Wallet adapters depend on `window`, so you keep that boundary clear. Everything under it can be client-side, while the layout itself can stay server-rendered. That gives you the nice Next.js defaults without fighting the wallet.

On the home page, `CreateMarketForm` is just a form with a question input and a resolution time picker. It calls the `create_market` instruction and then resets the UI. `MarketsList` is the read side: it fetches markets and renders a `MarketCard` for each one.

On the activity page, `PositionsList` does the same thing but for user positions. Each `PositionCard` can show the bet amount, the outcome, and whether the position has been claimed. That makes the flow feel complete without adding a bunch of extra complexity.

`Providers` is also where we set the cluster and connection. In this repo it's devnet, but it could be mainnet, localnet, or a custom RPC. You typically wire this up in `app/components/providers.tsx` and keep it centralized so every component gets the same connection and wallet context.

We also keep state local to the components that need it. `MarketCard` gets the decoded market plus a couple of callbacks. No global store, no heavy state management. For a tutorial, that keeps the mental load low.

---

## 4.2 — Data Fetching Pattern (8 min | ~700 words)

<!-- Code walkthrough of markets-list.tsx -->

For data fetching, we keep it plain and predictable. We don't use an indexer or a database layer in this tutorial. The frontend talks straight to the RPC, pulls raw accounts, and decodes them with the generated client.

The main place this happens is `markets-list.tsx`. It calls `getProgramAccounts`, filters by the Market discriminator, and then decodes each account into a usable object. That's it.

**Walkthrough: `app/components/markets-list.tsx`**

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

So what's going on here? `getProgramAccounts` returns every account owned by the program. The memcmp filter is how we narrow it down. Anchor prepends an 8 byte discriminator to each account type, so by filtering on that discriminator we only get `Market` accounts and skip `UserPosition` accounts.

Once we have the raw account data, we decode it with `getMarketDecoder()`. This comes from the codegen layer, which means the layout matches the Rust struct exactly. We do not manually parse bytes, and we do not risk subtle bugs.

In practice, you'd wrap this in a try/catch and set a loading state. If the RPC call fails or the decoder throws, you can show a simple "Retry" button. In a tutorial we can keep it light, but in a real app you'd want some guardrails for flaky RPCs.

The polling every 3 seconds is a conscious trade-off. It's simple and reliable, but it is not real-time. For production you might switch to WebSockets or an indexer, but for a tutorial the polling approach keeps the code short and easy to follow.

One nice side effect of decoding on the client is that we can compute derived fields on the fly. For example, we can compute implied probability from `yes_pool` and `no_pool`, or show total liquidity as `yes_pool + no_pool`. None of that needs to live on-chain.

If you want to add caching, React Query or SWR is a natural next step. But again, for a tutorial we keep it minimal and readable.

Also notice the cleanup in `useEffect`. That matters. If you navigate away from the page, you don't want multiple intervals stacking up. A simple `clearInterval` keeps things safe.

In a bigger app, you might pull the fetching into a custom hook like `useMarkets` and share it across pages. But for this tutorial, keeping the logic close to the component makes it easier to follow.

On the activity page, the pattern is similar but for `UserPosition` accounts. You can either fetch all positions and filter on the client, or add a memcmp filter for the user pubkey. The offset is a little trickier because the account has a discriminator and a market pubkey first, but once you compute it you can filter efficiently.

You can also sort and group the markets on the client. For example, you can split active vs resolved by comparing `resolution_time` to `Date.now() / 1000`. That keeps the UI clean without adding extra on-chain fields.

If the number of markets grows, `getProgramAccounts` will start to feel heavy. At that point you'd switch to an indexer or at least cache the results. But for a workshop-size demo, it's totally fine.

One more small detail: pools are stored as lamports, so the UI should convert to SOL when displaying values. It's just `lamports / LAMPORTS_PER_SOL`, but keeping that conversion centralized avoids off-by-ones and makes the UI consistent.

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

This little filter is doing a lot of work. It keeps the RPC payload smaller and keeps the decoder logic focused on a single account type.

---

## 4.3 — Transaction Flow (7 min | ~500 words)

<!-- Code + round-trip diagram -->

Now let's look at the happy path for a bet. The pattern is the same for create, resolve, and claim: build the instruction with the generated client, send it with the wallet, and then let the UI update on the next poll.

**Walkthrough: Betting in `app/components/market-card.tsx`**

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

The key thing here is that the instruction builder is fully typed. If you pass the wrong type for `amount` or forget a required account, TypeScript tells you immediately. That saves a ton of time.

After `sendTransaction`, you can show a toast, set a local "pending" flag, or optimistically update the UI. In this tutorial we keep it simple and rely on polling to refresh. That keeps the UI consistent with on-chain state without building a bunch of extra state management.

Also note the `BigInt` conversion. u64 values in Rust map to `bigint` in TypeScript, so we always convert `solAmount` to lamports and then to `BigInt`. This avoids precision issues with large numbers.

If you want stronger UX, you can wait for confirmation. Wallet adapters usually give you the signature right away, then you can call `connection.confirmTransaction(signature)` and only clear the pending state once it lands. That gives you a more accurate loading indicator.

The same pattern applies to the other instructions. `create_market` builds a different instruction with a question and resolution time, `resolve_market` only needs the creator to sign, and `claim_winnings` uses the user's position PDA. Once you learn the flow for one instruction, the rest feel familiar.

In practice, you'll also want to wrap the handler in a try/catch. If the wallet rejects the signature or the program throws, you can surface a friendly error. The generated client already gives you typed errors, so you can map them to messages like "Betting is closed" instead of a generic failure.

For `create_market`, the main UI work is converting the user's date input into a unix timestamp. A simple `Math.floor(date.getTime() / 1000)` keeps it aligned with the program's `resolution_time`. It's a small detail, but if you get it wrong you will hit the "ResolutionTimeInPast" error immediately.

You can also add small UX touches like disabling the bet buttons while a transaction is pending, or showing the current pool sizes on the card. Those don't change the architecture, but they make the app feel much smoother.

If you run into "Transaction too large" or "Blockhash not found" during testing, it's usually a devnet or wallet timing issue. Retrying with a fresh blockhash fixes it. For heavier programs you might add a compute budget instruction, but this program is tiny so it isn't needed.

Another pattern you can use is optimistic UI. You can temporarily update the pool totals in the card, then reconcile on the next poll. It's optional, but it makes the UI feel snappy even if the RPC is slow.

**Diagram: Full Round Trip**

```
User clicks          Build            Sign &           Program          Account
"Bet YES"      →   Instruction   →    Send       →   Executes     →    Updated
                        │                               │
                   Generated               Validation + state change
                   client                  (checked_add, time check)
```

This flow repeats across the app. Once you learn it for `place_bet`, everything else is just a variation on the same pattern.
