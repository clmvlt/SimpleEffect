//! The fonts shipped with SimpleEffect: the latin and latin-ext subsets (WOFF2) of a few families
//! from the Fontsource packages (`fonts/<family>/`, each with its licence), embedded in the
//! binary so they exist on every machine and need no packaging step.
//!
//! To add a family: copy its `*-latin-*.woff2` and `*-latin-ext-*.woff2` files and `LICENSE`
//! (OFL or Apache 2.0) into `fonts/<family>/`, then add one [`face`] line per file with the
//! weights, style and `unicode-range` of the package's CSS.

use se_protocol::{FontFace, FontFamily, FontSource};

/// A face embedded in the binary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BundledFace {
    /// Family name, as texts name it.
    pub family: &'static str,
    /// Path under `fonts/` (also its id, without the extension).
    pub path: &'static str,
    /// The WOFF2 file.
    pub bytes: &'static [u8],
    /// Lightest weight drawn.
    pub weight_min: u16,
    /// Heaviest weight drawn (variable fonts cover a range).
    pub weight_max: u16,
    /// Italic face.
    pub italic: bool,
    /// The CSS `unicode-range` of the subset.
    pub unicode_range: &'static str,
}

impl BundledFace {
    /// The face id served by `/media/fonts/{id}`: `bundled-` and the file name without extension.
    #[must_use]
    pub fn id(&self) -> String {
        let file = self.path.rsplit('/').next().unwrap_or(self.path);
        format!("bundled-{}", file.trim_end_matches(".woff2"))
    }
}

#[allow(
    clippy::too_many_arguments,
    reason = "one line per face reads like the CSS it mirrors"
)]
const fn face(
    family: &'static str,
    path: &'static str,
    bytes: &'static [u8],
    weight_min: u16,
    weight_max: u16,
    italic: bool,
    unicode_range: &'static str,
) -> BundledFace {
    BundledFace {
        family,
        path,
        bytes,
        weight_min,
        weight_max,
        italic,
        unicode_range,
    }
}

