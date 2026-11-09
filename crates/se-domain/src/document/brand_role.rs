//! [`BrandRole`]: a value of a brand kit a template parameter can be bound to (prompt 25).

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A role of a brand kit: a palette colour, a font or the main logo.
///
/// A motion template parameter declares one (`brand: 'accent'` in its schema) so instances made
/// in a project with a kit start with the kit's value instead of the schema default: one library
/// template then fits every brand.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum BrandRole {
    /// Main brand colour.
    Primary,
    /// Second brand colour.
    Secondary,
    /// Highlight colour.
    Accent,
    /// Text colour.
    Text,
    /// Background colour.
    Background,
    /// Font of titles.
    Heading,
    /// Font of body text.
    Body,
    /// The main logo (an image).
    Logo,
}

impl BrandRole {
    /// Every role, in palette order then fonts then logo.
    pub const ALL: [Self; 8] = [
        Self::Primary,
        Self::Secondary,
        Self::Accent,
        Self::Text,
        Self::Background,
        Self::Heading,
        Self::Body,
        Self::Logo,
    ];

    /// The name written in schemas and JSON (`accent`).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Primary => "primary",
            Self::Secondary => "secondary",
            Self::Accent => "accent",
            Self::Text => "text",
            Self::Background => "background",
            Self::Heading => "heading",
            Self::Body => "body",
            Self::Logo => "logo",
        }
    }

    /// The role called `name`, if any.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|role| role.name() == name)
    }

    /// The parameter kind the role fits (`color`, `font` or `asset`, as `TemplateParamSpec`
    /// kind names).
    #[must_use]
    pub const fn param_kind(self) -> &'static str {
        match self {
            Self::Primary | Self::Secondary | Self::Accent | Self::Text | Self::Background => {
                "color"
            }
            Self::Heading | Self::Body => "font",
            Self::Logo => "asset",
        }
    }

    /// The roles fitting a parameter kind, comma-separated (for messages).
    #[must_use]
    pub fn names_for_kind(kind: &str) -> String {
        Self::ALL
            .into_iter()
            .filter(|role| role.param_kind() == kind)
            .map(Self::name)
            .collect::<Vec<_>>()
            .join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_round_trip_and_match_serde() {
        for role in BrandRole::ALL {
            assert_eq!(BrandRole::from_name(role.name()), Some(role));
            assert_eq!(
                serde_json::to_string(&role).unwrap(),
                format!("\"{}\"", role.name())
            );
        }
        assert_eq!(BrandRole::from_name("tertiary"), None);
    }

    #[test]
    fn roles_fit_one_kind_each() {
        assert_eq!(
            BrandRole::names_for_kind("color"),
            "primary, secondary, accent, text, background"
        );
        assert_eq!(BrandRole::names_for_kind("font"), "heading, body");
        assert_eq!(BrandRole::names_for_kind("asset"), "logo");
    }
}
