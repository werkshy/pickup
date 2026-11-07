use std::path::Path;

use sha2::{Digest, Sha256};

/// Alphabet shared with `rand::distr::Alphanumeric`, i.e. `[0-9A-Za-z]`.
const ALPHABET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

const BASE: u128 = ALPHABET.len() as u128;

/// Number of base62 characters after the prefix. 10 chars is ~59.5 bits of
/// entropy, which makes accidental collisions negligible for any library size.
const ID_LEN: usize = 10;

const TRACK_PREFIX: &str = "trk_";
const CATEGORY_PREFIX: &str = "cat_";
const ARTIST_PREFIX: &str = "art_";
const ALBUM_PREFIX: &str = "alb_";
const DISC_PREFIX: &str = "dsc_";

/**
 * Stable ID for a track, hashed from its path relative to the music dir.
 */
pub fn generate_track_id(path: &Path) -> String {
    id_for(TRACK_PREFIX, path)
}

/**
 * Stable ID for a category, hashed from its directory name. The implicit
 * default category has no directory, so it is hashed from its logical path
 * (the category name).
 */
pub fn generate_category_id(path: &Path) -> String {
    id_for(CATEGORY_PREFIX, path)
}

/**
 * Stable ID for an artist, hashed from the `category/artist` path that locates
 * it. Including the category means two artists sharing a name in different
 * categories get distinct IDs, which matches them being separate entries in the
 * collection.
 */
pub fn generate_artist_id(path: &Path) -> String {
    id_for(ARTIST_PREFIX, path)
}

/**
 * Stable ID for an album, hashed from the `category/[artist/]album` path that
 * locates it. A bare album and an album under an artist therefore hash
 * different paths, even if they share a name.
 */
pub fn generate_album_id(path: &Path) -> String {
    id_for(ALBUM_PREFIX, path)
}

/**
 * Stable ID for a disc, hashed from the `category/[artist/]album/disc` path that
 * locates it. A disc is nested under its album, so it needs its own prefix:
 * without one, disc 1 and disc 2 of the same album would share an ID.
 */
pub fn generate_disc_id(path: &Path) -> String {
    id_for(DISC_PREFIX, path)
}

/**
 * Hash the path something sits at into a prefixed base62 ID.
 */
fn id_for(prefix: &str, path: &Path) -> String {
    let digest = Sha256::digest(path.to_string_lossy().as_bytes());
    format!("{prefix}{}", hash_to_id(&digest))
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

    use super::{
        generate_album_id, generate_artist_id, generate_category_id, generate_disc_id,
        generate_track_id,
    };

    const TRACK_PATH: &str = "Music/Artist/Album/01 Track.mp3";
    const CATEGORY_PATH: &str = "_Grunge";
    const ARTIST_PATH: &str = "Music/Artist";
    const ALBUM_PATH: &str = "Music/Artist/Album";
    const DISC_PATH: &str = "Music/Artist/Album/CD 1";

    #[test]
    fn test_id_is_prefixed_and_correct_length() {
        for (prefix, id) in [
            ("trk_", generate_track_id(Path::new(TRACK_PATH))),
            ("cat_", generate_category_id(Path::new(CATEGORY_PATH))),
            ("art_", generate_artist_id(Path::new(ARTIST_PATH))),
            ("alb_", generate_album_id(Path::new(ALBUM_PATH))),
            ("dsc_", generate_disc_id(Path::new(DISC_PATH))),
        ] {
            assert!(id.starts_with(prefix), "{id} should start with {prefix}");
            assert_eq!(prefix.len() + 10, id.len());
            assert!(id[prefix.len()..]
                .chars()
                .all(|c| c.is_ascii_alphanumeric()));
        }
    }

    #[test]
    fn test_id_is_deterministic() {
        assert_eq!(
            generate_track_id(Path::new(TRACK_PATH)),
            generate_track_id(Path::new(TRACK_PATH))
        );
        assert_eq!(
            generate_artist_id(Path::new(ARTIST_PATH)),
            generate_artist_id(Path::new(ARTIST_PATH))
        );
        assert_eq!(
            generate_album_id(Path::new(ALBUM_PATH)),
            generate_album_id(Path::new(ALBUM_PATH))
        );
        assert_eq!(
            generate_disc_id(Path::new(DISC_PATH)),
            generate_disc_id(Path::new(DISC_PATH))
        );
    }

    #[test]
    fn test_ids_differ_by_path() {
        let a = generate_track_id(Path::new(TRACK_PATH));
        let b = generate_track_id(Path::new("Music/Artist/Album/02 Track.mp3"));

        assert_ne!(a, b);
    }

    /**
     * Disc N is an `Album` nested inside its parent album, so both are looked up
     * in the same ID space. The prefix is the only thing keeping them apart.
     */
    #[test]
    fn test_album_and_disc_ids_differ_for_nested_paths() {
        let album = generate_album_id(Path::new(DISC_PATH));
        let disc = generate_disc_id(Path::new(DISC_PATH));

        assert_ne!(album, disc);
    }

    #[test]
    fn test_disc_ids_differ_within_an_album() {
        let one = generate_disc_id(Path::new("Music/Artist/Album/CD 1"));
        let two = generate_disc_id(Path::new("Music/Artist/Album/CD 2"));

        assert_ne!(one, two);
    }

    /**
     * Artist names are only unique within a category, so the category is part
     * of the hash.
     */
    #[test]
    fn test_artist_ids_are_scoped_to_category() {
        let music = generate_artist_id(Path::new("Music/Artist"));
        let free = generate_artist_id(Path::new("_Free/Artist"));

        assert_ne!(music, free);
    }

    /**
     * A bare album is keyed under the category, an album under an artist under
     * the artist. Same album name either way.
     */
    #[test]
    fn test_album_ids_differ_between_bare_and_artist_album() {
        let bare = generate_album_id(Path::new("_Free/Album"));
        let under_artist = generate_album_id(Path::new("_Free/Artist/Album"));

        assert_ne!(bare, under_artist);
    }
}
