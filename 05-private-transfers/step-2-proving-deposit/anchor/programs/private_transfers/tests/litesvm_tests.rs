/// LiteSVM Tests for Private Transfers
///
/// These tests use anchor-litesvm for fast, in-process testing of the
/// deposit functionality. Full E2E tests with ZK proof verification
/// are in tests/e2e.ts (requires backend server for proof generation).

use anchor_litesvm::AnchorLiteSVM;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use sha2::{Sha256, Digest};

// System program ID
const SYSTEM_PROGRAM_ID: Pubkey = solana_sdk::pubkey!("11111111111111111111111111111111");

// Declare the program - this generates client types from the IDL
// Use a different name to avoid conflict with the crate name
mod pt_program {
    anchor_lang::declare_program!(private_transfers);
}

use pt_program::private_transfers::client;

// Program ID must match lib.rs
const PROGRAM_ID: Pubkey = solana_sdk::pubkey!("2QRZu5cWy8x8jEFc9nhsnrnQSMAKwNpiLpCXrMRb3oUn");

const LAMPORTS_PER_SOL: u64 = 1_000_000_000;

/// Compute a test commitment using SHA256 (placeholder for Poseidon)
fn compute_commitment(nullifier: &[u8; 32], secret: &[u8; 32], amount: u64) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(nullifier);
    hasher.update(secret);

    // Amount as 32-byte big-endian
    let mut amount_bytes = [0u8; 32];
    amount_bytes[24..32].copy_from_slice(&amount.to_be_bytes());
    hasher.update(&amount_bytes);

    hasher.finalize().into()
}

/// Compute a test Merkle root using SHA256 (placeholder for Poseidon)
fn compute_new_root(commitment: &[u8; 32], leaf_index: u64) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(commitment);
    hasher.update(&leaf_index.to_le_bytes());
    hasher.finalize().into()
}

/// Generate random 32 bytes
fn random_bytes() -> [u8; 32] {
    use std::time::{SystemTime, UNIX_EPOCH};
    let mut hasher = Sha256::new();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    hasher.update(&nanos.to_le_bytes());
    hasher.update(&std::process::id().to_le_bytes());
    hasher.finalize().into()
}

/// Find PDA for pool
fn find_pool_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"pool"], &PROGRAM_ID)
}

/// Find PDA for pool vault
fn find_vault_pda(pool: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"vault", pool.as_ref()], &PROGRAM_ID)
}

/// Find PDA for nullifier set
fn find_nullifier_set_pda(pool: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"nullifiers", pool.as_ref()], &PROGRAM_ID)
}

#[test]
fn test_initialize_pool() {
    // Set up LiteSVM with our program
    let mut ctx = AnchorLiteSVM::build_with_program(
        PROGRAM_ID,
        include_bytes!("../../../target/deploy/private_transfers.so"),
    );

    // Create funded authority account
    let authority = Keypair::new();
    ctx.svm.airdrop(&authority.pubkey(), 10 * LAMPORTS_PER_SOL).unwrap();

    // Find PDAs
    let (pool_pda, _) = find_pool_pda();
    let (vault_pda, _) = find_vault_pda(&pool_pda);
    let (nullifier_set_pda, _) = find_nullifier_set_pda(&pool_pda);

    // Build initialize instruction using anchor-litesvm
    let accounts = client::accounts::Initialize {
        pool: pool_pda,
        nullifier_set: nullifier_set_pda,
        pool_vault: vault_pda,
        authority: authority.pubkey(),
        system_program: SYSTEM_PROGRAM_ID,
    };

    let args = client::args::Initialize {};

    let ix = ctx.program()
        .accounts(accounts)
        .args(args)
        .instruction()
        .unwrap();

    // Execute instruction
    let result = ctx.execute_instruction(ix, &[&authority]).unwrap();
    result.assert_success();

    // Verify pool state
    let pool_account = ctx.svm.get_account(&pool_pda).expect("Pool account should exist");
    assert!(pool_account.data.len() > 8, "Pool account should have data");

    println!("✓ Pool initialized successfully");
}

#[test]
fn test_single_deposit() {
    // Set up LiteSVM with our program
    let mut ctx = AnchorLiteSVM::build_with_program(
        PROGRAM_ID,
        include_bytes!("../../../target/deploy/private_transfers.so"),
    );

    // Create funded authority/depositor account
    let authority = Keypair::new();
    ctx.svm.airdrop(&authority.pubkey(), 10 * LAMPORTS_PER_SOL).unwrap();

    // Find PDAs
    let (pool_pda, _) = find_pool_pda();
    let (vault_pda, _) = find_vault_pda(&pool_pda);
    let (nullifier_set_pda, _) = find_nullifier_set_pda(&pool_pda);

    // Initialize pool first
    let init_accounts = client::accounts::Initialize {
        pool: pool_pda,
        nullifier_set: nullifier_set_pda,
        pool_vault: vault_pda,
        authority: authority.pubkey(),
        system_program: SYSTEM_PROGRAM_ID,
    };

    let init_ix = ctx.program()
        .accounts(init_accounts)
        .args(client::args::Initialize {})
        .instruction()
        .unwrap();

    ctx.execute_instruction(init_ix, &[&authority]).unwrap().assert_success();

    // Now make a deposit
    let deposit_amount = LAMPORTS_PER_SOL / 10; // 0.1 SOL
    let nullifier = random_bytes();
    let secret = random_bytes();
    let commitment = compute_commitment(&nullifier, &secret, deposit_amount);
    let new_root = compute_new_root(&commitment, 0);

    let deposit_accounts = client::accounts::Deposit {
        pool: pool_pda,
        pool_vault: vault_pda,
        depositor: authority.pubkey(),
        system_program: SYSTEM_PROGRAM_ID,
    };

    let deposit_args = client::args::Deposit {
        commitment,
        new_root,
        amount: deposit_amount,
    };

    let deposit_ix = ctx.program()
        .accounts(deposit_accounts)
        .args(deposit_args)
        .instruction()
        .unwrap();

    let result = ctx.execute_instruction(deposit_ix, &[&authority]).unwrap();
    result.assert_success();

    println!("✓ Single deposit successful: {} lamports", deposit_amount);
}

