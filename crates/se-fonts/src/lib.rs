//! # se-fonts
//!
//! The font adapter: implements [`se_app::ports::FontCatalog`] with the fonts **bundled** in the
//! binary (a few families under the SIL Open Font License or Apache 2.0, `fonts/`) and the fonts
//! **installed** on the system (read once with `fontdb`). se-core serves each face under
//! `GET /media/fonts/{id}`; the renderer loads them with `FontFace` (prompt 15).
//!
//! Allowed workspace dependencies: `se-app` (the port) and `se-protocol` (its DTOs).

mod bundled;
mod catalog;
mod face_id;
mod system;

pub use bundled::{BUNDLED_FACES, BundledFace};
pub use catalog::LocalFontCatalog;
