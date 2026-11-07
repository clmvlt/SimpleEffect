//! [`AppError`]: why a use case failed, and its protocol code.

use std::path::PathBuf;

use se_domain::{AssetId, AssetKind, DomainError, DomainErrorCode, Flicks, JobId, ProjectId};
use se_protocol::{ErrorCode, ProtocolError};

use crate::ports::{MediaError, StorageError};

/// A failed use case. Messages are actionable: they reach users and AI agents unchanged.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum AppError {
    /// A persistence port failed.
    #[error(transparent)]
    Storage(#[from] StorageError),
    /// The project is not open in this core.
    #[error("project {project_id} is not open: open it first (openProject)")]
    ProjectNotOpen {
        /// The project.
        project_id: ProjectId,
    },
    /// Another folder holding a project with the same id is open (a copy made outside the app).
    #[error(
        "{} holds project {project_id}, which is already open from {}: it is probably a copy \
         made outside SimpleEffect; close the other one first, or use « Duplicate » to make copies",
        path.display(),
        open_path.display()
    )]
    SameProjectOpen {
        /// The project id both folders hold.
        project_id: ProjectId,
        /// The folder being opened.
        path: PathBuf,
        /// The folder already open.
        open_path: PathBuf,
    },
    /// A name was empty.
    #[error("the {what} name must not be empty: type a name")]
    EmptyName {
        /// What was being named (`project`, `sequence`).
        what: &'static str,
    },
    /// A sequence format breaks a document rule.
    #[error("invalid format: {details}")]
    InvalidFormat {
        /// The violated rules, as validation messages.
        details: String,
    },
    /// The journal cannot be replayed on top of the snapshot.
    #[error(
        "the journal of {} cannot be replayed: {reason}; project.json and its backups \
         (project.json.bak.1–3) are intact",
        path.display()
    )]
    JournalBroken {
        /// The project folder.
        path: PathBuf,
        /// Which entry failed and why.
        reason: String,
    },
    /// The loaded document breaks document invariants (e.g. edited by hand).
    #[error(
        "{} holds an invalid document: {details}; restore a backup (project.json.bak.1–3) or fix \
         the file",
        path.display()
    )]
    InvalidDocument {
        /// The project folder.
        path: PathBuf,
        /// The violated rules.
        details: String,
    },
    /// The document refused the change.
    #[error(transparent)]
    Domain(#[from] DomainError),
    /// The project's actor stopped while the command was waiting for it.
    #[error("project {project_id} was closed while the command was running: open it again")]
    ActorStopped {
        /// The project.
        project_id: ProjectId,
    },
    /// A media port failed (file missing, unreadable, ffmpeg absent…).
    #[error(transparent)]
    Media(#[from] MediaError),
    /// The open project has no such asset.
    #[error("project {project_id} has no asset {asset_id}: refresh the media bin")]
    AssetNotFound {
        /// The project.
        project_id: ProjectId,
        /// The asset.
        asset_id: AssetId,
    },
    /// A replacement file holds another kind of media than the asset it should replace.
    #[error(
        "{} is {found:?} media but the asset is {expected:?} media: pick a file of the same kind",
        path.display()
    )]
    MediaKindMismatch {
        /// The replacement file.
        path: PathBuf,
        /// Kind of the asset.
        expected: AssetKind,
        /// Kind of the file.
        found: AssetKind,
    },
    /// No active job has this id.
    #[error("no queued or running job {job_id}: it may have finished already")]
    JobNotFound {
        /// The job.
        job_id: JobId,
    },
    /// A mezzanine or proxy was requested for an asset it does not apply to.
    #[error("asset {asset_id} gets no {derived}: {reason}")]
    DerivedNotApplicable {
        /// The asset.
        asset_id: AssetId,
        /// `mezzanine` or `proxy`.
        derived: &'static str,
        /// Why, and what to do instead.
        reason: String,
    },
    /// A time outside an asset's media.
    #[error(
        "{:.3} s is outside asset {asset_id}, which lasts {:.3} s: pick a time between 0 and its end",
        time.to_seconds_f64(),
        duration.to_seconds_f64()
    )]
    MediaTimeOutOfRange {
        /// The asset.
        asset_id: AssetId,
        /// The time asked for.
        time: Flicks,
        /// Its media's duration.
        duration: Flicks,
    },
    /// A setting value is not accepted.
    #[error("invalid setting {name}: {reason}")]
    InvalidSetting {
        /// The setting.
        name: &'static str,
        /// What is accepted.
        reason: String,
    },
    /// An item edit that cannot be done as asked (nothing listed, several sequences, a gap that
    /// is not there any more…).
    #[error("{reason}")]
    InvalidEdit {
        /// What is wrong and what to do instead.
        reason: String,
    },
    /// A ripple edit would cut content of a track it must shift.
    #[error(
        "this ripple edit would also cut what track {track_name} holds between {:.3} s and \
         {:.3} s (tracks holding linked items shift together so they stay in sync): move or trim \
         that content first, or lock track {track_name} to leave it out of ripple edits",
        start.to_seconds_f64(),
        end.to_seconds_f64()
    )]
    RippleBlocked {
        /// Name of the track.
        track_name: String,
        /// Start of the content that would be cut.
        start: Flicks,
        /// Its end.
        end: Flicks,
    },
    /// No listed cut has media to cross-fade.
    #[error(
        "there is no media to cross-fade at this cut: both items need media beyond their edges          (trim them to free some), must last long enough, and the cut must not already have a          transition"
    )]
    TransitionNoRoom,
    /// A bug: an invariant of the application itself was broken.
    #[error("internal error: {0}; please report it")]
    Internal(String),
}

