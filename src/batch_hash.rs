//! Batched hashing of 64-byte parent-node inputs.

/// Hash each 64-byte chunk of `input` into the corresponding 32-byte digest in `output`.
///
/// Uses the `hashtree` SIMD backend when available, which hashes many independent
/// chunks per call, and falls back to `ethereum_hashing` otherwise.
pub fn hash_pairs(input: &[u8], output: &mut [u8]) {
    debug_assert_eq!(input.len(), output.len() * 2);
    debug_assert_eq!(input.len() % 64, 0);
    if hashtree::hash_pairs(input, output) {
        return;
    }
    for (chunk, out) in input.chunks_exact(64).zip(output.chunks_exact_mut(32)) {
        out.copy_from_slice(&ethereum_hashing::hash_fixed(chunk));
    }
}

#[cfg(all(
    feature = "hashtree",
    any(target_arch = "x86_64", target_arch = "aarch64")
))]
mod hashtree {
    use std::sync::LazyLock;

    /// `hashtree_rs::init` selects the best backend for this CPU and returns 0 if
    /// none is available.
    static AVAILABLE: LazyLock<bool> = LazyLock::new(|| hashtree_rs::init() != 0);

    pub fn hash_pairs(input: &[u8], output: &mut [u8]) -> bool {
        if !*AVAILABLE {
            return false;
        }
        hashtree_rs::hash(output, input, input.len() / 64);
        true
    }
}

#[cfg(not(all(
    feature = "hashtree",
    any(target_arch = "x86_64", target_arch = "aarch64")
)))]
mod hashtree {
    pub fn hash_pairs(_input: &[u8], _output: &mut [u8]) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_ethereum_hashing() {
        let input: Vec<u8> = (0..64 * 33).map(|i| (i % 251) as u8).collect();
        let mut output = vec![0u8; 32 * 33];
        hash_pairs(&input, &mut output);
        for (chunk, out) in input.chunks_exact(64).zip(output.chunks_exact(32)) {
            assert_eq!(out, ethereum_hashing::hash_fixed(chunk));
        }
    }
}
