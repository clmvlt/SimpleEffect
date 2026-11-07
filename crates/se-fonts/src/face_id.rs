//! Stable ids of system font faces.

use std::path::Path;

/// The id of the face `index` of the font file at `path`: `system-` and a hash of both, stable
/// while the file stays where it is.
#[must_use]
pub fn system_face_id(path: &Path, index: u32) -> String {
    let key = format!("{}#{index}", path.display());
    format!("system-{:016x}", xxhash_rust::xxh3::xxh3_64(key.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_depend_on_the_file_and_the_face() {
        let path = Path::new("/fonts/Arial.ttc");
        assert_eq!(system_face_id(path, 0), system_face_id(path, 0));
        assert_ne!(system_face_id(path, 0), system_face_id(path, 1));
        assert!(system_face_id(path, 0).starts_with("system-"));
        assert_eq!(system_face_id(path, 0).len(), "system-".len() + 16);
    }
}
