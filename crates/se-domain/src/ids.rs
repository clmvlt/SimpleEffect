//! Identifiers of document entities: one newtype per kind over a UUIDv7 (`docs/architecture.md`
//! §5.2).
//!
//! This module only parses and validates ids. New ids come from the `IdGenerator` port of se-app
//! (deterministic in tests), which wraps the UUIDs it creates with `from_uuid`. Ids travel as
//! hyphenated lowercase strings (`01890a5d-ac96-774b-bcce-b302099a8057`) and are `string` in
//! TypeScript.

use std::{fmt, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use ts_rs::TS;
pub use uuid::Uuid;
use uuid::{Variant, Version};

/// Length of the hyphenated form, the only textual form accepted.
const HYPHENATED_LENGTH: usize = 36;

/// A string or UUID that is not a valid id of the expected kind.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error(
    "'{value}' is not a valid {kind} id: expected a hyphenated UUIDv7 such as \
     01890a5d-ac96-774b-bcce-b302099a8057"
)]
pub struct IdError {
    /// Kind of id that was expected (`item`, `asset`…).
    pub kind: &'static str,
    /// The rejected value.
    pub value: String,
}

/// Parses `text` as a hyphenated RFC 9562 UUID of version 7.
fn parse_uuid_v7(text: &str, kind: &'static str) -> Result<Uuid, IdError> {
    let uuid = Uuid::try_parse(text)
        .ok()
        .filter(|_| text.len() == HYPHENATED_LENGTH);
    match uuid {
        Some(uuid) => check_v7(uuid, kind),
        None => Err(IdError {
            kind,
            value: text.to_owned(),
        }),
    }
}

/// Accepts only version-7 UUIDs of the RFC variant.
fn check_v7(uuid: Uuid, kind: &'static str) -> Result<Uuid, IdError> {
    if uuid.get_version() == Some(Version::SortRand) && uuid.get_variant() == Variant::RFC4122 {
        Ok(uuid)
    } else {
        Err(IdError {
            kind,
            value: uuid.hyphenated().to_string(),
        })
    }
}

