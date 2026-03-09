// Merkle tree utilities for proving membership.
// A Merkle proof shows a leaf exists in a tree by providing sibling hashes
// from the leaf up to the root. We rehash at each level and check we get the expected root.
use dep::poseidon;

/// Computes the Merkle root from a leaf and its proof
///
/// # Arguments
/// * `leaf` - The leaf value to verify
/// * `path` - Array of sibling hashes along the path to root
/// * `is_even` - Boolean array indicating if leaf is on left (even) or right (odd) at each level
///
/// # Returns
/// The computed Merkle root
// `<let DEPTH: u32>` is a Noir generic - the caller specifies DEPTH at compile time.
// This lets us reuse this function for trees of any depth.
pub fn compute_merkle_root<let DEPTH: u32>(
    leaf: Field,
    path: [Field; DEPTH],
    is_even: [bool; DEPTH]
) -> Field {
    let mut current = leaf;

    for i in 0..DEPTH {
        let sibling = path[i];

        // is_even[i] tells us which side our node is on at level i.
        // true = left child (even index like 0,2,4), false = right child (odd index like 1,3,5).
        // We need this to know the hash order: Poseidon(left, right) not Poseidon(right, left).
        let (left, right) = if is_even[i] {
            (current, sibling)
        } else {
            (sibling, current)
        };

        current = poseidon::poseidon2::Poseidon2::hash([left, right], 2);
    }

    current
}

/// Verifies a Merkle proof
///
/// # Arguments
/// * `leaf` - The leaf value to verify
/// * `root` - The expected Merkle root
/// * `path` - Array of sibling hashes
/// * `is_even` - Boolean array for path direction
///
/// # Returns
/// True if the proof is valid
pub fn verify_merkle_proof<let DEPTH: u32>(
    leaf: Field,
    root: Field,
    path: [Field; DEPTH],
    is_even: [bool; DEPTH]
) -> bool {
    let computed_root = compute_merkle_root(leaf, path, is_even);
    computed_root == root
}
