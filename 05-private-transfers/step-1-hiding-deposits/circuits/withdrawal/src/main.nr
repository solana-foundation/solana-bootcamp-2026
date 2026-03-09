mod merkle_tree;

use dep::poseidon;
use merkle_tree::compute_merkle_root;

// Merkle tree depth - supports 2^10 = 1024 deposits
global TREE_DEPTH: u32 = 10;

/// Private Transfer Circuit
///
/// Proves knowledge of (nullifier, secret, amount) such that:
/// 1. commitment = Poseidon(nullifier, secret, amount) exists in the Merkle tree
/// 2. nullifier_hash = Poseidon(nullifier) matches the public input
/// 3. The proof is bound to a specific recipient (public input)
fn main(
    root: pub Field,
    nullifier_hash: pub Field,
    recipient: pub Field,
    amount: pub Field,
    nullifier: Field,
    secret: Field,
    merkle_proof: [Field; TREE_DEPTH],
    is_even: [bool; TREE_DEPTH]
) {
    let commitment = poseidon::poseidon2::Poseidon2::hash([nullifier, secret, amount], 3);

    let computed_nullifier_hash = poseidon::poseidon2::Poseidon2::hash([nullifier], 1);
    assert(computed_nullifier_hash == nullifier_hash, "Invalid nullifier hash");

    let computed_root = compute_merkle_root(commitment, merkle_proof, is_even);
    assert(computed_root == root, "Invalid Merkle proof");

    // recipient is a public input, binding the proof to this specific recipient
    let _ = recipient;
}

fn compute_commitment(nullifier: Field, secret: Field, amount: Field) -> Field {
    poseidon::poseidon2::Poseidon2::hash([nullifier, secret, amount], 3)
}

fn compute_nullifier_hash(nullifier: Field) -> Field {
    poseidon::poseidon2::Poseidon2::hash([nullifier], 1)
}

#[test]
fn test_commitment_computation() {
    let nullifier = 12345;
    let secret = 67890;
    let amount = 1000000000; // 1 SOL in lamports
    let commitment = compute_commitment(nullifier, secret, amount);
    // Commitment should be deterministic
    let commitment2 = compute_commitment(nullifier, secret, amount);
    assert(commitment == commitment2);

    // Different amount should produce different commitment
    let commitment3 = compute_commitment(nullifier, secret, amount + 1);
    assert(commitment != commitment3);
}

#[test]
fn test_nullifier_hash() {
    let nullifier = 12345;
    let hash1 = compute_nullifier_hash(nullifier);
    let hash2 = compute_nullifier_hash(nullifier);
    assert(hash1 == hash2);

    // Different nullifier should produce different hash
    let hash3 = compute_nullifier_hash(nullifier + 1);
    assert(hash1 != hash3);
}

#[test]
fn test_commitment_uniqueness() {
    // Same nullifier/secret but different amounts should produce different commitments
    let nullifier = 99999;
    let secret = 88888;
    let amount1 = 100000000;  // 0.1 SOL
    let amount2 = 200000000;  // 0.2 SOL

    let commitment1 = compute_commitment(nullifier, secret, amount1);
    let commitment2 = compute_commitment(nullifier, secret, amount2);

    assert(commitment1 != commitment2);
}

fn compute_empty_tree_zeros() -> [Field; TREE_DEPTH] {
    let mut zeros: [Field; TREE_DEPTH] = [0; TREE_DEPTH];
    let mut current: Field = 0;

    for i in 0..TREE_DEPTH {
        zeros[i] = current;
        current = poseidon::poseidon2::Poseidon2::hash([current, current], 2);
    }

    zeros
}

#[test]
fn test_generate_valid_inputs() {
    let nullifier: Field = 12345;
    let secret: Field = 67890;
    let amount: Field = 1000000000;

    let commitment = compute_commitment(nullifier, secret, amount);
    let nullifier_hash = compute_nullifier_hash(nullifier);
    let zeros = compute_empty_tree_zeros();

    println(commitment);
    println(nullifier_hash);
    println(zeros);

    // For leaf at index 0, siblings are empty subtrees
    let merkle_proof = zeros;
    let is_even = [true; TREE_DEPTH];

    let root = merkle_tree::compute_merkle_root(commitment, merkle_proof, is_even);
    println(root);

    let computed = merkle_tree::compute_merkle_root(commitment, merkle_proof, is_even);
    assert(computed == root);
}
