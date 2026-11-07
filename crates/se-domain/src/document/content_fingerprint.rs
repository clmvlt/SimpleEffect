//! [`ContentFingerprint`]: a quick identity of a media file's content.

use serde::{Deserialize, Deserializer, Serialize};
use ts_rs::TS;

/// Number of hexadecimal digits of [`ContentFingerprint::sample_hash`] (a 64-bit hash).
const SAMPLE_HASH_DIGITS: usize = 16;

/// A quick identity of a file's content, computed by se-media at import (ADR-0018): the size,
/// the modification time and a 64-bit xxh3 hash of the first and last mebibyte.
///
/// Two files with the same size and sample hash are taken for the same content (deduplication at
/// import, relinking); the modification time is part of [`ContentFingerprint::cache_key`] only, so
/// a file rewritten in place gets fresh thumbnails and peaks.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ContentFingerprint {
    size_bytes: u64,
    modified_unix_ms: i64,
    sample_hash: String,
}

/// A sample hash that is not 16 lowercase hexadecimal digits.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("'{0}' is not a sample hash: expected 16 lowercase hexadecimal digits")]
pub struct FingerprintError(pub String);

impl ContentFingerprint {
    /// A fingerprint from its parts.
    ///
    /// # Errors
    /// [`FingerprintError`] unless `sample_hash` is 16 lowercase hexadecimal digits (it names a
    /// cache folder, so nothing else may get in).
    pub fn new(
        size_bytes: u64,
        modified_unix_ms: i64,
        sample_hash: impl Into<String>,
    ) -> Result<Self, FingerprintError> {
        let sample_hash = sample_hash.into();
        let valid = sample_hash.len() == SAMPLE_HASH_DIGITS
            && sample_hash
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
        if !valid {
            return Err(FingerprintError(sample_hash));
        }
        Ok(Self {
            size_bytes,
            modified_unix_ms,
            sample_hash,
        })
    }

    /// Size of the file in bytes.
    #[must_use]
    pub fn size_bytes(&self) -> u64 {
        self.size_bytes
    }

    /// Last modification time, in milliseconds since the Unix epoch.
    #[must_use]
    pub fn modified_unix_ms(&self) -> i64 {
        self.modified_unix_ms
    }

    /// xxh3-64 of the first and last mebibyte, as 16 lowercase hexadecimal digits.
    #[must_use]
    pub fn sample_hash(&self) -> &str {
        &self.sample_hash
    }

    /// Whether `other` describes the same content (size and sample hash; the modification time
    /// changes when a file is copied without its attributes).
    #[must_use]
    pub fn same_content(&self, other: &Self) -> bool {
        self.size_bytes == other.size_bytes && self.sample_hash == other.sample_hash
    }

    /// Name of the cache folder of this content (`<hash>-<size>-<mtime>`): only digits, letters
    /// and `-`, so it is a safe file name on every OS.
    #[must_use]
    pub fn cache_key(&self) -> String {
        format!(
            "{}-{}-{}",
            self.sample_hash, self.size_bytes, self.modified_unix_ms
        )
    }
}

impl<'de> Deserialize<'de> for ContentFingerprint {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Raw {
            size_bytes: u64,
            modified_unix_ms: i64,
            sample_hash: String,
        }
        let raw = Raw::deserialize(deserializer)?;
        Self::new(raw.size_bytes, raw.modified_unix_ms, raw.sample_hash)
            .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HASH: &str = "0123456789abcdef";

    #[test]
    fn same_content_ignores_the_modification_time() {
        let original = ContentFingerprint::new(1000, 5, HASH).unwrap();
        let copy = ContentFingerprint::new(1000, 9, HASH).unwrap();
        let other = ContentFingerprint::new(1001, 5, HASH).unwrap();
        assert!(original.same_content(&copy));
        assert!(!original.same_content(&other));
        assert_ne!(original.cache_key(), copy.cache_key());
    }

    #[test]
    fn cache_keys_are_safe_file_names() {
        let fingerprint = ContentFingerprint::new(42, -3, HASH).unwrap();
        assert_eq!(fingerprint.cache_key(), "0123456789abcdef-42--3");
    }

    #[test]
    fn hashes_other_than_sixteen_lowercase_hex_digits_are_refused() {
        for hash in [
            "",
            "0123456789ABCDEF",
            "../../etc/passwd",
            "0123456789abcde",
            "g123456789abcdef",
        ] {
            assert!(ContentFingerprint::new(1, 1, hash).is_err(), "{hash}");
        }
        let json = r#"{"sizeBytes":1,"modifiedUnixMs":2,"sampleHash":"../x"}"#;
        assert!(serde_json::from_str::<ContentFingerprint>(json).is_err());
    }

    #[test]
    fn round_trips_through_json() {
        let fingerprint = ContentFingerprint::new(7, 8, HASH).unwrap();
        let json = serde_json::to_string(&fingerprint).unwrap();
        assert_eq!(
            json,
            r#"{"sizeBytes":7,"modifiedUnixMs":8,"sampleHash":"0123456789abcdef"}"#
        );
        assert_eq!(
            serde_json::from_str::<ContentFingerprint>(&json).unwrap(),
            fingerprint
        );
    }
}
