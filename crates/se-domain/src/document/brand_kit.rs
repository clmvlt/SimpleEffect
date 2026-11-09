//! [`BrandKit`]: the visual identity a project uses (prompt 25, ADR-0036).
//!
//! Kits live in the app's library (`<app data>/brand-kits/`); a project keeps a **copy** of the
//! kit it uses (with its library id and version in `source`) so it renders and behaves the same
//! on a machine without that library. New texts take `text_style`, new caption styles
//! `caption_style`, and template parameters bound to a [`BrandRole`] the matching value.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{Animatable, BrandRole, CaptionStyle, TemplateParamValue, TextStyle};
use crate::{AssetId, BrandKitId, LibraryTemplateId, color::Color};

/// Longest kit name, in characters.
pub const MAX_BRAND_KIT_NAME_CHARS: usize = 80;
/// Most logos a kit holds.
pub const MAX_BRAND_LOGOS: usize = 8;
/// Most favourite intro (or outro) templates a kit lists.
pub const MAX_BRAND_FAVORITES: usize = 8;

/// A brand identity: fonts, a named palette, logos and the default looks of new texts and
/// captions. Every value is static (no keyframes): it is a starting point, copied into what it
/// styles.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct BrandKit {
    /// The library kit this copy comes from, and its version then; `null` for a kit that exists
    /// only in this project (made before prompt 25, or whose library entry is gone).
    pub source: Option<BrandKitSource>,
    /// Name shown in the interface (« Acme », « Chaîne perso »).
    pub name: String,
    /// The two font roles.
    pub fonts: BrandFonts,
    /// The five colour roles.
    pub colors: BrandPalette,
    /// Logos, as image assets of the project (the first one is the main logo).
    pub logos: Vec<BrandLogo>,
    /// Style of new texts (static values).
    pub text_style: TextStyle,
    /// Look of new caption styles; its `id` is not used (new styles get their own).
    pub caption_style: CaptionStyle,
    /// Favourite intro templates of the library, most preferred first.
    pub intro_templates: Vec<LibraryTemplateRef>,
    /// Favourite outro templates of the library, most preferred first.
    pub outro_templates: Vec<LibraryTemplateRef>,
}

/// Where a project's kit comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct BrandKitSource {
    /// The kit of the app's library.
    pub kit_id: BrandKitId,
    /// Its version when it was copied (a newer library version is proposed, never imposed).
    pub version: u32,
}

/// Font families by role (names of the font catalogue).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct BrandFonts {
    /// Titles and headings.
    pub heading: String,
    /// Body text, captions, small print.
    pub body: String,
}

/// Colours by role.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct BrandPalette {
    /// Main brand colour.
    pub primary: Color,
    /// Second brand colour.
    pub secondary: Color,
    /// Highlights, calls to action.
    pub accent: Color,
    /// Text over the background colour.
    pub text: Color,
    /// Plates and backgrounds.
    pub background: Color,
}

/// One logo of a kit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct BrandLogo {
    /// Name shown in the interface (« Logo blanc »).
    pub name: String,
    /// The image asset of the project.
    pub asset_id: AssetId,
}

/// A template of the app's library, named so it can be shown without the library.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct LibraryTemplateRef {
    /// The library entry.
    pub id: LibraryTemplateId,
    /// Its name when it was chosen.
    pub name: String,
}

impl BrandPalette {
    /// The colour of a palette role (`None` for font and logo roles).
    #[must_use]
    pub const fn color(&self, role: BrandRole) -> Option<Color> {
        match role {
            BrandRole::Primary => Some(self.primary),
            BrandRole::Secondary => Some(self.secondary),
            BrandRole::Accent => Some(self.accent),
            BrandRole::Text => Some(self.text),
            BrandRole::Background => Some(self.background),
            BrandRole::Heading | BrandRole::Body | BrandRole::Logo => None,
        }
    }

    /// The five colours with their role, in palette order.
    #[must_use]
    pub const fn entries(&self) -> [(BrandRole, Color); 5] {
        [
            (BrandRole::Primary, self.primary),
            (BrandRole::Secondary, self.secondary),
            (BrandRole::Accent, self.accent),
            (BrandRole::Text, self.text),
            (BrandRole::Background, self.background),
        ]
    }
}

impl BrandKit {
    /// The value this kit gives a template parameter bound to `role`: a colour, a font family or
    /// the main logo (`None` when the kit has no logo).
    #[must_use]
    pub fn value_for(&self, role: BrandRole) -> Option<TemplateParamValue> {
        match role {
            BrandRole::Heading => Some(TemplateParamValue::Font {
                value: self.fonts.heading.clone(),
            }),
            BrandRole::Body => Some(TemplateParamValue::Font {
                value: self.fonts.body.clone(),
            }),
            BrandRole::Logo => self.logos.first().map(|logo| TemplateParamValue::Asset {
                value: Some(logo.asset_id),
            }),
            colour => self
                .colors
                .color(colour)
                .map(|color| TemplateParamValue::Color {
                    value: Animatable::constant(color),
                }),
        }
    }

