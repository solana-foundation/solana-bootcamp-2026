**~1.5 min**

# Step 4.3: On-chain Verification - Script


We have our ZK proof. Now we verify it on Solana. This is the final piece in the puzzle of zero knowledge proofs. 

![cpi](../assets/cpi.png)

---

The way it works is through using a tool called Sunspot. Sunspot allows us to generate a separate Solana program from your verification key. This verifier program has the verification logic baked in - you can have a look at it if you like but it's a lot of Groth16 math that we don't need to understand.

Then our program calls this verifier via CPI. If the proof is invalid, the whole transaction fails atomically.

---


In this step we'll:

1. Install Sunspot
1. Generate the Solana verifier program with Sunspot
2. Deploy it to devnet
3. Add the verifier program ID to our code
4. Add the CPI call to verify proofs


