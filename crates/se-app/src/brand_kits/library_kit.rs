//! Library kits as values: the defaults of a new kit, checks of a kit to save, and the project
//! copy a kit becomes once its logos are assets.

use se_domain::{
    BrandFonts, BrandKit, BrandKitId, BrandKitSource, BrandLogo, BrandPalette, CaptionStyleId,
    Color, DEFAULT_CAPTION_PRESET, MAX_BRAND_LOGOS, TextStyle, caption_style_from_preset,
};
use se_protocol::{LibraryBrandKit, LibraryLogo};

use crate::AppError;

/// Font of titles in a new kit (bundled).
pub const DEFAULT_HEADING_FONT: &str = "Montserrat";
/// Font of body text in a new kit (bundled).
pub const DEFAULT_BODY_FONT: &str = "Inter";

/// A new kit called `name`: Montserrat titles, Inter text, a blue, dark and yellow palette,
/// bold white texts and the `minimal` caption look in Inter.
///
/// # Errors
/// [`AppError::Internal`] if the embedded caption presets are broken (a bug).
pub fn new_library_kit(
    id: BrandKitId,
    name: &str,
    style_id: CaptionStyleId,
    updated_at: u64,
) -> Result<LibraryBrandKit, AppError> {
    let mut caption_style = caption_style_from_preset(DEFAULT_CAPTION_PRESET, style_id)
        .map_err(|error| AppError::Internal(error.to_string()))?;
    DEFAULT_BODY_FONT.clone_into(&mut caption_style.font_family);
    Ok(LibraryBrandKit {
        id,
        version: 1,
        updated_at,
        name: name.trim().to_owned(),
        fonts: BrandFonts {
            heading: DEFAULT_HEADING_FONT.to_owned(),
            body: DEFAULT_BODY_FONT.to_owned(),
        },
        colors: BrandPalette {
            primary: Color::opaque(0x2f, 0x6b, 0xff),
            secondary: Color::opaque(0x1a, 0x1a, 0x2e),
            accent: Color::opaque(0xff, 0xcc, 0x00),
            text: Color::WHITE,
            background: Color::opaque(0x11, 0x11, 0x11),
        },
        logos: Vec::new(),
        text_style: TextStyle {
            font_family: DEFAULT_HEADING_FONT.to_owned(),
            font_weight: 700,
            ..TextStyle::default()
        },
        caption_style,
        intro_templates: Vec::new(),
        outro_templates: Vec::new(),
    })
}

/// The project copy of `kit` whose logos became the assets `logos` (same order).
#[must_use]
pub fn project_kit(kit: &LibraryBrandKit, logos: Vec<BrandLogo>) -> BrandKit {
    BrandKit {
        source: Some(BrandKitSource {
            kit_id: kit.id,
            version: kit.version,
        }),
        name: kit.name.clone(),
        fonts: kit.fonts.clone(),
        colors: kit.colors,
        logos,
        text_style: kit.text_style.clone(),
        caption_style: kit.caption_style.clone(),
        intro_templates: kit.intro_templates.clone(),
        outro_templates: kit.outro_templates.clone(),
    }
}

/// Why `kit` cannot be saved, if anything: its values (as a project kit would be checked), its
/// logos' names and count, and logo files that the stored kit `stored_logos` does not hold (new
/// logos only come through `addBrandKitLogo`, which picks their file names).
///
/// # Errors
/// [`AppError::InvalidLibraryEntry`] naming each problem.
pub fn check_library_kit(
    kit: &LibraryBrandKit,
    stored_logos: &[LibraryLogo],
) -> Result<(), AppError> {
    let mut problems: Vec<String> = project_kit(kit, Vec::new())
        .problems()
        .into_iter()
        .map(|(field, rule)| format!("{field} {rule}"))
        .collect();
    if kit.logos.len() > MAX_BRAND_LOGOS {
        problems.push(format!("logos must hold at most {MAX_BRAND_LOGOS} logos"));
    }
    for logo in &kit.logos {
        if logo.name.trim().is_empty() {
            problems.push(format!("the logo {} needs a name", logo.file));
        }
        if !is_logo_file_name(&logo.file)
            || !stored_logos.iter().any(|stored| stored.file == logo.file)
        {
            problems.push(format!(
                "logo file {} is not part of this kit: add images with « Ajouter un logo »",
                logo.file
            ));
        }
    }
    if problems.is_empty() {
        Ok(())
    } else {
        Err(AppError::InvalidLibraryEntry {
            reason: format!(
                "brand kit « {} » cannot be saved: {}",
                kit.name,
                problems.join("; ")
            ),
        })
    }
}

