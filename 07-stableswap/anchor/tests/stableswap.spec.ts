/**
 * StableSwap AMM — Integration Tests
 *
 * Tests the full lifecycle of a two-token stableswap pool:
 *   1. Initialize a pool (USDC/USDT analogue)
 *   2. Add liquidity
 *   3. Swap A→B and B→A
 *   4. Remove liquidity
 *   5. Slippage protection
 *
 * Account resolution in @anchor-lang/core 1.0.0-rc.2:
 *   - AUTO-resolved: accounts with fixed addresses (tokenProgram, systemProgram),
 *     PDA accounts whose seeds can be computed, and ATAs.
 *   - PASS manually: user-specific accounts and non-derivable mints/vaults.
 */

import * as anchor from "@anchor-lang/core";
import { BN, Program } from "@anchor-lang/core";
import { Keypair, PublicKey } from "@solana/web3.js";
import {
  createMint,
  createAssociatedTokenAccount,
  mintTo,
  getAccount,
  getAssociatedTokenAddress,
} from "@solana/spl-token";
import { Stableswap } from "../target/types/stableswap";

// ─── Constants ────────────────────────────────────────────────────────────────

const PROGRAM_ID = new PublicKey("CorabfeniSyoc4aLcJe7t9b3RaFX5tzVWXdewU1xuA6B");

// 6-decimal token amounts (like USDC/USDT)
const ONE_TOKEN = 1_000_000; // 1.0
const MILLION = 1_000_000_000_000; // 1,000,000.0

const AMPLIFICATION = 100; // Standard for stablecoin pairs
const FEE_BPS = 4; // 0.04%

// ─── Test Suite ───────────────────────────────────────────────────────────────

