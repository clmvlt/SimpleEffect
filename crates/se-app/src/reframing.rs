//! Automatic reframing (prompt 26, ADR-0037): clips follow their subject from the faces found in
//! their media. Faces are an analysis of each media's content, run on request by a
//! `SubjectDetection` job (pictures sampled by ffmpeg, faces found by the detector, stored in the
//! project database); the geometry — shots, subject, window, smoothing, simplification — is the
//! domain's (`se_domain::tracking`), so a clip can be recomputed with other settings at once.
//!
//! - `service`: [`ReframeService`] (reframe clips, versions with reframing, the faces of a media
//!   for the viewer, waits for agents).
//! - `plan`: which clips can follow a subject, and their tracked reframes (pure).
//! - `sampling`: picture size, interval and source of the sampling (pure).
//! - `readiness`: where an asset's analyses stand.
//! - `detection_job`: the job sampling pictures and detecting faces.

mod detection_job;
mod plan;
mod readiness;
mod sampling;
mod service;

/// Labels of the transactions built here (translated by the renderer's `history-label.ts`).
pub mod labels {
    /// Clips made to follow their subject.
    pub const AUTO_REFRAME: &str = "Auto reframe";
}

pub use self::{
    plan::{
        AssetAnalyses, ReframableClip, reframe_operations, select_clips, tracked_reframe,
        video_clips,
    },
    readiness::AnalysisState,
    sampling::{SAMPLE_INTERVAL, SAMPLE_LONGEST_SIDE, sample_request, sample_size},
    service::{AnalysesRound, ReframeService, ReframeServiceDeps},
};
