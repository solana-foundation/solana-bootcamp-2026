import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Vault } from "../target/types/vault";

describe("Initialize Labubu Collection", () => {
  // Configure Anchor provider
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Vault as Program<Vault>;

  // Derive collection PDA
  const [collectionPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("collection")],
    program.programId
  );

  it("Initialize Collection", async () => {
    console.log("🚀 Initializing Labubu Collection...");
    console.log("Program ID:", program.programId.toString());
    console.log("Collection PDA:", collectionPda.toString());

    try {
      const tx = await program.methods
        .initializeCollection()
        .accounts({
          authority: provider.wallet.publicKey,
          collection: collectionPda,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();

      console.log("✅ Collection initialized successfully!");
      console.log("Transaction signature:", tx);

      // Read collection account to verify
      const collectionAccount = await program.account.labubuCollection.fetch(
        collectionPda
      );
      console.log("Total supply:", collectionAccount.remainingSupply);
      console.log("Total minted:", collectionAccount.totalMinted);
    } catch (error) {
      console.error("❌ Initialization failed:", error);
      throw error;
    }
  });

  it("Create Mints for 11 Labubu types", async () => {
    console.log("🎨 Creating 11 Labubu Mints...");

    const labubuNames = [
      "Zone Out",
      "Ab Roller",
      "Confident",
      "Show Off",
      "Stretch Out",
      "Sweating",
      "Sleeping",
      "Little Bird",
      "Americano",
      "Lay Down",
      "Secret Edition ⭐",
    ];

    for (let i = 1; i <= 11; i++) {
      const [mintPda] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("labubu_mint"), Buffer.from([i])],
        program.programId
      );

      console.log(`  Creating Labubu #${i} (${labubuNames[i - 1]})...`);

      try {
        const tx = await program.methods
          .createLabubuMint(i)
          .accounts({
            authority: provider.wallet.publicKey,
            collection: collectionPda,
            mint: mintPda,
            tokenProgram: anchor.web3.SYSTEM_PROGRAM_ID, // Token2022 program ID
            systemProgram: anchor.web3.SystemProgram.programId,
            rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          })
          .rpc();

        console.log(`  ✅ Mint #${i} created successfully: ${mintPda.toString()}`);
      } catch (error) {
        console.error(`  ❌ Mint #${i} creation failed:`, error);
        throw error;
      }
    }

    console.log("🎉 All Mints created successfully!");
  });
});