describe("stableswap", () => {
  anchor.setProvider(anchor.AnchorProvider.env());

  const provider = anchor.getProvider() as anchor.AnchorProvider;
  const program = anchor.workspace.Stableswap as Program<Stableswap>;
  const payer = (provider.wallet as anchor.Wallet).payer;

  let mintA: PublicKey;
  let mintB: PublicKey;
  let lpMint: Keypair;

  // Pool PDA (for off-chain fetch)
  let poolPda: PublicKey;
  let poolBump: number;

  // Pool vault ATAs owned by pool PDA
  let vaultA: PublicKey;
  let vaultB: PublicKey;

  // User token accounts
  let userTokenA: PublicKey;
  let userTokenB: PublicKey;
  let userLpToken: PublicKey;

  // ─── Setup ─────────────────────────────────────────────────────────────────

  beforeAll(async () => {
    // Create two stablecoin-like mints (6 decimals, like USDC/USDT)
    mintA = await createMint(provider.connection, payer, payer.publicKey, null, 6);
    mintB = await createMint(provider.connection, payer, payer.publicKey, null, 6);
    lpMint = Keypair.generate();

    // Derive pool PDA for off-chain use (e.g., fetching pool state)
    [poolPda, poolBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("pool"), mintA.toBuffer(), mintB.toBuffer()],
      PROGRAM_ID
    );

    // Pool vault ATAs owned by pool PDA
    vaultA = await getAssociatedTokenAddress(mintA, poolPda, true);
    vaultB = await getAssociatedTokenAddress(mintB, poolPda, true);

    // User token accounts
    userTokenA = await createAssociatedTokenAccount(provider.connection, payer, mintA, payer.publicKey);
    userTokenB = await createAssociatedTokenAccount(provider.connection, payer, mintB, payer.publicKey);

    // Fund user with 2M of each token
    await mintTo(provider.connection, payer, mintA, userTokenA, payer, 2 * MILLION);
    await mintTo(provider.connection, payer, mintB, userTokenB, payer, 2 * MILLION);

    console.log("Setup complete:");
    console.log("  mintA  :", mintA.toBase58());
    console.log("  mintB  :", mintB.toBase58());
    console.log("  pool   :", poolPda.toBase58());
  });

  // ─── Test 1: Initialize Pool ───────────────────────────────────────────────

  it("initializes a two-token stableswap pool", async () => {
    // initializePool manually-passed accounts: admin, tokenMintA, tokenMintB, lpMint
    // Auto-resolved: pool (PDA), vaultA, vaultB, systemProgram, tokenProgram,
    //                associatedTokenProgram, rent
    await program.methods
      .initializePool(new BN(AMPLIFICATION), FEE_BPS)
      .accounts({
        admin: payer.publicKey,
        tokenMintA: mintA,
        tokenMintB: mintB,
        lpMint: lpMint.publicKey,
      })
      .signers([lpMint])
      .rpc();

    const pool = await program.account.pool.fetch(poolPda);

    expect(pool.admin.toBase58()).toEqual(payer.publicKey.toBase58());
    expect(pool.tokenMintA.toBase58()).toEqual(mintA.toBase58());
    expect(pool.tokenMintB.toBase58()).toEqual(mintB.toBase58());
    expect(pool.lpMint.toBase58()).toEqual(lpMint.publicKey.toBase58());
    expect(pool.vaultA.toBase58()).toEqual(vaultA.toBase58());
    expect(pool.vaultB.toBase58()).toEqual(vaultB.toBase58());
    expect(pool.amplification.toNumber()).toEqual(AMPLIFICATION);
    expect(pool.feeBps).toEqual(FEE_BPS);
    expect(pool.bump).toEqual(poolBump);

    console.log("Pool initialized: A=%d, fee=%dbps", AMPLIFICATION, FEE_BPS);
  });

  // ─── Test 2: Add Initial Liquidity ─────────────────────────────────────────

  it("adds initial liquidity and mints LP tokens", async () => {
    userLpToken = await createAssociatedTokenAccount(
      provider.connection, payer, lpMint.publicKey, payer.publicKey
    );

    const depositA = MILLION;
    const depositB = MILLION;

    // addLiquidity manually-passed accounts: tokenMintA, tokenMintB, vaultA, vaultB,
    //   lpMint, userTokenA, userTokenB, userLpToken, user
    // Auto-resolved: pool (PDA), tokenProgram
    await program.methods
      .addLiquidity(new BN(depositA), new BN(depositB), new BN(0))
      .accounts({
        tokenMintA: mintA,
        tokenMintB: mintB,
        vaultA,
        vaultB,
        lpMint: lpMint.publicKey,
        userTokenA,
        userTokenB,
        userLpToken,
        user: payer.publicKey,
      })
      .rpc();

    const vaultAInfo = await getAccount(provider.connection, vaultA);
    const vaultBInfo = await getAccount(provider.connection, vaultB);
    expect(Number(vaultAInfo.amount)).toEqual(depositA);
    expect(Number(vaultBInfo.amount)).toEqual(depositB);

    const lpInfo = await getAccount(provider.connection, userLpToken);
    const lpMinted = Number(lpInfo.amount);
    // D ≈ 2M for balanced pool; we receive D - MINIMUM_LIQUIDITY(1000) LP tokens
    expect(lpMinted).toBeGreaterThan(1_999_000_000_000);

    console.log("Initial liquidity: %d A, %d B → %d LP", depositA, depositB, lpMinted);
  });

  // ─── Test 3: Add More Liquidity ────────────────────────────────────────────

  it("adds proportional subsequent liquidity", async () => {
    const lpBefore = await getAccount(provider.connection, userLpToken);
    const lpSupplyBefore = Number(lpBefore.amount);

    const depositA = 100_000 * ONE_TOKEN; // +10% of pool
    const depositB = 100_000 * ONE_TOKEN;

    await program.methods
      .addLiquidity(new BN(depositA), new BN(depositB), new BN(0))
      .accounts({
        tokenMintA: mintA,
        tokenMintB: mintB,
        vaultA,
        vaultB,
        lpMint: lpMint.publicKey,
        userTokenA,
        userTokenB,
        userLpToken,
        user: payer.publicKey,
      })
      .rpc();

    const lpAfter = await getAccount(provider.connection, userLpToken);
    const lpMinted = Number(lpAfter.amount) - lpSupplyBefore;
    const ratio = lpMinted / lpSupplyBefore;

    console.log("  Additional LP ratio:", ratio.toFixed(4));
    expect(ratio).toBeGreaterThan(0.09);
    expect(ratio).toBeLessThan(0.11);
  });

  // ─── Test 4: Swap A → B ────────────────────────────────────────────────────

  it("swaps A→B with very low slippage (stablecoin advantage)", async () => {
    const swapAmount = 1_000 * ONE_TOKEN; // 1000 USDC

    const userBBefore = await getAccount(provider.connection, userTokenB);

    // swap manually-passed: tokenMintA, tokenMintB, vaultA, vaultB,
    //   userInput, userOutput, user
    // Auto-resolved: pool (PDA), tokenProgram
    await program.methods
      .swap(new BN(swapAmount), new BN(0), true /* a_to_b */)
      .accounts({
        tokenMintA: mintA,
        tokenMintB: mintB,
        vaultA,
        vaultB,
        userInput: userTokenA,
        userOutput: userTokenB,
        user: payer.publicKey,
      })
      .rpc();

    const userBAfter = await getAccount(provider.connection, userTokenB);
    const received = Number(userBAfter.amount) - Number(userBBefore.amount);

    // With A=100 and fee=0.04%, expect >99.5% of input returned
    const slippage = 1 - received / swapAmount;
    console.log("  A→B swap: %d in, %d out, slippage=%.4f%%", swapAmount, received, slippage * 100);
    expect(received).toBeGreaterThan(swapAmount * 0.995);
    expect(received).toBeLessThan(swapAmount);
  });

  // ─── Test 5: Swap B → A ────────────────────────────────────────────────────

  it("swaps B→A with very low slippage", async () => {
    const swapAmount = 500 * ONE_TOKEN;

    const userABefore = await getAccount(provider.connection, userTokenA);

    await program.methods
      .swap(new BN(swapAmount), new BN(0), false /* b_to_a */)
      .accounts({
        tokenMintA: mintA,
        tokenMintB: mintB,
        vaultA,
        vaultB,
        userInput: userTokenB,
        userOutput: userTokenA,
        user: payer.publicKey,
      })
      .rpc();

    const userAAfter = await getAccount(provider.connection, userTokenA);
    const received = Number(userAAfter.amount) - Number(userABefore.amount);

    console.log("  B→A swap: %d in, %d out", swapAmount, received);
    expect(received).toBeGreaterThan(swapAmount * 0.995);
    expect(received).toBeLessThan(swapAmount);
  });

  // ─── Test 6: Slippage Guard on Swap ───────────────────────────────────────

  it("rejects a swap that does not meet minimum output", async () => {
    const swapAmount = ONE_TOKEN;

    // Demand 100% of input out — impossible since fee > 0
    await expect(
      program.methods
        .swap(new BN(swapAmount), new BN(swapAmount), true)
        .accounts({
          tokenMintA: mintA,
          tokenMintB: mintB,
          vaultA,
          vaultB,
          userInput: userTokenA,
          userOutput: userTokenB,
          user: payer.publicKey,
        })
        .rpc()
    ).rejects.toThrow();
  });

  // ─── Test 7: Remove Liquidity ──────────────────────────────────────────────

  it("removes liquidity proportionally and receives both tokens", async () => {
    const lpAccount = await getAccount(provider.connection, userLpToken);
    const lpBalance = Number(lpAccount.amount);
    const lpToRemove = Math.floor(lpBalance / 2);

    const userABefore = await getAccount(provider.connection, userTokenA);
    const userBBefore = await getAccount(provider.connection, userTokenB);

    // removeLiquidity manually-passed: tokenMintA, tokenMintB, vaultA, vaultB,
    //   lpMint, userTokenA, userTokenB, userLpToken, user
    // Auto-resolved: pool (PDA), tokenProgram
    await program.methods
      .removeLiquidity(new BN(lpToRemove), new BN(0), new BN(0))
      .accounts({
        tokenMintA: mintA,
        tokenMintB: mintB,
        vaultA,
        vaultB,
        lpMint: lpMint.publicKey,
        userTokenA,
        userTokenB,
        userLpToken,
        user: payer.publicKey,
      })
      .rpc();

    const userAAfter = await getAccount(provider.connection, userTokenA);
    const userBAfter = await getAccount(provider.connection, userTokenB);
    const receivedA = Number(userAAfter.amount) - Number(userABefore.amount);
    const receivedB = Number(userBAfter.amount) - Number(userBBefore.amount);

    console.log("  Burned %d LP → %d A, %d B", lpToRemove, receivedA, receivedB);

    expect(receivedA).toBeGreaterThan(0);
    expect(receivedB).toBeGreaterThan(0);

    // LP supply should decrease by the burned amount
    const lpAfter = await getAccount(provider.connection, userLpToken);
    expect(Number(lpAfter.amount)).toEqual(lpBalance - lpToRemove);
  });

  // ─── Test 8: Remove Liquidity Slippage Guard ───────────────────────────────

  it("rejects liquidity removal if output below minimum", async () => {
    const lpAccount = await getAccount(provider.connection, userLpToken);
    const lpToRemove = Math.floor(Number(lpAccount.amount) / 10);

    await expect(
      program.methods
        .removeLiquidity(
          new BN(lpToRemove),
          new BN(MILLION * 1000), // Impossibly large min_a
          new BN(0)
        )
        .accounts({
          tokenMintA: mintA,
          tokenMintB: mintB,
          vaultA,
          vaultB,
          lpMint: lpMint.publicKey,
          userTokenA,
          userTokenB,
          userLpToken,
          user: payer.publicKey,
        })
        .rpc()
    ).rejects.toThrow();
  });

  // ─── Test 9: StableSwap vs Constant Product Comparison ────────────────────

  it("demonstrates significantly lower slippage than constant-product AMM", async () => {
    const vaultAInfo = await getAccount(provider.connection, vaultA);
    const reserveA = Number(vaultAInfo.amount);
    const swapAmount = Math.floor(reserveA * 0.1); // 10% of pool

    const userBBefore = await getAccount(provider.connection, userTokenB);

    await program.methods
      .swap(new BN(swapAmount), new BN(0), true)
      .accounts({
        tokenMintA: mintA,
        tokenMintB: mintB,
        vaultA,
        vaultB,
        userInput: userTokenA,
        userOutput: userTokenB,
        user: payer.publicKey,
      })
      .rpc();

    const userBAfter = await getAccount(provider.connection, userTokenB);
    const received = Number(userBAfter.amount) - Number(userBBefore.amount);
    const efficiency = received / swapAmount;

    // StableSwap (A=100): should give >98% efficiency for 10% of pool swap
    // Constant product: would give ~90.9% for the same trade
    console.log(
      "  10%% pool swap: in=%d out=%d efficiency=%.2f%%",
      swapAmount, received, efficiency * 100
    );
    expect(efficiency).toBeGreaterThan(0.98); // Stableswap advantage!
  });
});
