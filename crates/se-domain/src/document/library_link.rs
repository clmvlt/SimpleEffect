//! [`LibraryLink`]: the library entry a project's motion template was copied from (prompt 25).

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::LibraryTemplateId;

/// The app's template library entry a project template is a copy of, and which version of it.
///
/// The project keeps the whole template (it opens without the library); the link only lets the
/// interface propose an update when the library holds a newer version (never applied by itself),
/// and lets « Enregistrer dans la bibliothèque » write a new version of the same entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct LibraryLink {
    /// The library entry.
    pub template_id: LibraryTemplateId,
    /// The library version this copy holds.
    pub version: u32,
}

impl LibraryLink {
    /// Whether the library's `version` of the entry is newer than this copy.
    #[must_use]
    pub const fn is_outdated_by(self, version: u32) -> bool {
        version > self.version
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_higher_library_version_is_newer() {
        let link = LibraryLink {
            template_id: LibraryTemplateId::from_test_index(1),
            version: 3,
        };
        assert!(link.is_outdated_by(4));
        assert!(!link.is_outdated_by(3));
        assert!(!link.is_outdated_by(2));
    }
}
