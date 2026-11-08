//! Smart cuts (prompt 22, ADR-0032): text-based editing, filler words and silence removal, for
//! the user and for AI agents through the same planning.
//!
//! A cut is planned in three steps, each pure:
//! 1. **resolve** ([`resolve`]): what to remove, from the sequence as it is — words mapped
//!    through the current clips (`se_domain::sequence_transcript`) with natural margins
//!    ([`words`]), fillers found in them ([`fillers`]), or silences read from the assets' level
//!    envelopes ([`silences`]) — turned into frame-aligned ranges by [`plan::plan_cuts`];
//! 2. **tracks** ([`apply::cut_tracks`]): the speech's tracks and their linked partners (or every
//!    unlocked track);
//! 3. **plan** ([`apply::plan_cut`]): one `RippleDeleteRanges` and the links of the new parts.
//!
//! The result is one transaction of ordinary items: undone in one step, adjustable by hand.
//! [`SmartCutService`] runs user cuts; `AgentEditService::apply_cuts` runs agents' cuts with
//! the « cut » scope rule.

pub mod apply;
pub mod fillers;
pub mod plan;
mod request;
pub mod resolve;
pub mod service;
pub mod settings;
mod silence_cuts;
pub mod silences;
mod word_context;
pub mod words;

use std::collections::BTreeSet;

pub use request::{CutMedia, CutRequest, CutTarget, ResolvedCuts};
use se_domain::{Flicks, Project, TrackId};
pub use service::{SilenceCut, SmartCutService, SmartCutServiceDeps};

use crate::{AppError, edit_plan::find_sequence};

/// Labels of the transactions of smart cuts (translated by the renderer's `history-label.ts`).
pub mod labels {
    /// Words removed in the transcript.
    pub const REMOVE_WORDS: &str = "Remove words";
    /// Filler words removed.
    pub const REMOVE_FILLERS: &str = "Remove fillers";
    /// Silences removed.
    pub const REMOVE_SILENCES: &str = "Remove silences";
}

/// `request` resolved against `project`, with the tracks its cuts edit.
///
/// # Errors
/// Unknown sequence or tracks; a locked track holding the speech while there is something to cut.
pub fn resolve_with_tracks(
    project: &Project,
    request: &CutRequest,
    media: &CutMedia,
) -> Result<(ResolvedCuts, BTreeSet<TrackId>), AppError> {
    let resolved = resolve::resolve(project, request, media)?;
    let sequence = find_sequence(project, request.sequence_id)?;
    if !resolved.cuts.is_empty()
        && let Some(locked) = sequence
            .tracks
            .iter()
            .find(|track| track.locked && resolved.seeds.contains(&track.id))
    {
        return Err(AppError::InvalidEdit {
            reason: format!(
                "track {} is locked: unlock it to cut its words or silences",
                locked.name
            ),
        });
    }
    let from = resolved
        .cuts
        .first()
        .map_or(Flicks::ZERO, |cut| cut.start());
    let tracks = apply::cut_tracks(sequence, &resolved.seeds, from, request.all_tracks);
    Ok((resolved, tracks))
}