#[test]
fn test_multiple_deposits() {
    // Set up LiteSVM with our program
    let mut ctx = AnchorLiteSVM::build_with_program(
        PROGRAM_ID,
        include_bytes!("../../../target/deploy/private_transfers.so"),
    );

    // Create funded authority/depositor account
    let authority = Keypair::new();
    ctx.svm.airdrop(&authority.pubkey(), 10 * LAMPORTS_PER_SOL).unwrap();

    // Find PDAs
    let (pool_pda, _) = find_pool_pda();
    let (vault_pda, _) = find_vault_pda(&pool_pda);
    let (nullifier_set_pda, _) = find_nullifier_set_pda(&pool_pda);

    // Initialize pool first
    let init_accounts = client::accounts::Initialize {
        pool: pool_pda,
        nullifier_set: nullifier_set_pda,
        pool_vault: vault_pda,
        authority: authority.pubkey(),
        system_program: SYSTEM_PROGRAM_ID,
    };

    let init_ix = ctx.program()
        .accounts(init_accounts)
        .args(client::args::Initialize {})
        .instruction()
        .unwrap();

    ctx.execute_instruction(init_ix, &[&authority]).unwrap().assert_success();

    // Make multiple deposits
    for i in 0..3 {
        let deposit_amount = (50_000_000 + i * 10_000_000) as u64; // 0.05 + i*0.01 SOL
        let nullifier = random_bytes();
        let secret = random_bytes();
        let commitment = compute_commitment(&nullifier, &secret, deposit_amount);
        let new_root = compute_new_root(&commitment, i as u64);

        let deposit_accounts = client::accounts::Deposit {
            pool: pool_pda,
            pool_vault: vault_pda,
            depositor: authority.pubkey(),
            system_program: SYSTEM_PROGRAM_ID,
        };

        let deposit_args = client::args::Deposit {
            commitment,
            new_root,
            amount: deposit_amount,
        };

        let deposit_ix = ctx.program()
            .accounts(deposit_accounts)
            .args(deposit_args)
            .instruction()
            .unwrap();

        let result = ctx.execute_instruction(deposit_ix, &[&authority]).unwrap();
        result.assert_success();

        println!("  Deposit {}: {} lamports", i + 1, deposit_amount);
    }

    println!("✓ Multiple deposits successful");
}

#[test]
fn test_reject_small_deposit() {
    // Set up LiteSVM with our program
    let mut ctx = AnchorLiteSVM::build_with_program(
        PROGRAM_ID,
        include_bytes!("../../../target/deploy/private_transfers.so"),
    );

    // Create funded authority/depositor account
    let authority = Keypair::new();
    ctx.svm.airdrop(&authority.pubkey(), 10 * LAMPORTS_PER_SOL).unwrap();

    // Find PDAs
    let (pool_pda, _) = find_pool_pda();
    let (vault_pda, _) = find_vault_pda(&pool_pda);
    let (nullifier_set_pda, _) = find_nullifier_set_pda(&pool_pda);

    // Initialize pool first
    let init_accounts = client::accounts::Initialize {
        pool: pool_pda,
        nullifier_set: nullifier_set_pda,
        pool_vault: vault_pda,
        authority: authority.pubkey(),
        system_program: SYSTEM_PROGRAM_ID,
    };

    let init_ix = ctx.program()
        .accounts(init_accounts)
        .args(client::args::Initialize {})
        .instruction()
        .unwrap();

    ctx.execute_instruction(init_ix, &[&authority]).unwrap().assert_success();

    // Try to make a deposit below minimum (0.0001 SOL = 100_000 lamports)
    let below_min_amount = 100_000u64; // Below MIN_DEPOSIT_AMOUNT (1_000_000)
    let nullifier = random_bytes();
    let secret = random_bytes();
    let commitment = compute_commitment(&nullifier, &secret, below_min_amount);
    let new_root = compute_new_root(&commitment, 0);

    let deposit_accounts = client::accounts::Deposit {
        pool: pool_pda,
        pool_vault: vault_pda,
        depositor: authority.pubkey(),
        system_program: SYSTEM_PROGRAM_ID,
    };

    let deposit_args = client::args::Deposit {
        commitment,
        new_root,
        amount: below_min_amount,
    };

    let deposit_ix = ctx.program()
        .accounts(deposit_accounts)
        .args(deposit_args)
        .instruction()
        .unwrap();

    // Execute instruction - this should succeed but the transaction should fail
    let result = ctx.execute_instruction(deposit_ix, &[&authority]).unwrap();

    // This should fail with DepositTooSmall error
    result.assert_failure();

    println!("✓ Small deposit correctly rejected");
}
