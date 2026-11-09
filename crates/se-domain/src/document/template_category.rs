//! [`TemplateCategory`]: where a motion template is filed in the template browser (prompt 25).

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// The kind of graphic a motion template draws, declared by its module (`meta.category`) and
/// used by the template browser's categories. Caption styles of the library form their own
/// category there.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum TemplateCategory {
    /// Titles and title cards.
    Title,
    /// Name and role over the picture, in the lower third.
    LowerThird,
    /// Calls to action (subscribe, follow, link).
    CallToAction,
    /// Graphics covering a cut between two shots.
    Transition,
    /// Anything else (progress bars, counters, badges…).
    #[default]
    Other,
}

impl TemplateCategory {
    /// Every category, in the browser's order.
    pub const ALL: [Self; 5] = [
        Self::Title,
        Self::LowerThird,
        Self::CallToAction,
        Self::Transition,
        Self::Other,
    ];

    /// The name written in modules and JSON (`lowerThird`).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Title => "title",
            Self::LowerThird => "lowerThird",
            Self::CallToAction => "callToAction",
            Self::Transition => "transition",
            Self::Other => "other",
        }
    }

    /// The category called `name`, if any.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|category| category.name() == name)
    }

    /// Every name, comma-separated (for messages).
    #[must_use]
    pub fn names() -> String {
        Self::ALL
            .into_iter()
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
        for category in TemplateCategory::ALL {
            assert_eq!(TemplateCategory::from_name(category.name()), Some(category));
            assert_eq!(
                serde_json::to_string(&category).unwrap(),
                format!("\"{}\"", category.name())
            );
        }
        assert_eq!(TemplateCategory::from_name("intro"), None);
    }
}
