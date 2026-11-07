//! What clients see of an open project: normalised entities, patches and undo state
//! (`docs/architecture.md` §5.6). Pure conversions from the document; no IO.

mod patch;
mod snapshot;

pub use patch::project_patch;
use se_domain::History;
use se_protocol::HistoryState;
pub use snapshot::project_snapshot;

/// What undo and redo would do now.
#[must_use]
pub fn history_state(history: &History) -> HistoryState {
    HistoryState {
        can_undo: history.can_undo(),
        can_redo: history.can_redo(),
        undo_label: history.undo_label().map(str::to_owned),
        redo_label: history.redo_label().map(str::to_owned),
    }
}
