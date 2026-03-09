/**
 * Initialize Pool Script
 *
 * Run this once after deploying the program to initialize the pool.
 *
 * Usage: bun run init-pool.ts
 */

import {
  createSolanaRpc,
  address,
  getProgramDerivedAddress,
  pipe,
  createTransactionMessage,
  setTransactionMessageFeePayerSigner,
  setTransactionMessageLifetimeUsingBlockhash,
  appendTransactionMessageInstructions,
  signTransactionMessageWithSigners,
  createKeyPairSignerFromBytes,
  getAddressEncoder,
  getBase64EncodedWireTransaction,
} from '@solana/kit';
import * as fs from 'fs';
import * as os from 'os';

// Update this to match your deployed program ID
const PROGRAM_ID = 'HzEfEnt2E6T6gmy9VQi2d15TN5PYAy78iq7WHPF9ddHB';
const RPC_URL = 'https://api.devnet.solana.com';

async function initializePool() {
  console.log('🚀 Initializing Pool...\n');

  const rpc = createSolanaRpc(RPC_URL);
  const programAddress = address(PROGRAM_ID);

  // Load keypair from default Solana CLI location
  const keypairPath = os.homedir() + '/.config/solana/id.json';
  if (!fs.existsSync(keypairPath)) {
    console.error('❌ No keypair found at', keypairPath);
    console.error('   Run: solana-keygen new');
    process.exit(1);
  }

  const keypairData = JSON.parse(fs.readFileSync(keypairPath, 'utf-8'));
  const signer = await createKeyPairSignerFromBytes(new Uint8Array(keypairData));

  console.log('Authority:', signer.address);
  console.log('Program:', PROGRAM_ID);
  console.log('');

  // Derive PDAs
  const encoder = new TextEncoder();
  const addressEncoder = getAddressEncoder();

  const [poolPda] = await getProgramDerivedAddress({
    programAddress,
    seeds: [encoder.encode('pool')],
  });

  const [nullifierSetPda] = await getProgramDerivedAddress({
    programAddress,
    seeds: [encoder.encode('nullifiers'), addressEncoder.encode(poolPda)],
  });

  const [poolVaultPda] = await getProgramDerivedAddress({
    programAddress,
    seeds: [encoder.encode('vault'), addressEncoder.encode(poolPda)],
  });

  console.log('PDAs:');
  console.log('  Pool:', poolPda);
  console.log('  Nullifier Set:', nullifierSetPda);
  console.log('  Vault:', poolVaultPda);
  console.log('');

  // Check if already initialized
  const accountInfo = await rpc.getAccountInfo(poolPda, { encoding: 'base64' }).send();
  if (accountInfo.value) {
    console.log('✅ Pool is already initialized!');
    return;
  }

  console.log('Pool not initialized. Sending initialize transaction...');

  // Build initialize instruction
  // Discriminator for initialize: [175, 175, 109, 31, 13, 152, 155, 237]
  const discriminator = new Uint8Array([175, 175, 109, 31, 13, 152, 155, 237]);

  const instruction = {
    programAddress,
    accounts: [
      { address: poolPda, role: 1 },              // writable
      { address: nullifierSetPda, role: 1 },      // writable
      { address: poolVaultPda, role: 0 },         // readonly
      { address: signer.address, role: 3 },       // writable signer
      { address: address('11111111111111111111111111111111'), role: 0 }, // system program
    ],
    data: discriminator,
  };

  const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();

  const message = pipe(
    createTransactionMessage({ version: 0 }),
    tx => setTransactionMessageFeePayerSigner(signer, tx),
    tx => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, tx),
    tx => appendTransactionMessageInstructions([instruction], tx),
  );

  const signedTx = await signTransactionMessageWithSigners(message);

  // Send using raw RPC call
  const encodedTx = getBase64EncodedWireTransaction(signedTx);
  const result = await rpc.sendTransaction(encodedTx, { encoding: 'base64' }).send();

  console.log('');
  console.log('✅ Initialize transaction sent!');
  console.log('   Signature:', result);
  console.log('   Explorer: https://explorer.solana.com/tx/' + result + '?cluster=devnet');
  console.log('');
  console.log('⏳ Waiting for confirmation...');

  // Wait a bit for confirmation
  await new Promise(resolve => setTimeout(resolve, 5000));

  // Verify
  const poolAccount = await rpc.getAccountInfo(poolPda, { encoding: 'base64' }).send();
  if (poolAccount.value) {
    console.log('✅ Pool initialized successfully!');
  } else {
    console.log('⚠️  Transaction sent but pool not found yet. Check explorer.');
  }
}

initializePool().catch(err => {
  console.error('Error:', err.message || err);
  process.exit(1);
});
