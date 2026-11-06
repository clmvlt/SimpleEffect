//! [`Color`]: an sRGB colour with straight alpha, written as hex.

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use ts_rs::TS;

/// An 8-bit sRGB colour with straight (non-premultiplied) alpha.
///
/// Serialized as `#rrggbb` when opaque and `#rrggbbaa` otherwise, in lowercase; parsing accepts
/// either form in any letter case. This is how users, agents and the UI colour fields write it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, TS)]
#[ts(type = "string")]
pub struct Color {
    /// Red channel.
    pub red: u8,
    /// Green channel.
    pub green: u8,
    /// Blue channel.
    pub blue: u8,
    /// Opacity: 0 transparent, 255 opaque.
    pub alpha: u8,
}

/// Text that is not a `#rrggbb` or `#rrggbbaa` colour.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("'{0}' is not a colour: write it as #rrggbb or #rrggbbaa (hexadecimal), e.g. #ffcc00")]
pub struct ColorParseError(pub String);

impl Color {
    /// Opaque white.
    pub const WHITE: Self = Self::opaque(255, 255, 255);
    /// Opaque black.
    pub const BLACK: Self = Self::opaque(0, 0, 0);

    /// An opaque colour.
    #[must_use]
    pub const fn opaque(red: u8, green: u8, blue: u8) -> Self {
        Self {
            red,
            green,
            blue,
            alpha: 255,
        }
    }

    /// Parses `#rrggbb` or `#rrggbbaa`.
    ///
    /// # Errors
    /// [`ColorParseError`] for any other text.
    pub fn parse(text: &str) -> Result<Self, ColorParseError> {
        let error = || ColorParseError(text.to_owned());
        let digits = text.strip_prefix('#').ok_or_else(error)?;
        if !matches!(digits.len(), 6 | 8) || !digits.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(error());
        }
        let channel = |index: usize| {
            digits
                .get(index..index + 2)
                .and_then(|pair| u8::from_str_radix(pair, 16).ok())
                .ok_or_else(error)
        };
        Ok(Self {
            red: channel(0)?,
            green: channel(2)?,
            blue: channel(4)?,
            alpha: if digits.len() == 8 { channel(6)? } else { 255 },
        })
    }
}

impl fmt::Display for Color {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "#{:02x}{:02x}{:02x}",
            self.red, self.green, self.blue
        )?;
        if self.alpha != 255 {
            write!(formatter, "{:02x}", self.alpha)?;
        }
        Ok(())
    }
}

impl Serialize for Color {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        Self::parse(&text).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_opaque_and_translucent_colours() {
        assert_eq!(Color::parse("#FFcc00"), Ok(Color::opaque(255, 204, 0)));
        assert_eq!(Color::parse("#00000080").map(|color| color.alpha), Ok(0x80));
    }

    #[test]
    fn displays_in_canonical_lowercase() {
        assert_eq!(Color::opaque(255, 204, 0).to_string(), "#ffcc00");
        let translucent = Color {
            alpha: 0x80,
            ..Color::BLACK
        };
        assert_eq!(translucent.to_string(), "#00000080");
    }

    #[test]
    fn rejects_other_notations() {
        for text in [
            "ffcc00", "#fc0", "#ffcc0", "#ffcc00f", "#gggggg", "#ffçc00", "",
        ] {
            let error = Color::parse(text).unwrap_err();
            assert!(error.to_string().contains("#rrggbb"), "{text}");
        }
    }

    #[test]
    fn round_trips_through_json() {
        let json = serde_json::to_string(&Color::WHITE).unwrap();
        assert_eq!(json, "\"#ffffff\"");
        assert_eq!(serde_json::from_str::<Color>(&json).unwrap(), Color::WHITE);
    }
}
