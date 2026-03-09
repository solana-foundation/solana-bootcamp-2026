import { Noir } from "@noir-lang/noir_js";
import * as fs from "fs";
import * as path from "path";

const __dirname = path.dirname(new URL(import.meta.url).pathname);

async function main() {
  // Load the compiled circuit
  const circuitPath = path.resolve(__dirname, "../circuits/hasher/target/hasher.json");
  const circuit = JSON.parse(fs.readFileSync(circuitPath, "utf-8"));

  // Create Noir instance
  const noir = new Noir(circuit);

  // Test values
  const nullifier = "12345";
  const secret = "67890";
  const amount = "1000000000";

  // Execute the circuit to get the witness
  const { returnValue } = await noir.execute({
    nullifier,
    secret,
    amount,
  });

  console.log("Noir JS execution result:");
  console.log("Return value:", returnValue);

  // The return value should be [commitment, nullifier_hash]
  if (Array.isArray(returnValue)) {
    const commitment = returnValue[0];
    const nullifierHash = returnValue[1];
    console.log("");
    console.log("Commitment:", commitment);
    console.log("Nullifier hash:", nullifierHash);
  }
}

main().catch(console.error);
