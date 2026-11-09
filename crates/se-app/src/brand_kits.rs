//! Brand kits (prompt 25, ADR-0036): fonts, a named palette, logos and default looks saved in the
//! app's library and copied into the projects that use them.
//!
//! - `library_kit`: defaults of a new kit, checks of a kit to save, the project copy of a kit.
//! - `service`: [`BrandKitService`] — the library's kits (create, save, logos, remove) and each
//!   project's copy (choose, update, add a project's own kit to the library).
//!
//! A project keeps a whole copy (`Project.brand_kit`, its logos imported as assets from copies in
//! `<project>/brand-kit/`), so it opens on another machine without the library. Saving a library
//! kit updates the open projects using it at once; other projects are offered the newer version
//! when they open (the renderer compares versions). New texts, caption styles and brand-bound
//! template values read the project's copy (`project_service/graphics.rs`, `captions/styles.rs`,
//! `template_library::instance_values`, se-mcp's `add_text`).

mod library_kit;
mod service;

pub use library_kit::{
    DEFAULT_BODY_FONT, DEFAULT_HEADING_FONT, check_library_kit, is_logo_file_name, logo_file_name,
    new_library_kit, project_kit,
};
pub use service::{BrandKitService, BrandKitServiceDeps, PROJECT_LOGO_FOLDER};

/// Labels of the brand kit transactions (translated by the renderer's `history-label.ts`).
pub mod labels {
    /// A kit chosen for the project.
    pub const CHOOSE_BRAND_KIT: &str = "Choose brand kit";
    /// The project's kit replaced by a newer version (or linked to the library).
    pub const UPDATE_BRAND_KIT: &str = "Update brand kit";
    /// The project's kit removed.
    pub const REMOVE_BRAND_KIT: &str = "Remove brand kit";
}
