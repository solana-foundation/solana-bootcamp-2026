import * as anchor from "@anchor-lang/core";
import { BN, Program } from "@anchor-lang/core";
import { PublicKey } from "@solana/web3.js";
import { Voting } from "../target/types/voting";

const PROGRAM_ID = new PublicKey("65KHV8cXwJ8apTKMqnpSdhdHkHhRySatgKMwnxm6C3gG");

describe("voting", () => {
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Voting as Program<Voting>;

  const POLL_ID = new BN(1);

  const [pollAddress] = PublicKey.findProgramAddressSync(
    [Buffer.from("poll"), POLL_ID.toArrayLike(Buffer, "le", 8)],
    PROGRAM_ID
  );

  it("initializes a poll", async () => {
    await program.methods
      .initializePoll(
        POLL_ID,
        new BN(0),
        new BN(1893456000), // 2030-01-01
        "Test Poll",
        "A poll to test the voting program"
      )
      .rpc();

    const pollAccount = await program.account.pollAccount.fetch(pollAddress);
    console.log("Poll account:", pollAccount);

    expect(pollAccount.pollName).toEqual("Test Poll");
    expect(pollAccount.pollDescription).toEqual("A poll to test the voting program");
    expect(pollAccount.pollVotingStart.toNumber()).toEqual(0);
    expect(pollAccount.pollVotingEnd.toNumber()).toEqual(1893456000);
    expect(pollAccount.pollOptionIndex.toNumber()).toEqual(0);
  });

  it("initializes candidates", async () => {
    await program.methods
      .initializeCandidate(POLL_ID, "Alice")
      .rpc();

    await program.methods
      .initializeCandidate(POLL_ID, "Bob")
      .rpc();

    const pollAccount = await program.account.pollAccount.fetch(pollAddress);
    expect(pollAccount.pollOptionIndex.toNumber()).toEqual(2);
  });

  it("casts a vote", async () => {
    const [aliceAddress] = PublicKey.findProgramAddressSync(
      [POLL_ID.toArrayLike(Buffer, "le", 8), Buffer.from("Alice")],
      PROGRAM_ID
    );

    await program.methods
      .vote(POLL_ID, "Alice")
      .rpc();

    const aliceAccount = await program.account.candidateAccount.fetch(aliceAddress);
    console.log("Alice account:", aliceAccount);
    expect(aliceAccount.candidateVotes.toNumber()).toEqual(1);
  });
});
