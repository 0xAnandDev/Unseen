//! Helpers for the 256-bit perceptual hash (PDQ-style).
//!
//! A PDQ hash is 256 bits. Written as hex, that is exactly 64 characters
//! (each hex character holds 4 bits, and 64 x 4 = 256).

/// Size of a hash in hex characters.
pub const HASH_HEX_LEN: usize = 64;

/// Highest possible Hamming distance between two 256-bit hashes.
pub const MAX_DISTANCE: i32 = 256;

/// Default "close enough" threshold. PDQ's authors suggest treating
/// hashes 31 bits apart or less as the same picture.
pub const DEFAULT_MAX_DISTANCE: i32 = 31;

/// Check that the text is a valid 64-character hex hash and return it in lowercase.
pub fn normalize_hash(input: &str) -> Result<String, String> {
    let trimmed = input.trim();
    if trimmed.len() != HASH_HEX_LEN {
        return Err(format!(
            "hash must be {HASH_HEX_LEN} hex characters, got {}",
            trimmed.len()
        ));
    }
    if !trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("hash may only contain 0-9 and a-f".to_string());
    }
    Ok(trimmed.to_ascii_lowercase())
}

/// Count how many bits differ between two hex hashes (the Hamming distance).
/// The database does this for search; this Rust version is used in tests
/// and is handy for understanding what the SQL does.
#[allow(dead_code)] // used by tests now; the hash checker will use it later
pub fn hamming_distance(a: &str, b: &str) -> Option<u32> {
    let a = hex::decode(a).ok()?;
    let b = hex::decode(b).ok()?;
    if a.len() != b.len() {
        return None;
    }
    // XOR gives a 1 wherever the bits are different; count_ones counts them.
    Some(a.iter().zip(&b).map(|(x, y)| (x ^ y).count_ones()).sum())
}

#[cfg(test)]
mod tests {
    use super::*;

    const ZEROS: &str = "0000000000000000000000000000000000000000000000000000000000000000";

    #[test]
    fn accepts_valid_hash_and_lowercases_it() {
        let upper = "ABCDEF".repeat(10) + "0123";
        assert_eq!(normalize_hash(&upper).unwrap(), upper.to_lowercase());
    }

    #[test]
    fn rejects_wrong_length() {
        assert!(normalize_hash("abc").is_err());
    }

    #[test]
    fn rejects_non_hex() {
        let bad = "z".repeat(64);
        assert!(normalize_hash(&bad).is_err());
    }

    #[test]
    fn distance_of_identical_hashes_is_zero() {
        assert_eq!(hamming_distance(ZEROS, ZEROS), Some(0));
    }

    #[test]
    fn distance_counts_flipped_bits() {
        // "f" is 1111 in binary, so changing one 0 to f flips 4 bits.
        let four_bits = format!("f{}", &ZEROS[1..]);
        assert_eq!(hamming_distance(ZEROS, &four_bits), Some(4));
        let all_ones = "f".repeat(64);
        assert_eq!(hamming_distance(ZEROS, &all_ones), Some(256));
    }
}
