**~2 min**

# Step 3: Preventing Double-Spend - Script

So we can deposit into the pool without storing us as the depositor, then prove a commitment exists in order to withdraw. But what stops someone from proving the same deposit multiple times, or proving we know some other commitment that isn't yours? Nothing yet!

That's where nullifiers come in, and that's what we're going to talk about in this step. The nullifier solves both problems: it proves you own a commitment (because only you know the nullifier you used when creating it), and it prevents double spending (because we can track which nullifiers have been used).

The problem is that we can't just mark "commitment X was spent" when we do a withdraw, because then it links commitment X with the withdrawal. We need to track "something was spent" without revealing which commitment.

---

Remember when we deposited, we generated a random nullifier and included it in our commitment.

When you withdraw, you have to send the hash of that nullifier, the nullifier hash. Our SOlana program needs to store all the used nullifier hashes, so that if you try to withdraw twice, you'd submit the same hash twice, and our program can reject it.

![using_nullifier](../assets/using_nullifier.png)

---

It isn't possible to link a nullifier with the commitment. The commitment was a hash of the nullifier, a secret, and an amount. The nullifier hash is just a hash of the nullifier. They have completely different outputs and we can't reverse one or derive one from the other. Observers will see a deposit with some sort of commitment hash, then a withdrawal with some sort of nullifier hash, and no connection.

---

This step is pretty simple. We'll update our program to

1. Create a NullifierSet account to store used nullifier hashes
2. Check the nullifier hasn't been used during withdrawal
3. Mark nullifiers as used after successful withdrawal
4. Update the WithdrawEvent to include the nullifier hash

![nullifier_set_PDA](../assets/nullifier_set_PDA.png)