impl AppError {
    /// The stable protocol code of this error.
    #[must_use]
    pub fn code(&self) -> ErrorCode {
        match self {
            Self::Storage(error) => storage_code(error),
            Self::ProjectNotOpen { .. } | Self::ActorStopped { .. } => ErrorCode::ProjectNotOpen,
            Self::SameProjectOpen { .. } => ErrorCode::ProjectLocked,
            Self::EmptyName { .. } | Self::InvalidFormat { .. } | Self::InvalidSetting { .. } => {
                ErrorCode::InvalidRequest
            }
            Self::DerivedNotApplicable { .. } => ErrorCode::DerivedNotApplicable,
            Self::MediaTimeOutOfRange { .. } => ErrorCode::MediaTimeOutOfRange,
            Self::JournalBroken { .. } | Self::InvalidDocument { .. } => {
                ErrorCode::ProjectCorrupted
            }
            Self::Domain(error) => match error.code() {
                DomainErrorCode::NothingToUndo => ErrorCode::NothingToUndo,
                DomainErrorCode::NothingToRedo => ErrorCode::NothingToRedo,
                _ => ErrorCode::EditRejected,
            },
            Self::Media(error) => media_code(error),
            Self::AssetNotFound { .. } => ErrorCode::AssetNotFound,
            Self::MediaKindMismatch { .. } => ErrorCode::MediaKindMismatch,
            Self::JobNotFound { .. } => ErrorCode::JobNotFound,
            Self::InvalidEdit { .. } => ErrorCode::EditRejected,
            Self::RippleBlocked { .. } => ErrorCode::RippleBlocked,
            Self::TransitionNoRoom => ErrorCode::TransitionNoRoom,
            Self::Internal(_) => ErrorCode::Internal,
        }
    }
}

/// The protocol code of a media port failure (also used per file in import reports).
#[must_use]
pub(crate) fn media_code(error: &MediaError) -> ErrorCode {
    match error {
        MediaError::NotFound { .. } => ErrorCode::MediaNotFound,
        MediaError::Unsupported { .. } => ErrorCode::MediaUnsupported,
        MediaError::ToolsMissing { .. } => ErrorCode::MediaToolsMissing,
        MediaError::Io { .. } => ErrorCode::StorageFailed,
        MediaError::Failed(_) => ErrorCode::Internal,
    }
}

fn storage_code(error: &StorageError) -> ErrorCode {
    match error {
        StorageError::NotFound { .. } => ErrorCode::ProjectNotFound,
        StorageError::Locked { .. } => ErrorCode::ProjectLocked,
        StorageError::AlreadyExists { .. } => ErrorCode::ProjectAlreadyExists,
        StorageError::SchemaTooNew { .. } => ErrorCode::SchemaTooNew,
        StorageError::Corrupted { .. } => ErrorCode::ProjectCorrupted,
        StorageError::Io { .. } => ErrorCode::StorageFailed,
    }
}

impl From<AppError> for ProtocolError {
    fn from(error: AppError) -> Self {
        let protocol = Self::new(error.code(), error.to_string());
        match &error {
            // `{"domainCode":"OVERLAPPING_ITEMS"}`: clients and agents can match the domain code.
            AppError::Domain(domain) => {
                protocol.with_detail("domainCode", domain.code().to_string())
            }
            _ => protocol,
        }
    }
}

#[cfg(test)]
mod tests {
    use se_domain::ItemId;

    use super::*;

    #[test]
    fn storage_errors_map_to_project_codes() {
        let locked = AppError::from(StorageError::Locked {
            path: PathBuf::from("Film.seproj"),
        });
        assert_eq!(locked.code(), ErrorCode::ProjectLocked);
        assert!(locked.to_string().contains("Film.seproj"));
    }

    #[test]
    fn domain_errors_carry_their_domain_code() {
        let error = AppError::from(DomainError::ItemNotFound {
            item: ItemId::from_test_index(9),
        });
        let protocol = ProtocolError::from(error);
        assert_eq!(protocol.code, ErrorCode::EditRejected);
        let details = protocol.details.unwrap();
        assert_eq!(details["domainCode"], "ITEM_NOT_FOUND");
    }

    #[test]
    fn nothing_to_undo_keeps_its_own_code() {
        assert_eq!(
            AppError::from(DomainError::NothingToUndo).code(),
            ErrorCode::NothingToUndo
        );
    }
}
