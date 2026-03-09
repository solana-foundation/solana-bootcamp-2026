import { poseidon2Hash } from "@zkpassport/poseidon2";

const TREE_DEPTH = 10;

// Compute empty tree zeros for the new Poseidon2 implementation
function computeEmptyTreeZeros(): string[] {
  const zeros: string[] = [];
  let current = 0n;

  for (let i = 0; i < TREE_DEPTH; i++) {
    zeros.push("0x" + current.toString(16).padStart(64, "0"));
    current = poseidon2Hash([current, current]);
  }

  return zeros;
}

const zeros = computeEmptyTreeZeros();
console.log("const EMPTY_TREE_ZEROS = [");
zeros.forEach((z, i) => {
  console.log(`  "${z}",${i < zeros.length - 1 ? "" : ""}`);
});
console.log("];");
