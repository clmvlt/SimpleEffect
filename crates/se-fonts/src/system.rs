//! The fonts installed on the system, read with `fontdb`.

use std::{
    collections::{BTreeMap, HashMap},
    path::Path,
};

use se_app::ports::FontData;
use se_protocol::{FontFace, FontFamily, FontSource};

use crate::face_id::system_face_id;

/// The installed fonts: families and where each face's file is.
#[derive(Debug, Default)]
pub struct SystemFonts {
    /// Families sorted by name (case-insensitive).
    pub families: Vec<FontFamily>,
    /// The file of each face, by id.
    pub files: HashMap<String, FontData>,
}

/// Reads every font installed on the system (blocking: call it on the blocking pool). Families
/// named in `skip` (the bundled ones) and hidden families (`.SF NS`…) are left out.
#[must_use]
pub fn load_system_fonts(skip: &[String]) -> SystemFonts {
    let mut database = fontdb::Database::new();
    database.load_system_fonts();
    let mut grouped: BTreeMap<String, (String, Vec<FontFace>)> = BTreeMap::new();
    let mut files = HashMap::new();
    for face in database.faces() {
        let path = match &face.source {
            fontdb::Source::File(path) | fontdb::Source::SharedFile(path, _) => path.clone(),
            fontdb::Source::Binary(_) => continue,
        };
        let Some(name) = family_name(face) else {
            continue;
        };
        if name.starts_with('.') || skip.contains(&name) {
            continue;
        }
        let Some((format, content_type)) = file_format(&path) else {
            continue;
        };
        let id = system_face_id(&path, face.index);
        let weight = face.weight.0.clamp(1, 1000);
        let entry = grouped
            .entry(name.to_lowercase())
            .or_insert_with(|| (name.clone(), Vec::new()));
        if entry.1.iter().any(|existing| existing.id == id) {
            continue;
        }
        entry.1.push(FontFace {
            id: id.clone(),
            weight_min: weight,
            weight_max: weight,
            italic: face.style != fontdb::Style::Normal,
            unicode_range: None,
            format: format.to_owned(),
        });
        files.insert(id, FontData::File { path, content_type });
    }
    let families = grouped
        .into_values()
        .map(|(name, mut faces)| {
            faces.sort_by_key(|face| (face.weight_min, face.italic));
            FontFamily {
                name,
                source: FontSource::System,
                faces,
            }
        })
        .collect();
    SystemFonts { families, files }
}

/// The face's family name in English (or its first name).
fn family_name(face: &fontdb::FaceInfo) -> Option<String> {
    face.families
        .iter()
        .find(|(_, language)| *language == fontdb::Language::English_UnitedStates)
        .or_else(|| face.families.first())
        .map(|(name, _)| name.trim().to_owned())
        .filter(|name| !name.is_empty())
}

/// The CSS format and MIME type of a font file, from its extension; `None` for other files.
fn file_format(path: &Path) -> Option<(&'static str, &'static str)> {
    let extension = path.extension()?.to_str()?.to_ascii_lowercase();
    Some(match extension.as_str() {
        "ttf" => ("truetype", "font/ttf"),
        "otf" => ("opentype", "font/otf"),
        "ttc" | "otc" => ("collection", "font/collection"),
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_come_from_the_extension() {
        assert_eq!(
            file_format(Path::new("a/Arial.TTF")),
            Some(("truetype", "font/ttf"))
        );
        assert_eq!(
            file_format(Path::new("a/b.otf")),
            Some(("opentype", "font/otf"))
        );
        assert_eq!(
            file_format(Path::new("a/Helvetica.ttc")),
            Some(("collection", "font/collection"))
        );
        assert_eq!(file_format(Path::new("a/font.woff")), None);
        assert_eq!(file_format(Path::new("a/font")), None);
    }

    #[test]
    fn system_fonts_load_without_the_skipped_families() {
        let fonts = load_system_fonts(&["Inter".to_owned()]);
        assert!(fonts.families.iter().all(|family| family.name != "Inter"));
        for family in &fonts.families {
            assert_ne!(family.faces.len(), 0, "{} has no face", family.name);
            for face in &family.faces {
                assert!(
                    fonts.files.contains_key(&face.id),
                    "{} has no file",
                    face.id
                );
            }
        }
    }
}
