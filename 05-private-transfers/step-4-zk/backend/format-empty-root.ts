import { poseidon2Hash } from "@zkpassport/poseidon2";

const TREE_DEPTH = 10;

// Compute the final empty tree root
let current = 0n;
for (let i = 0; i < TREE_DEPTH; i++) {
  current = poseidon2Hash([current, current]);
}

const hex = current.toString(16).padStart(64, "0");
console.log("Empty root (hex):", "0x" + hex);
console.log("");
console.log("Rust format:");
console.log("pub const EMPTY_ROOT: [u8; 32] = [");

// Convert to bytes and format for Rust
const bytes: string[] = [];
for (let i = 0; i < 64; i += 2) {
  bytes.push("0x" + hex.slice(i, i + 2));
}

// Format in rows of 16 bytes
for (let i = 0; i < 32; i += 16) {
  const row = bytes.slice(i, i + 16).join(", ");
  console.log(`    ${row},`);
}
console.log("];");