/// Every bundled face. Inter is the default font of new texts.
pub const BUNDLED_FACES: &[BundledFace] = &[
    face(
        "Inter",
        "inter/inter-latin-ext-wght-normal.woff2",
        include_bytes!("../fonts/inter/inter-latin-ext-wght-normal.woff2"),
        100,
        900,
        false,
        "U+0100-02BA,U+02BD-02C5,U+02C7-02CC,U+02CE-02D7,U+02DD-02FF,U+0304,U+0308,U+0329,U+1D00-1DBF,U+1E00-1E9F,U+1EF2-1EFF,U+2020,U+20A0-20AB,U+20AD-20C0,U+2113,U+2C60-2C7F,U+A720-A7FF",
    ),
    face(
        "Inter",
        "inter/inter-latin-wght-normal.woff2",
        include_bytes!("../fonts/inter/inter-latin-wght-normal.woff2"),
        100,
        900,
        false,
        "U+0000-00FF,U+0131,U+0152-0153,U+02BB-02BC,U+02C6,U+02DA,U+02DC,U+0304,U+0308,U+0329,U+2000-206F,U+20AC,U+2122,U+2191,U+2193,U+2212,U+2215,U+FEFF,U+FFFD",
    ),
    face(
        "Inter",
        "inter/inter-latin-ext-wght-italic.woff2",
        include_bytes!("../fonts/inter/inter-latin-ext-wght-italic.woff2"),
        100,
        900,
        true,
        "U+0100-02BA,U+02BD-02C5,U+02C7-02CC,U+02CE-02D7,U+02DD-02FF,U+0304,U+0308,U+0329,U+1D00-1DBF,U+1E00-1E9F,U+1EF2-1EFF,U+2020,U+20A0-20AB,U+20AD-20C0,U+2113,U+2C60-2C7F,U+A720-A7FF",
    ),
    face(
        "Inter",
        "inter/inter-latin-wght-italic.woff2",
        include_bytes!("../fonts/inter/inter-latin-wght-italic.woff2"),
        100,
        900,
        true,
        "U+0000-00FF,U+0131,U+0152-0153,U+02BB-02BC,U+02C6,U+02DA,U+02DC,U+0304,U+0308,U+0329,U+2000-206F,U+20AC,U+2122,U+2191,U+2193,U+2212,U+2215,U+FEFF,U+FFFD",
    ),
    face(
        "Montserrat",
        "montserrat/montserrat-latin-ext-wght-normal.woff2",
        include_bytes!("../fonts/montserrat/montserrat-latin-ext-wght-normal.woff2"),
        100,
        900,
        false,
        "U+0100-02BA,U+02BD-02C5,U+02C7-02CC,U+02CE-02D7,U+02DD-02FF,U+0304,U+0308,U+0329,U+1D00-1DBF,U+1E00-1E9F,U+1EF2-1EFF,U+2020,U+20A0-20AB,U+20AD-20C0,U+2113,U+2C60-2C7F,U+A720-A7FF",
    ),
    face(
        "Montserrat",
        "montserrat/montserrat-latin-wght-normal.woff2",
        include_bytes!("../fonts/montserrat/montserrat-latin-wght-normal.woff2"),
        100,
        900,
        false,
        "U+0000-00FF,U+0131,U+0152-0153,U+02BB-02BC,U+02C6,U+02DA,U+02DC,U+0304,U+0308,U+0329,U+2000-206F,U+20AC,U+2122,U+2191,U+2193,U+2212,U+2215,U+FEFF,U+FFFD",
    ),
    face(
        "Montserrat",
        "montserrat/montserrat-latin-ext-wght-italic.woff2",
        include_bytes!("../fonts/montserrat/montserrat-latin-ext-wght-italic.woff2"),
        100,
        900,
        true,
        "U+0100-02BA,U+02BD-02C5,U+02C7-02CC,U+02CE-02D7,U+02DD-02FF,U+0304,U+0308,U+0329,U+1D00-1DBF,U+1E00-1E9F,U+1EF2-1EFF,U+2020,U+20A0-20AB,U+20AD-20C0,U+2113,U+2C60-2C7F,U+A720-A7FF",
    ),
    face(
        "Montserrat",
        "montserrat/montserrat-latin-wght-italic.woff2",
        include_bytes!("../fonts/montserrat/montserrat-latin-wght-italic.woff2"),
        100,
        900,
        true,
        "U+0000-00FF,U+0131,U+0152-0153,U+02BB-02BC,U+02C6,U+02DA,U+02DC,U+0304,U+0308,U+0329,U+2000-206F,U+20AC,U+2122,U+2191,U+2193,U+2212,U+2215,U+FEFF,U+FFFD",
    ),
    face(
        "Oswald",
        "oswald/oswald-latin-ext-wght-normal.woff2",
        include_bytes!("../fonts/oswald/oswald-latin-ext-wght-normal.woff2"),
        200,
        700,
        false,
        "U+0100-02BA,U+02BD-02C5,U+02C7-02CC,U+02CE-02D7,U+02DD-02FF,U+0304,U+0308,U+0329,U+1D00-1DBF,U+1E00-1E9F,U+1EF2-1EFF,U+2020,U+20A0-20AB,U+20AD-20C0,U+2113,U+2C60-2C7F,U+A720-A7FF",
    ),
    face(
        "Oswald",
        "oswald/oswald-latin-wght-normal.woff2",
        include_bytes!("../fonts/oswald/oswald-latin-wght-normal.woff2"),
        200,
        700,
        false,
        "U+0000-00FF,U+0131,U+0152-0153,U+02BB-02BC,U+02C6,U+02DA,U+02DC,U+0304,U+0308,U+0329,U+2000-206F,U+20AC,U+2122,U+2191,U+2193,U+2212,U+2215,U+FEFF,U+FFFD",
    ),
    face(
        "Playfair Display",
        "playfair-display/playfair-display-latin-ext-wght-normal.woff2",
        include_bytes!("../fonts/playfair-display/playfair-display-latin-ext-wght-normal.woff2"),
        400,
        900,
        false,
        "U+0100-02BA,U+02BD-02C5,U+02C7-02CC,U+02CE-02D7,U+02DD-02FF,U+0304,U+0308,U+0329,U+1D00-1DBF,U+1E00-1E9F,U+1EF2-1EFF,U+2020,U+20A0-20AB,U+20AD-20C0,U+2113,U+2C60-2C7F,U+A720-A7FF",
    ),
    face(
        "Playfair Display",
        "playfair-display/playfair-display-latin-wght-normal.woff2",
        include_bytes!("../fonts/playfair-display/playfair-display-latin-wght-normal.woff2"),
        400,
        900,
        false,
        "U+0000-00FF,U+0131,U+0152-0153,U+02BB-02BC,U+02C6,U+02DA,U+02DC,U+0304,U+0308,U+0329,U+2000-206F,U+20AC,U+2122,U+2191,U+2193,U+2212,U+2215,U+FEFF,U+FFFD",
    ),
    face(
        "Playfair Display",
        "playfair-display/playfair-display-latin-ext-wght-italic.woff2",
        include_bytes!("../fonts/playfair-display/playfair-display-latin-ext-wght-italic.woff2"),
        400,
        900,
        true,
        "U+0100-02BA,U+02BD-02C5,U+02C7-02CC,U+02CE-02D7,U+02DD-02FF,U+0304,U+0308,U+0329,U+1D00-1DBF,U+1E00-1E9F,U+1EF2-1EFF,U+2020,U+20A0-20AB,U+20AD-20C0,U+2113,U+2C60-2C7F,U+A720-A7FF",
    ),
    face(
        "Playfair Display",
        "playfair-display/playfair-display-latin-wght-italic.woff2",
        include_bytes!("../fonts/playfair-display/playfair-display-latin-wght-italic.woff2"),
        400,
        900,
        true,
        "U+0000-00FF,U+0131,U+0152-0153,U+02BB-02BC,U+02C6,U+02DA,U+02DC,U+0304,U+0308,U+0329,U+2000-206F,U+20AC,U+2122,U+2191,U+2193,U+2212,U+2215,U+FEFF,U+FFFD",
    ),
    face(
        "Bebas Neue",
        "bebas-neue/bebas-neue-latin-ext-400-normal.woff2",
        include_bytes!("../fonts/bebas-neue/bebas-neue-latin-ext-400-normal.woff2"),
        400,
        400,
        false,
        "U+0100-02BA,U+02BD-02C5,U+02C7-02CC,U+02CE-02D7,U+02DD-02FF,U+0304,U+0308,U+0329,U+1D00-1DBF,U+1E00-1E9F,U+1EF2-1EFF,U+2020,U+20A0-20AB,U+20AD-20C0,U+2113,U+2C60-2C7F,U+A720-A7FF",
    ),
    face(
        "Bebas Neue",
        "bebas-neue/bebas-neue-latin-400-normal.woff2",
        include_bytes!("../fonts/bebas-neue/bebas-neue-latin-400-normal.woff2"),
        400,
        400,
        false,
        "U+0000-00FF,U+0131,U+0152-0153,U+02BB-02BC,U+02C6,U+02DA,U+02DC,U+0304,U+0308,U+0329,U+2000-206F,U+20AC,U+2122,U+2191,U+2193,U+2212,U+2215,U+FEFF,U+FFFD",
    ),
    face(
        "Permanent Marker",
        "permanent-marker/permanent-marker-latin-400-normal.woff2",
        include_bytes!("../fonts/permanent-marker/permanent-marker-latin-400-normal.woff2"),
        400,
        400,
        false,
        "U+0000-00FF,U+0131,U+0152-0153,U+02BB-02BC,U+02C6,U+02DA,U+02DC,U+0304,U+0308,U+0329,U+2000-206F,U+20AC,U+2122,U+2191,U+2193,U+2212,U+2215,U+FEFF,U+FFFD",
    ),
];

