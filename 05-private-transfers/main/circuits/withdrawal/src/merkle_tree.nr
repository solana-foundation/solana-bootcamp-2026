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
pub fn compute_merkle_root<let DEPTH: u32>(
    leaf: Field,
    path: [Field; DEPTH],
    is_even: [bool; DEPTH]
) -> Field {
    let mut current = leaf;

    for i in 0..DEPTH {
        let sibling = path[i];

        // If is_even[i] is true, current is on the left (even index)
        // Otherwise current is on the right (odd index)
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
