//! The template library (prompt 25, ADR-0036): motion templates and caption styles saved once in
//! the app's data folder and reused in every project.
//!
//! - `plan`: pure planning in a project — the versioned copy of an entry (reused when the same
//!   version is already there), the values of new instances (brand kit roles, then explicit
//!   values), the update of a copy to a newer library version (values migrated, never automatic).
//! - `info`: checking names, descriptions and tags.
//! - `archive_import`: `.setemplate` files checked like an agent's module before they enter the
//!   library.
//! - `service`: [`TemplateLibraryService`], the user's use cases.
//!
//! Projects keep whole copies (`MotionTemplate.library` links them to their entry), so they open
//! and render without the library.

mod archive_import;
mod info;
mod plan;
mod service;

pub use info::{MAX_TAG_CHARS, MAX_TAGS, checked_info};
pub(crate) use plan::plan_copy;
pub use plan::{existing_copy, instance_values, project_copy};
pub use service::{TemplateLibraryDeps, TemplateLibraryService};

/// Labels of the library transactions (translated by the renderer's `history-label.ts`).
pub mod labels {
    /// A library template copied into the project and instantiated.
    pub const INSERT_TEMPLATE: &str = "Insert motion template";
    /// A project template replaced by a newer library version.
    pub const UPDATE_TEMPLATE: &str = "Update motion template";
    /// A project template linked to the entry it was saved as.
    pub const SAVE_TO_LIBRARY: &str = "Save template to library";
}