/// The bundled families, sorted by name, their faces lightest first.
#[must_use]
pub fn bundled_families() -> Vec<FontFamily> {
    let mut families: Vec<FontFamily> = Vec::new();
    for bundled in BUNDLED_FACES {
        let face = FontFace {
            id: bundled.id(),
            weight_min: bundled.weight_min,
            weight_max: bundled.weight_max,
            italic: bundled.italic,
            unicode_range: Some(bundled.unicode_range.to_owned()),
            format: "woff2".to_owned(),
        };
        match families
            .iter_mut()
            .find(|family| family.name == bundled.family)
        {
            Some(family) => family.faces.push(face),
            None => families.push(FontFamily {
                name: bundled.family.to_owned(),
                source: FontSource::Bundled,
                faces: vec![face],
            }),
        }
    }
    for family in &mut families {
        family
            .faces
            .sort_by_key(|face| (face.weight_min, face.italic));
    }
    families.sort_by(|left, right| left.name.cmp(&right.name));
    families
}

/// The bundled face with this id.
#[must_use]
pub fn bundled_face(id: &str) -> Option<&'static BundledFace> {
    BUNDLED_FACES.iter().find(|face| face.id() == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_bundled_face_is_a_woff2_file_with_a_unique_id() {
        let mut ids = std::collections::HashSet::new();
        for face in BUNDLED_FACES {
            assert!(
                face.bytes.starts_with(b"wOF2"),
                "{} is not WOFF2",
                face.path
            );
            assert!(ids.insert(face.id()), "{} is listed twice", face.id());
            assert!(face.weight_min <= face.weight_max);
            assert!(face.unicode_range.starts_with("U+"));
        }
    }

    #[test]
    fn inter_is_bundled_upright_and_italic() {
        let families = bundled_families();
        let inter = families
            .iter()
            .find(|family| family.name == "Inter")
            .unwrap();
        assert_eq!(inter.source, FontSource::Bundled);
        assert!(inter.faces.iter().any(|face| face.italic));
        assert!(
            inter
                .faces
                .iter()
                .all(|face| face.weight_min == 100 && face.weight_max == 900)
        );
        assert_eq!(
            bundled_face("bundled-inter-latin-wght-normal").map(|face| face.family),
            Some("Inter")
        );
    }
}