    /// Whether `asset` is one of the kit's logos.
    #[must_use]
    pub fn uses_asset(&self, asset: AssetId) -> bool {
        self.logos.iter().any(|logo| logo.asset_id == asset)
    }

    /// What is wrong with the kit's own values, as `(JSON field path, rule)`; empty when it is
    /// usable. Logo references are checked against the project by validation.
    #[must_use]
    pub fn problems(&self) -> Vec<(String, String)> {
        let mut problems = Vec::new();
        let mut check = |ok: bool, field: &str, rule: &str| {
            if !ok {
                problems.push((field.to_owned(), rule.to_owned()));
            }
        };
        let name_chars = self.name.trim().chars().count();
        check(
            (1..=MAX_BRAND_KIT_NAME_CHARS).contains(&name_chars),
            "name",
            "must hold 1 to 80 characters",
        );
        check(
            !self.fonts.heading.trim().is_empty(),
            "fonts.heading",
            "must name a font",
        );
        check(
            !self.fonts.body.trim().is_empty(),
            "fonts.body",
            "must name a font",
        );
        check(
            self.logos.len() <= MAX_BRAND_LOGOS,
            "logos",
            "must hold at most 8 logos",
        );
        for (index, logo) in self.logos.iter().enumerate() {
            check(
                !logo.name.trim().is_empty(),
                &format!("logos[{index}].name"),
                "must not be empty",
            );
        }
        check(
            self.intro_templates.len() <= MAX_BRAND_FAVORITES
                && self.outro_templates.len() <= MAX_BRAND_FAVORITES,
            "introTemplates",
            "intro and outro favourites hold at most 8 templates each",
        );
        problems.extend(text_style_problems(&self.text_style));
        for (field, rule) in self.caption_style.problems() {
            problems.push((format!("captionStyle.{field}"), rule.to_owned()));
        }
        problems
    }
}

/// Problems of a kit's default text style: the static rules of texts, and no keyframes.
fn text_style_problems(style: &TextStyle) -> Vec<(String, String)> {
    let mut problems = Vec::new();
    let mut check = |ok: bool, field: &str, rule: &str| {
        if !ok {
            problems.push((format!("textStyle.{field}"), rule.to_owned()));
        }
    };
    check(
        !style.font_family.trim().is_empty(),
        "fontFamily",
        "must name a font",
    );
    check(
        TextStyle::WEIGHT_RANGE.contains(&style.font_weight),
        "fontWeight",
        "must be between 1 and 1000",
    );
    check(
        style.line_height > 0.0 && style.line_height <= 10.0,
        "lineHeight",
        "must be in (0, 10]",
    );
    let size = match &style.font_size {
        Animatable::Static { value } => value.0,
        Animatable::Keyframed { .. } => 0.0,
    };
    check(
        size.is_finite() && size > 0.0,
        "fontSize",
        "must be a static size above 0 px",
    );
    let keyframed = style.color.is_keyframed()
        || style.letter_spacing.is_keyframed()
        || style.stroke.color.is_keyframed()
        || style.stroke.width.is_keyframed()
        || style.shadow.color.is_keyframed()
        || style.shadow.blur.is_keyframed()
        || style.shadow.offset.is_keyframed()
        || style.background.color.is_keyframed();
    check(
        !keyframed,
        "",
        "must hold static values only (no keyframes): it is the starting look of new texts",
    );
    problems
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DesignPixels, Keyframe, test_support::example_project};

    fn kit() -> BrandKit {
        example_project().brand_kit.unwrap()
    }

    #[test]
    fn the_example_kit_is_usable() {
        assert_eq!(kit().problems(), Vec::<(String, String)>::new());
    }

    #[test]
    fn roles_give_colours_fonts_and_the_main_logo() {
        let kit = kit();
        assert_eq!(
            kit.value_for(BrandRole::Accent),
            Some(TemplateParamValue::Color {
                value: Animatable::constant(kit.colors.accent)
            })
        );
        assert_eq!(
            kit.value_for(BrandRole::Body),
            Some(TemplateParamValue::Font {
                value: kit.fonts.body.clone()
            })
        );
        assert_eq!(
            kit.value_for(BrandRole::Logo),
            Some(TemplateParamValue::Asset {
                value: Some(kit.logos[0].asset_id)
            })
        );
        let mut bare = kit;
        bare.logos.clear();
        assert_eq!(bare.value_for(BrandRole::Logo), None);
    }

    #[test]
    fn problems_name_each_faulty_field() {
        let mut kit = kit();
        kit.name = " ".to_owned();
        kit.fonts.body = String::new();
        kit.logos[0].name = String::new();
        kit.text_style.font_size = Animatable::Keyframed {
            keyframes: vec![Keyframe {
                time: crate::Flicks::ZERO,
                value: DesignPixels(40.0),
                easing: crate::Easing::Linear,
            }],
        };
        kit.caption_style.max_lines = 9;
        let fields: Vec<String> = kit.problems().into_iter().map(|(field, _)| field).collect();
        assert_eq!(
            fields,
            [
                "name",
                "fonts.body",
                "logos[0].name",
                "textStyle.fontSize",
                "captionStyle.maxLines"
            ]
        );
    }
}
