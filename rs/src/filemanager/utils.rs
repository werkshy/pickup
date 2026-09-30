use std::path::Path;

use sha2::{Digest, Sha256};

/// Alphabet shared with `rand::distr::Alphanumeric`, i.e. `[0-9A-Za-z]`.
const ALPHABET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

const BASE: u128 = ALPHABET.len() as u128;

/// Number of base62 characters after the prefix. 10 chars is ~59.5 bits of
/// entropy, which makes accidental collisions negligible for any library size.
const ID_LEN: usize = 10;

const TRACK_PREFIX: &str = "trk_";

pub fn generate_track_id(path: &Path) -> String {
    let digest = Sha256::digest(path.to_string_lossy().as_bytes());
    format!("{TRACK_PREFIX}{}", hash_to_id(&digest))
}

/**
 * Encode a digest as a fixed-length base62 string. Takes the leading 16 bytes as
 * a big-endian integer; since `2^128` dwarfs `62^ID_LEN` there is no modulo bias.
 */
fn hash_to_id(digest: &[u8]) -> String {
    let mut value = u128::from_be_bytes(digest[..16].try_into().unwrap());
    let mut id = vec![ALPHABET[0]; ID_LEN];
    for char in id.iter_mut().rev() {
        *char = ALPHABET[(value % BASE) as usize];
        value /= BASE;
    }
    String::from_utf8(id).unwrap()
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::generate_track_id;

    #[test]
    fn test_id_is_prefixed_and_correct_length() {
        let id = generate_track_id(Path::new("Music/Artist/Album/01 Track.mp3"));

        assert!(id.starts_with("trk_"));
        assert_eq!(14, id.len());
    }

    #[test]
    fn test_id_only_uses_alphanumeric() {
        let id = generate_track_id(Path::new("Music/Artist/Album/01 Track.mp3"));

        assert!(id[4..].chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn test_id_is_deterministic() {
        let path = Path::new("Music/Artist/Album/01 Track.mp3");

        assert_eq!(generate_track_id(path), generate_track_id(path));
    }

    #[test]
    fn test_ids_differ_by_path() {
        let a = generate_track_id(Path::new("Music/Artist/Album/01 Track.mp3"));
        let b = generate_track_id(Path::new("Music/Artist/Album/02 Track.mp3"));

        assert_ne!(a, b);
    }
}
