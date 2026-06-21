use rand::Rng;

/// Generate random padding bytes of the given length.
pub fn generate(length: usize) -> Vec<u8> {
    if length == 0 {
        return Vec::new();
    }
    let mut rng = rand::thread_rng();
    (0..length).map(|_| rng.gen()).collect()
}

/// Compute padding length to bring a payload to the next block boundary.
///
/// `block_size` must be a power of two or any positive integer.
/// If `current_len` is already aligned, returns `block_size` (adds a full pad block).
pub fn pad_to_boundary(current_len: usize, block_size: usize) -> usize {
    if block_size == 0 {
        return 0;
    }
    let remainder = current_len % block_size;
    if remainder == 0 {
        block_size
    } else {
        block_size - remainder
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_padding_length() {
        let pad = generate(32);
        assert_eq!(pad.len(), 32);
    }

    #[test]
    fn test_generate_padding_zero() {
        let pad = generate(0);
        assert!(pad.is_empty());
    }

    #[test]
    fn test_pad_to_boundary_aligned() {
        assert_eq!(pad_to_boundary(64, 32), 32);
    }

    #[test]
    fn test_pad_to_boundary_unaligned() {
        assert_eq!(pad_to_boundary(10, 32), 22);
    }

    #[test]
    fn test_pad_to_boundary_exact() {
        assert_eq!(pad_to_boundary(0, 32), 32);
    }

    #[test]
    fn test_pad_to_boundary_zero_block() {
        assert_eq!(pad_to_boundary(42, 0), 0);
    }
}
