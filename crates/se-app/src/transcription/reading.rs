//! Reading transcripts (an asset's, a sequence's, full-text search) and the project's
//! transcription settings (one undoable edit).

use std::collections::BTreeMap;

use se_domain::{
    Actor, AssetId, ItemKind, ProjectId, SequenceId, SequenceTranscript, SetTranscriptionSettings,
    TrackKind, Transaction, Transcript, TranscriptionSettings, sequence_transcript,
};

use super::{TranscriptionService, labels};
use crate::{AppError, new_ids::NewIds, ports::TranscriptHit, project_actor::ProjectActorHandle};

/// The most search hits returned.
pub const MAX_SEARCH_HITS: usize = 50;

/// A sequence's words and what is missing from them.
#[derive(Debug, Clone, PartialEq)]
pub struct SequenceWords {
    /// The words of transcribed assets, in sequence time.
    pub transcript: SequenceTranscript,
    /// Assets played by the sequence's sound clips that have no transcript of their current
    /// content.
    pub untranscribed: Vec<AssetId>,
}

impl TranscriptionService {
    /// The transcript of `asset_id`'s current content (`None` when not transcribed, or
    /// transcribed before a relink to other content).
    ///
    /// # Errors
    /// [`AppError::ProjectNotOpen`], [`AppError::AssetNotFound`], [`AppError::Storage`].
    pub async fn transcript(
        &self,
        project_id: ProjectId,
        asset_id: AssetId,
    ) -> Result<Option<Transcript>, AppError> {
        let handle = self.handle(project_id)?;
        let project = handle.view().await?.project;
        let asset = project
            .assets
            .iter()
            .find(|asset| asset.id == asset_id)
            .ok_or(AppError::AssetNotFound {
                project_id,
                asset_id,
            })?;
        let Some(fingerprint) = &asset.fingerprint else {
            return Ok(None);
        };
        current_transcript(&handle, asset_id, &fingerprint.cache_key()).await
    }

    /// The words of `sequence_id` in sequence time, through its sound clips.
    ///
    /// # Errors
    /// [`AppError::ProjectNotOpen`], [`AppError::Domain`] (unknown sequence), [`AppError::Storage`].
    pub async fn sequence_words(
        &self,
        project_id: ProjectId,
        sequence_id: SequenceId,
    ) -> Result<SequenceWords, AppError> {
        let handle = self.handle(project_id)?;
        let project = handle.view().await?.project;
        let sequence = project
            .sequences
            .iter()
            .find(|sequence| sequence.id == sequence_id)
            .ok_or(AppError::Domain(se_domain::DomainError::SequenceNotFound {
                sequence: sequence_id,
            }))?;
        let played: Vec<AssetId> = sequence
            .tracks
            .iter()
            .filter(|track| track.kind == TrackKind::Audio)
            .flat_map(|track| &track.items)
            .filter_map(|item| match &item.kind {
                ItemKind::MediaClip(clip) => Some(clip.asset_id),
                _ => None,
            })
            .collect();
        let mut transcripts = BTreeMap::new();
        let mut untranscribed = Vec::new();
        for asset_id in played {
            if transcripts.contains_key(&asset_id) || untranscribed.contains(&asset_id) {
                continue;
            }
            let key = project
                .assets
                .iter()
                .find(|asset| asset.id == asset_id)
                .and_then(|asset| asset.fingerprint.as_ref())
                .map(se_domain::ContentFingerprint::cache_key);
            let transcript = match key {
                Some(key) => current_transcript(&handle, asset_id, &key).await?,
                None => None,
            };
            match transcript {
                Some(transcript) => {
                    transcripts.insert(asset_id, transcript);
                }
                None => untranscribed.push(asset_id),
            }
        }
        Ok(SequenceWords {
            transcript: sequence_transcript(sequence, |asset| transcripts.get(&asset)),
            untranscribed,
        })
    }

    /// Transcript segments holding every word of `query`, best first (at most
    /// [`MAX_SEARCH_HITS`]).
    ///
    /// # Errors
    /// [`AppError::ProjectNotOpen`], [`AppError::Storage`].
    pub async fn search(
        &self,
        project_id: ProjectId,
        query: &str,
    ) -> Result<Vec<TranscriptHit>, AppError> {
        let handle = self.handle(project_id)?;
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }
        Ok(handle
            .transcripts()
            .search_transcripts(query, MAX_SEARCH_HITS)
            .await?)
    }

    /// Sets the project's transcription language and vocabulary (one undoable user edit; nothing
    /// when unchanged). Already transcribed media keep their transcripts.
    ///
    /// # Errors
    /// [`AppError::ProjectNotOpen`], [`AppError::Domain`] for invalid settings.
    pub async fn set_project_settings(
        &self,
        project_id: ProjectId,
        settings: TranscriptionSettings,
    ) -> Result<(), AppError> {
        let handle = self.handle(project_id)?;
        let current = handle.view().await?.project.settings.transcription.clone();
        if current == settings {
            return Ok(());
        }
        let transaction = Transaction {
            id: NewIds(self.deps().ids.as_ref()).transaction()?,
            actor: Actor::User,
            label: labels::SET_TRANSCRIPTION_SETTINGS.to_owned(),
            ops: vec![SetTranscriptionSettings { settings }.into()],
        };
        handle.apply(transaction).await.map(drop)
    }
}

/// The stored transcript of `asset_id` when it describes the content `content_key`.
pub(super) async fn current_transcript(
    handle: &ProjectActorHandle,
    asset_id: AssetId,
    content_key: &str,
) -> Result<Option<Transcript>, AppError> {
    let stored = handle.transcripts().load_transcript(asset_id).await?;
    Ok(stored
        .filter(|stored| stored.content_key == content_key)
        .map(|stored| stored.transcript))
}
