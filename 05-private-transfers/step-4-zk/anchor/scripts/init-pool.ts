/**
 * Initialize the pool on devnet (or reinitialize if needed)
 *
 * Run with: npx ts-node scripts/init-pool.ts
 */

import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PrivateTransfers } from "../target/types/private_transfers";
import { PublicKey, SystemProgram } from "@solana/web3.js";

async function main() {
  // Configure the client
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.PrivateTransfers as Program<PrivateTransfers>;

  console.log("Program ID:", program.programId.toString());
  console.log("Wallet:", provider.wallet.publicKey.toString());

  // Find PDAs
  const [poolPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("pool")],
    program.programId
  );
  const [poolVaultPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("vault"), poolPda.toBuffer()],
    program.programId
  );
  const [nullifierSetPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("nullifiers"), poolPda.toBuffer()],
    program.programId
  );

  console.log("\nPDAs:");
  console.log("  Pool:", poolPda.toString());
  console.log("  Vault:", poolVaultPda.toString());
  console.log("  Nullifier Set:", nullifierSetPda.toString());

  // Check if pool already exists
  const poolAccount = await provider.connection.getAccountInfo(poolPda);

  if (poolAccount) {
    console.log("\nPool already exists!");

    // Fetch and display current state
    const pool = await program.account.pool.fetch(poolPda);
    console.log("  Authority:", pool.authority.toString());
    console.log("  Next leaf index:", pool.nextLeafIndex.toString());
    console.log("  Total deposits:", pool.totalDeposits.toString());
    console.log("  Current root index:", pool.currentRootIndex.toString());

    console.log("\nTo reset, you need to:");
    console.log("1. Use localnet with --reset flag, OR");
    console.log("2. Deploy a new program with different ID");
    return;
  }

  // Initialize the pool
  console.log("\nInitializing pool...");

  try {
    const tx = await program.methods
      .initialize()
      .accounts({
        pool: poolPda,
        nullifierSet: nullifierSetPda,
        poolVault: poolVaultPda,
        authority: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log("Pool initialized! Tx:", tx);

    // Verify
    const pool = await program.account.pool.fetch(poolPda);
    console.log("\nPool state:");
    console.log("  Authority:", pool.authority.toString());
    console.log("  Next leaf index:", pool.nextLeafIndex.toString());
    console.log("  Total deposits:", pool.totalDeposits.toString());
  } catch (e) {
    console.error("Error initializing pool:", e);
  }
}

main().catch(console.error);
