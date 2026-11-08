//! Fake of [`TranscriptStore`]: the transcripts of a fake project folder, searched by a plain
//! accent-insensitive substring match (se-store uses FTS5).

use std::path::PathBuf;

use async_trait::async_trait;
use se_domain::AssetId;

use super::InMemoryProjectRepository;
use crate::ports::{StorageError, StoredTranscript, TranscriptHit, TranscriptStore};

/// The transcripts of one fake folder.
pub(super) struct InMemoryTranscriptStore {
    repository: InMemoryProjectRepository,
    path: PathBuf,
}

impl InMemoryTranscriptStore {
    pub(super) fn new(repository: InMemoryProjectRepository, path: PathBuf) -> Self {
        Self { repository, path }
    }
}

#[async_trait]
impl TranscriptStore for InMemoryTranscriptStore {
    async fn save_transcript(&self, transcript: &StoredTranscript) -> Result<(), StorageError> {
        self.repository.with_folder(&self.path, |folder| {
            folder
                .transcripts
                .insert(transcript.asset_id, transcript.clone());
            Ok(())
        })
    }

    async fn load_transcript(
        &self,
        asset_id: AssetId,
    ) -> Result<Option<StoredTranscript>, StorageError> {
        self.repository.with_folder(&self.path, |folder| {
            Ok(folder.transcripts.get(&asset_id).cloned())
        })
    }

    async fn search_transcripts(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<TranscriptHit>, StorageError> {
        let wanted: Vec<String> = query.split_whitespace().map(fold).collect();
        self.repository.with_folder(&self.path, |folder| {
            let mut hits = Vec::new();
            for stored in folder.transcripts.values() {
                let transcript = &stored.transcript;
                for (index, segment) in transcript.segments.iter().enumerate() {
                    let segment_index = u32::try_from(index).unwrap_or(u32::MAX);
                    let text = transcript.segment_text(segment_index);
                    let folded = fold(&text);
                    if !wanted.is_empty() && wanted.iter().all(|word| folded.contains(word)) {
                        hits.push(TranscriptHit {
                            asset_id: stored.asset_id,
                            segment: segment_index,
                            start: segment.start,
                            end: segment.end,
                            text,
                        });
                    }
                }
            }
            hits.truncate(limit);
            Ok(hits)
        })
    }
}

/// Lowercase without the accents of French letters.
fn fold(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| match c {
            'à' | 'â' | 'ä' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'î' | 'ï' => 'i',
            'ô' | 'ö' => 'o',
            'ù' | 'û' | 'ü' => 'u',
            'ç' => 'c',
            other => other,
        })
        .collect()
}
