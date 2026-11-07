//! [`LocalFontCatalog`]: bundled fonts first, then the installed ones (read once, on first use).

use async_trait::async_trait;
use se_app::ports::{FontCatalog, FontData};
use se_protocol::FontFamily;
use tokio::sync::OnceCell;

use crate::{
    bundled::{bundled_face, bundled_families},
    system::{SystemFonts, load_system_fonts},
};

/// The font catalogue of this machine.
#[derive(Debug)]
pub struct LocalFontCatalog {
    include_system: bool,
    system: OnceCell<SystemFonts>,
}

impl LocalFontCatalog {
    /// Bundled and installed fonts.
    #[must_use]
    pub fn new() -> Self {
        Self {
            include_system: true,
            system: OnceCell::new(),
        }
    }

    /// Bundled fonts only (tests, and machines where reading fonts fails).
    #[must_use]
    pub fn bundled_only() -> Self {
        Self {
            include_system: false,
            system: OnceCell::new(),
        }
    }

    /// The installed fonts, read on the blocking pool the first time.
    async fn system(&self) -> &SystemFonts {
        self.system
            .get_or_init(|| async {
                if !self.include_system {
                    return SystemFonts::default();
                }
                let skip: Vec<String> = bundled_families()
                    .into_iter()
                    .map(|family| family.name)
                    .collect();
                tokio::task::spawn_blocking(move || load_system_fonts(&skip))
                    .await
                    .unwrap_or_else(|error| {
                        tracing::warn!(%error, "reading the installed fonts failed");
                        SystemFonts::default()
                    })
            })
            .await
    }
}

impl Default for LocalFontCatalog {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FontCatalog for LocalFontCatalog {
    async fn families(&self) -> Vec<FontFamily> {
        let mut families = bundled_families();
        let system = self.system().await;
        tracing::debug!(count = system.families.len(), "installed font families");
        families.extend(system.families.iter().cloned());
        families
    }

    async fn face_data(&self, id: &str) -> Option<FontData> {
        if let Some(face) = bundled_face(id) {
            return Some(FontData::Embedded {
                bytes: face.bytes,
                content_type: "font/woff2",
            });
        }
        self.system().await.files.get(id).cloned()
    }
}

#[cfg(test)]
mod tests {
    use se_protocol::FontSource;

    use super::*;

    #[tokio::test]
    async fn bundled_families_come_first_and_their_faces_are_served() {
        let catalog = LocalFontCatalog::bundled_only();
        let families = catalog.families().await;
        assert!(
            families
                .iter()
                .all(|family| family.source == FontSource::Bundled)
        );
        let face = &families[0].faces[0];
        let Some(FontData::Embedded {
            bytes,
            content_type,
        }) = catalog.face_data(&face.id).await
        else {
            panic!("bundled faces are embedded");
        };
        assert!(bytes.starts_with(b"wOF2"));
        assert_eq!(content_type, "font/woff2");
        assert_eq!(catalog.face_data("system-0000000000000000").await, None);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn installed_families_follow_the_bundled_ones() {
        let catalog = LocalFontCatalog::new();
        let families = catalog.families().await;
        let first_system = families
            .iter()
            .position(|family| family.source == FontSource::System)
            .unwrap_or(families.len());
        assert!(
            families[..first_system]
                .iter()
                .all(|family| family.source == FontSource::Bundled)
        );
        assert!(
            families[first_system..]
                .iter()
                .all(|family| family.source == FontSource::System)
        );
    }
}