/// Declares an id newtype with parsing, display, serde (as a string) and a `string` TS type.
macro_rules! uuid_id {
    ($(#[$doc:meta])* $name:ident, $kind:literal) => {
        $(#[$doc])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, TS)]
        #[ts(type = "string")]
        pub struct $name(Uuid);

        impl $name {
            /// Kind name used in error messages.
            pub const KIND: &'static str = $kind;

            /// Parses a hyphenated UUIDv7 (any letter case; displayed in lowercase).
            ///
            /// # Errors
            /// [`IdError`] if `text` is not a hyphenated version-7 UUID.
            pub fn parse(text: &str) -> Result<Self, IdError> {
                parse_uuid_v7(text, Self::KIND).map(Self)
            }

            /// Wraps a UUID created by an id generator.
            ///
            /// # Errors
            /// [`IdError`] if `uuid` is not a version-7 UUID.
            pub fn from_uuid(uuid: Uuid) -> Result<Self, IdError> {
                check_v7(uuid, Self::KIND).map(Self)
            }

            /// The underlying UUID.
            #[must_use]
            pub const fn as_uuid(self) -> Uuid {
                self.0
            }

            /// A deterministic id for tests: the UUIDv7 whose timestamp is `index` milliseconds
            /// and whose random bits are zero (`00000000-0003-7000-8000-000000000000` for 3).
            #[cfg(any(test, feature = "test-support"))]
            #[must_use]
            pub const fn from_test_index(index: u64) -> Self {
                Self(uuid::Builder::from_unix_timestamp_millis(index, &[0; 10]).into_uuid())
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Display::fmt(&self.0.hyphenated(), formatter)
            }
        }

        impl FromStr for $name {
            type Err = IdError;

            fn from_str(text: &str) -> Result<Self, Self::Err> {
                Self::parse(text)
            }
        }

        impl Serialize for $name {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.collect_str(self)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let text = String::deserialize(deserializer)?;
                Self::parse(&text).map_err(serde::de::Error::custom)
            }
        }
    };
}

uuid_id!(
    /// Identifies a [`crate::Project`].
    ProjectId,
    "project"
);
uuid_id!(
    /// Identifies a [`crate::Sequence`] within its project.
    SequenceId,
    "sequence"
);
uuid_id!(
    /// Identifies a [`crate::Track`] within its project.
    TrackId,
    "track"
);
uuid_id!(
    /// Identifies an [`crate::Item`] within its project.
    ItemId,
    "item"
);
uuid_id!(
    /// Identifies an [`crate::Asset`] within its project.
    AssetId,
    "asset"
);
uuid_id!(
    /// Identifies an AI task (the request, its runs and its changeset).
    TaskId,
    "task"
);
uuid_id!(
    /// Identifies a changeset: all transactions of one AI task (prompt 06).
    ChangesetId,
    "changeset"
);
uuid_id!(
    /// Identifies a [`crate::MotionTemplate`] within its project.
    TemplateId,
    "template"
);
uuid_id!(
    /// Groups items that move and trim together (a video clip and its audio).
    LinkGroupId,
    "link group"
);
uuid_id!(
    /// Identifies a [`crate::Marker`] within its project.
    MarkerId,
    "marker"
);
uuid_id!(
    /// Identifies a [`crate::Transition`] within its project.
    TransitionId,
    "transition"
);
uuid_id!(
    /// Identifies a [`crate::CaptionStyle`] within its project (prompt 23).
    CaptionStyleId,
    "caption style"
);
uuid_id!(
    /// Identifies a brand kit of the app's library (prompt 25): projects keep a copy of the kit
    /// they use, with this id as its source.
    BrandKitId,
    "brand kit"
);
uuid_id!(
    /// Identifies an entry of the app's template library (a motion template or a caption style,
    /// prompt 25): project templates copied from it keep this id as their link.
    LibraryTemplateId,
    "library template"
);
uuid_id!(
    /// Identifies a [`crate::Transaction`]: one atomic group of operations by one actor.
    TransactionId,
    "transaction"
);
uuid_id!(
    /// Identifies a background job of the core (thumbnails, waveform peaks…). Jobs live in memory
    /// only: documents never reference them.
    JobId,
    "job"
);
uuid_id!(
    /// Identifies a video export of the core (prompt 27): its job, its render by the renderer and
    /// its upload routes. Exports live in memory only: documents never reference them.
    ExportId,
    "export"
);

#[cfg(test)]
mod tests {
    use super::*;

    const V7: &str = "01890a5d-ac96-774b-bcce-b302099a8057";

    #[test]
    fn parses_and_displays_hyphenated_v7_uuids() {
        let id = ItemId::parse(V7).unwrap();
        assert_eq!(id.to_string(), V7);
        assert_eq!(ItemId::parse(&V7.to_uppercase()).unwrap(), id);
        assert_eq!(V7.parse::<ItemId>().unwrap(), id);
    }

    #[test]
    fn rejects_other_versions_and_forms() {
        let v4 = "67e55044-10b1-426f-9247-bb680e5fe0c8";
        let simple = "01890a5dac96774bbccebb302099a805";
        for text in [v4, simple, "", "not-an-id", &format!("{{{V7}}}")] {
            let error = AssetId::parse(text).unwrap_err();
            assert_eq!(error.kind, "asset");
            assert!(error.to_string().contains("UUIDv7"), "{error}");
        }
        assert!(TrackId::from_uuid(Uuid::nil()).is_err());
    }

    #[test]
    fn test_ids_are_valid_distinct_and_readable() {
        let id = SequenceId::from_test_index(3);
        assert_eq!(id.to_string(), "00000000-0003-7000-8000-000000000000");
        assert_eq!(SequenceId::from_uuid(id.as_uuid()), Ok(id));
        assert_ne!(id, SequenceId::from_test_index(4));
    }

    #[test]
    fn serializes_as_a_string() {
        let id = MarkerId::parse(V7).unwrap();
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, format!("\"{V7}\""));
        assert_eq!(serde_json::from_str::<MarkerId>(&json).unwrap(), id);
        assert!(serde_json::from_str::<MarkerId>("\"nope\"").is_err());
    }
}
