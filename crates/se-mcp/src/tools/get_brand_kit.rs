//! `get_brand_kit`: the project's brand kit (prompt 25) — fonts, named palette, logos, the look
//! of new texts and captions, favourite intro and outro templates — or why there is none.

use schemars::JsonSchema;
use se_domain::{Animatable, BrandKit, CaptionStyle, Color, TextStyle};
use serde::{Deserialize, Serialize};

use crate::{
    context::ToolContext,
    output::{ToolError, ToolOutput},
    registry::McpTool,
};

/// The tool.
pub struct GetBrandKit;

/// No arguments.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Input {}

#[derive(Debug, Serialize)]
struct Unavailable {
    available: bool,
    message: &'static str,
}

#[derive(Debug, Serialize)]
struct Kit {
    available: bool,
    name: String,
    fonts: Fonts,
    colors: Palette,
    logos: Vec<Logo>,
    text_style: TextLook,
    caption_style: CaptionLook,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    intro_templates: Vec<Favorite>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    outro_templates: Vec<Favorite>,
}

#[derive(Debug, Serialize)]
struct Fonts {
    heading: String,
    body: String,
}

#[derive(Debug, Serialize)]
struct Palette {
    primary: String,
    secondary: String,
    accent: String,
    text: String,
    background: String,
}

#[derive(Debug, Serialize)]
struct Logo {
    name: String,
    asset_id: String,
}

/// The static look of new texts, in `add_text` terms.
#[derive(Debug, Serialize)]
struct TextLook {
    font_family: String,
    font_weight: u16,
    font_size: f64,
    color: String,
    uppercase: bool,
    stroke: Option<String>,
    shadow: bool,
    background: Option<String>,
}

#[derive(Debug, Serialize)]
struct CaptionLook {
    font_family: String,
    text_color: String,
    active_color: String,
    animation: se_domain::CaptionAnimation,
}

#[derive(Debug, Serialize)]
struct Favorite {
    library_template_id: String,
    name: String,
}

impl McpTool for GetBrandKit {
    type Input = Input;

    const NAME: &'static str = "get_brand_kit";

    const DESCRIPTION: &'static str = "The project's brand kit: heading and body fonts, the named \
        palette (primary, secondary, accent, text, background), logos (image asset ids for \
        add_image or template asset parameters), the look new texts and captions take, and \
        favourite intro/outro library templates. Use it consistently unless the user asks \
        otherwise. Answers available=false when the project has none.";

    async fn call(context: ToolContext, _input: Input) -> Result<ToolOutput, ToolError> {
        let project = context.project().await?;
        match &project.brand_kit {
            Some(kit) => ToolOutput::json(&describe(kit)),
            None => ToolOutput::json(&Unavailable {
                available: false,
                message: "This project has no brand kit: follow the user's wording and the styles \
                          already in the timeline (get_item) for fonts and colours.",
            }),
        }
    }
}

fn describe(kit: &BrandKit) -> Kit {
    let colors = &kit.colors;
    Kit {
        available: true,
        name: kit.name.clone(),
        fonts: Fonts {
            heading: kit.fonts.heading.clone(),
            body: kit.fonts.body.clone(),
        },
        colors: Palette {
            primary: colors.primary.to_string(),
            secondary: colors.secondary.to_string(),
            accent: colors.accent.to_string(),
            text: colors.text.to_string(),
            background: colors.background.to_string(),
        },
        logos: kit
            .logos
            .iter()
            .map(|logo| Logo {
                name: logo.name.clone(),
                asset_id: logo.asset_id.to_string(),
            })
            .collect(),
        text_style: text_look(&kit.text_style),
        caption_style: caption_look(&kit.caption_style),
        intro_templates: favorites(&kit.intro_templates),
        outro_templates: favorites(&kit.outro_templates),
    }
}

fn text_look(style: &TextStyle) -> TextLook {
    TextLook {
        font_family: style.font_family.clone(),
        font_weight: style.font_weight,
        font_size: held(&style.font_size).map_or(96.0, |size| size.0),
        color: held(&style.color).map_or_else(String::new, |color| color.to_string()),
        uppercase: style.text_case == se_domain::TextCase::Uppercase,
        stroke: style
            .stroke
            .enabled
            .then(|| held(&style.stroke.color).map_or_else(String::new, |color| color.to_string())),
        shadow: style.shadow.enabled,
        background: style.background.enabled.then(|| {
            held(&style.background.color).map_or_else(String::new, |color: Color| color.to_string())
        }),
    }
}

fn caption_look(style: &CaptionStyle) -> CaptionLook {
    CaptionLook {
        font_family: style.font_family.clone(),
        text_color: style.text_color.to_string(),
        active_color: style.active_color.to_string(),
        animation: style.animation,
    }
}

fn favorites(templates: &[se_domain::LibraryTemplateRef]) -> Vec<Favorite> {
    templates
        .iter()
        .map(|template| Favorite {
            library_template_id: template.id.to_string(),
            name: template.name.clone(),
        })
        .collect()
}

/// The static value of a kit's style field (kits hold no keyframes).
fn held<T: Clone>(value: &Animatable<T>) -> Option<T> {
    value.held_value_at(se_domain::Flicks::ZERO)
}
