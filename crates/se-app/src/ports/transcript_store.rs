//! [`TranscriptStore`]: the transcripts of a project's assets, kept in its database (tables
//! `transcripts`, `transcript_words`, `transcript_segments` and the FTS5 index
//! `transcript_search` of `project.db`, implemented by se-store).

use async_trait::async_trait;
use se_domain::{AssetId, Flicks, Transcript};

use super::StorageError;

/// One stored transcript of an asset.
#[derive(Debug, Clone, PartialEq)]
pub struct StoredTranscript {
    /// The asset transcribed.
    pub asset_id: AssetId,
    /// Cache key of the content transcribed (`ContentFingerprint::cache_key`): a transcript whose
    /// key differs from the asset's current one is stale (the asset was relinked).
    pub content_key: String,
    /// When it was made, in milliseconds since the Unix epoch (the summary's `transcribedAt`).
    pub created_at: i64,
    /// Where it ran (`cuda`, `metal`, `cpu`…), for diagnostics.
    pub backend: String,
    /// The words and segments.
    pub transcript: Transcript,
}

/// A segment of a stored transcript matching a search.
#[derive(Debug, Clone, PartialEq)]
pub struct TranscriptHit {
    /// The asset.
    pub asset_id: AssetId,
    /// The segment's index in its transcript.
    pub segment: u32,
    /// Its start in the asset.
    pub start: Flicks,
    /// Its end in the asset.
    pub end: Flicks,
    /// Its text.
    pub text: String,
}

/// Transcripts of one open project. Shared by the project's actor handle and its jobs; it is not
/// document state, so it is written outside the actor.
#[async_trait]
pub trait TranscriptStore: Send + Sync {
    /// Saves `transcript`, replacing the asset's previous one (words, segments, search index) in
    /// one write.
    ///
    /// # Errors
    /// IO errors.
    async fn save_transcript(&self, transcript: &StoredTranscript) -> Result<(), StorageError>;

    /// The stored transcript of `asset_id`, if any (rows of a newer format are ignored).
    ///
    /// # Errors
    /// [`StorageError::Corrupted`] for unreadable rows; IO errors.
    async fn load_transcript(
        &self,
        asset_id: AssetId,
    ) -> Result<Option<StoredTranscript>, StorageError>;

    /// Segments whose text holds every word of `query` (accents and case ignored, the last word
    /// as a prefix), best matches first, at most `limit`.
    ///
    /// # Errors
    /// IO errors.
    async fn search_transcripts(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<TranscriptHit>, StorageError>;
}