/// Whether `name` is a logo file name a kit may hold: `logos/` then letters, digits, `-` and
/// `.` (no other folder, no `..`). Library adapters and the `/media` route refuse other names.
#[must_use]
pub fn is_logo_file_name(name: &str) -> bool {
    name.strip_prefix("logos/").is_some_and(|rest| {
        (1..=64).contains(&rest.len())
            && !rest.starts_with('.')
            && !rest.contains("..")
            && rest.chars().all(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '-' | '.')
            })
    })
}

/// The file name of a new logo copied from a file with `extension` (lower case, `png` when
/// absent or odd): `logos/<id>.<extension>`.
#[must_use]
pub fn logo_file_name(id: se_domain::Uuid, extension: Option<&str>) -> String {
    let extension = extension
        .map(str::to_ascii_lowercase)
        .filter(|extension| {
            !extension.is_empty()
                && extension.len() <= 5
                && extension
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric())
        })
        .unwrap_or_else(|| "png".to_owned());
    format!("logos/{}.{extension}", id.hyphenated())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kit() -> LibraryBrandKit {
        new_library_kit(
            BrandKitId::from_test_index(1),
            " Acme ",
            CaptionStyleId::from_test_index(1),
            0,
        )
        .unwrap()
    }

    #[test]
    fn new_kits_are_usable_and_copy_into_projects() {
        let kit = kit();
        assert_eq!(kit.name, "Acme");
        check_library_kit(&kit, &[]).unwrap();
        let logo = BrandLogo {
            name: "Logo".to_owned(),
            asset_id: se_domain::AssetId::from_test_index(3),
        };
        let copy = project_kit(&kit, vec![logo.clone()]);
        assert_eq!(copy.source.unwrap().kit_id, kit.id);
        assert_eq!(copy.logos, [logo]);
    }

    #[test]
    fn foreign_logo_files_and_empty_fonts_are_refused() {
        let mut kit = kit();
        kit.fonts.heading = String::new();
        kit.logos.push(LibraryLogo {
            name: "Logo".to_owned(),
            file: "../../secret.png".to_owned(),
        });
        let message = check_library_kit(&kit, &[]).unwrap_err().to_string();
        assert!(message.contains("fonts.heading"), "{message}");
        assert!(message.contains("not part of this kit"), "{message}");
    }

    #[test]
    fn only_plain_names_in_the_logo_folder_are_logo_files() {
        assert!(is_logo_file_name("logos/0190-ab.png"));
        assert!(!is_logo_file_name("logos/../brand-kit.json"));
        assert!(!is_logo_file_name("logos/sub/x.png"));
        assert!(!is_logo_file_name("other/x.png"));
        assert!(!is_logo_file_name("logos/.hidden"));
        assert!(!is_logo_file_name("logos/"));
    }

    #[test]
    fn logo_names_keep_a_safe_extension() {
        let id = se_domain::Uuid::nil();
        assert_eq!(
            logo_file_name(id, Some("PNG")),
            "logos/00000000-0000-0000-0000-000000000000.png"
        );
        assert_eq!(
            logo_file_name(id, Some("../x")),
            "logos/00000000-0000-0000-0000-000000000000.png"
        );
        assert_eq!(
            logo_file_name(id, None),
            "logos/00000000-0000-0000-0000-000000000000.png"
        );
    }
}
