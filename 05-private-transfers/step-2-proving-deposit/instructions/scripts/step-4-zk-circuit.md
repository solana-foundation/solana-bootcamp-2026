# Step 4: The ZK Circuit - Script

Now we're onto the fun part - zero knowledge proofs! We can deposit privately, prove membership and prevent double-spending. But right now if we just send a Merkle proof we will be revealing which commitment is ours!

This is because when you're submitting a Merkle proof, you're basically saying "here's my commitment at leaf position 5, and here are the sibling hashes that prove it's in the tree." The proof contains your actual commitment, the path of sibling hashes, and the index position. Anyone watching the blockchain can see exactly which commitment you're claiming, look back at when that commitment was added during deposit, and link your deposit wallet to your withdrawal wallet.


![zk_circuit](../assets/zk_circuit.png)

---

We prove: "I know a nullifier, secret, and amount. The commitment is in the Merkle tree. The nullifier hash is correct."

The verifier on Solana is convinced that we have a valid deposit. But they learn nothing about which deposit.

---


We can write these circuits in a language called Noir. It allows someone to generate proofs client-side or on a backend server, and then the user can send that proof to be verified on Solana. The reason we want to use Noir is that syntax-wise it's very familiar to our favourite language Rust, and it also allows us to use whichever proving system we want. This means we can choose Groth16! Groth16 is especially great for us because Solana needs small proofs and fast verification.

---

## Public vs Private Inputs

**Public** (visible on-chain):
- Merkle root
- Nullifier hash
- Recipient address
- Amount

**Private** (never revealed):
- Nullifier
- Secret
- Merkle proof path
- Path directions

---

## What We'll Do

In this step we'll:

1. Look at the full withdrawal circuit
2. Install Nargo (Noir compiler)
4. Compile the circuit
5. Generate proving and verification keys

---

