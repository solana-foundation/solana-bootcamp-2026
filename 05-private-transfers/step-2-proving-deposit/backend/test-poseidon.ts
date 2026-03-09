import { poseidon2Hash } from "@zkpassport/poseidon2";

// Test values from Noir circuit
const nullifier = 12345n;
const secret = 67890n;
const amount = 1000000000n;

// Expected outputs from Noir (using noir-lang/poseidon Poseidon2)
const expectedCommitment = "0x1ab3f3a0210349137477c453a284d34ed76e600e4d4645fe0f794041cfeafec5";
const expectedNullifierHash = "0x1fed07ad686a727dfc33b91206d526e61f519dca9c5054ae729231c201717633";

console.log("Testing @zkpassport/poseidon2 against Noir output (noir-lang/poseidon):");
console.log("");

// Test hash_3 equivalent (commitment = hash(nullifier, secret, amount))
const commitment = poseidon2Hash([nullifier, secret, amount]);
const commitmentHex = "0x" + commitment.toString(16).padStart(64, "0");

// Test hash_1 equivalent (nullifier_hash = hash(nullifier))
const nullifierHash = poseidon2Hash([nullifier]);
const nullifierHashHex = "0x" + nullifierHash.toString(16).padStart(64, "0");

console.log("Commitment (hash of 3 values):");
console.log("  JS:    ", commitmentHex);
console.log("  Noir:  ", expectedCommitment);
console.log("  Match: ", commitmentHex === expectedCommitment);
console.log("");
console.log("Nullifier hash (hash of 1 value):");
console.log("  JS:    ", nullifierHashHex);
console.log("  Noir:  ", expectedNullifierHash);
console.log("  Match: ", nullifierHashHex === expectedNullifierHash);
console.log("");

if (commitmentHex === expectedCommitment && nullifierHashHex === expectedNullifierHash) {
  console.log("SUCCESS: @zkpassport/poseidon2 is compatible with noir-lang/poseidon!");
} else {
  console.log("FAILURE: Hashes don't match");
}
