//! Brand kits of the app's library (prompt 25, ADR-0036), stored in
//! `<app data>/brand-kits/<id>/`. A project uses a copy of one (`se_domain::BrandKit`); the
//! library kit holds its logos as files, the copy as image assets of the project.

use se_domain::{
    BrandFonts, BrandKitId, BrandPalette, CaptionStyle, LibraryTemplateRef, TextStyle,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A brand kit of the app's library.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct LibraryBrandKit {
    /// Stable id (project copies name it as their source).
    pub id: BrandKitId,
    /// Bumped by each save; project copies holding an older version are offered the update.
    pub version: u32,
    /// When it last changed (Unix milliseconds).
    pub updated_at: u64,
    /// Name shown in the interface.
    pub name: String,
    /// The two font roles.
    pub fonts: BrandFonts,
    /// The five colour roles.
    pub colors: BrandPalette,
    /// Logos, the first being the main one; their files live in the kit's folder (served at
    /// `/media/brand-kits/{kitId}/{file}`).
    pub logos: Vec<LibraryLogo>,
    /// Style of new texts (static values).
    pub text_style: TextStyle,
    /// Look of new caption styles.
    pub caption_style: CaptionStyle,
    /// Favourite intro templates of the library.
    pub intro_templates: Vec<LibraryTemplateRef>,
    /// Favourite outro templates of the library.
    pub outro_templates: Vec<LibraryTemplateRef>,
}

/// One logo of a library kit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct LibraryLogo {
    /// Name shown in the interface.
    pub name: String,
    /// The image's file name in the kit's folder (`logos/<uuid>.png`, chosen by the core).
    pub file: String,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn logos_are_files_of_the_kit() {
        let logo = LibraryLogo {
            name: "Logo blanc".to_owned(),
            file: "logos/0190.png".to_owned(),
        };
        assert_eq!(
            serde_json::to_value(logo).unwrap(),
            json!({ "name": "Logo blanc", "file": "logos/0190.png" })
        );
    }
}
