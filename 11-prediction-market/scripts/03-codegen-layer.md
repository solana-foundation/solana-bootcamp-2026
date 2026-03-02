# Part 3: The Codegen Layer

**Duration:** 15 min

---

## 3.1 — The Pipeline (5 min | ~350 words)

<!-- Diagram: Rust → IDL → TypeScript -->

Alright, Part 3 is all about the codegen layer. This is the bridge between our Rust program and the TypeScript frontend. Without it, you'd be hand-writing PDAs, instruction data, and account decoders. That is slow, easy to mess up, and honestly just not fun.

The big idea is simple: the Rust program is the source of truth, the IDL is the contract, and the generated client is what we actually use in the UI. When the program changes, we regenerate the client so everything stays in sync. No guessing about byte layouts, no copying account order from docs, no silent mismatches.

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

**Commands:**
```bash
npm run anchor-build   # Rust → IDL
npm run codama:js      # IDL → TypeScript
```

`anchor-build` compiles the program and emits an IDL JSON. That file is a schema: instructions, accounts, types, and errors. Then `codama:js` reads the IDL and outputs a TypeScript client with instruction builders, PDA helpers, account decoders, and error types. This all lands in `app/generated/prediction_market/`.

Quick workflow note: any time you change a Rust account, instruction, or error, you should rerun both commands. That keeps the client up to date. Think of the generated code as read-only. You don't hand-edit it; you regenerate it.

If you ever see a weird mismatch in the UI, the first thing to check is whether the IDL and generated client are up to date. The fastest fix is often: rebuild, regenerate, and reload. That eliminates a whole category of "why is this deserializing wrong" bugs.

So the pipeline is short and boring, which is good. It lets you focus on the actual logic instead of byte layouts.

One more note: the IDL is also useful outside of TypeScript. Other teams can use it to generate clients in different languages. That makes your on-chain program more portable without you doing extra work.

If you ever want to sanity-check what the program exposes, open the IDL in `anchor/target/idl/`. You can see the full list of instructions, the account layouts, and the exact field names. It's a great debugging tool when something feels off in the UI.

And honestly, the generated code itself is a great learning resource. You can open any instruction file and see exactly which accounts are required, which ones are writable, and which signer is expected. It's like a living spec.

---

## 3.2 — Generated Code Walkthrough (10 min | ~800 words)

<!-- Side-by-side IDL vs TypeScript -->

Let's look at what the generated client actually gives us. We'll use `place_bet` as the example because it touches a little bit of everything: instruction args, PDAs, and serialization.

**From IDL to TypeScript:**

```json
// IDL snippet (anchor/target/idl/prediction_market.json)
{
  "name": "placeBet",
  "args": [
    { "name": "amount", "type": "u64" },
    { "name": "betYes", "type": "bool" }
  ]
}
```

```typescript
// Generated: app/generated/prediction_market/instructions/placeBet.ts
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

On the left, the IDL just declares the instruction name and args. On the right, Codama turns that into a function we can call from the UI. We pass plain inputs, and it does all the hard stuff for us.

First, it derives PDAs. It knows the seeds for `Market` and `UserPosition` because they are in the IDL, so it can call `findMarketPda` and `findPositionPda` with the same seeds as the program. That means we never have to re-implement that logic in JavaScript.

Second, it serializes args into bytes. Rust uses Borsh under the hood, and the ordering matters. The generated encoder makes sure the bytes match the program exactly. For u64 values you will notice everything is `bigint` in TypeScript, which is a good thing. It keeps you from overflowing a normal JS number.

Third, it builds the account metas in the correct order, with the right writable and signer flags. That is one of the easiest places to make a mistake if you hand-roll instructions. The generated client saves you from that whole class of bugs.

The generated package also includes account decoders. For example, `getMarketDecoder()` knows how to read a `Market` account from raw bytes, including the 8 byte Anchor discriminator. That means you can fetch accounts with `getProgramAccounts`, then decode them with the same layout the program uses. Again, no manual layout math.

You'll also see helpers for errors and types. If the program returns `MarketError::BettingClosed`, the generated client can map that to a typed error on the frontend. That makes it easier to show real error messages instead of "Transaction failed."

Another nice piece is that the generated client exposes both encoders and decoders. That means you can write small tests that encode instruction data and compare it to expected bytes, or decode an account blob you captured from the RPC. It's a lightweight way to verify that your frontend and program are aligned.

So in practice, the generated client turns on-chain changes into frontend changes almost automatically. You update Rust, run the two commands, and the UI compiles against the new types. It's a huge productivity boost and it keeps the codebase honest.

This is why we spend time on the pipeline. It removes an entire layer of fragile glue code and lets you move fast without breaking things.

A small workflow tip: decide whether you want to commit the generated client. For tutorials I usually commit it so anyone can clone and run without extra steps. In a bigger team you might regenerate in CI instead. Either way, treat the generated folder as build output, not hand-written code.

There are a few extra goodies in the generated output that are worth calling out. You'll get the program ID as a constant, which keeps your frontend from drifting to the wrong address. You'll also get typed account interfaces, so if you hover a decoded `Market` object you see the exact fields and their types. That makes it harder to accidentally treat a u64 as a regular number.

The instruction builders usually come in sync and async flavors. The async ones are handy when PDAs are involved, because they need to derive addresses. The sync versions can be useful in tests or in scripts where you already computed the addresses.

You also get account discriminators. Anchor uses the first 8 bytes of an account to identify the type, and the generated client knows those discriminators. That means you can filter program accounts safely and decode only the ones you expect.

One thing people often miss: the client encoders and decoders are pure functions. That means you can test them in isolation. If you want to sanity-check a new field, decode a local fixture or encode a sample and compare it to what the program expects. It's a nice, tight feedback loop.

So when you hear "codegen," don't think of it as a nice-to-have. It's a guardrail. It keeps your on-chain and off-chain layers locked together as you iterate.

**Key Insight:** The generated client handles two hard problems:
1. **PDA derivation** - calculating deterministic addresses from seeds
2. **Serialization** - encoding args to bytes matching Rust's Borsh format

You never write this by hand.
