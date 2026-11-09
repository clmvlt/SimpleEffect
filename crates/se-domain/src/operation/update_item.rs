//! [`UpdateItem`]: changes the content of an item.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{
    Edit, Effect, ItemPatch, checks::check_revision, lookup::find_item, placement::place_item,
};
use crate::{DomainError, ItemId, Project};

/// Applies a typed [`ItemPatch`] to an item of the same kind. Inverse: [`super::RestoreItem`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct UpdateItem {
    /// The item changed.
    pub item: ItemId,
    /// What changes.
    pub patch: ItemPatch,
    /// Revision the caller read, if it wants the change refused when the item changed since.
    pub expected_revision: Option<u64>,
}

impl Edit for UpdateItem {
    fn apply_to(&self, project: &mut Project, effect: &mut Effect) -> Result<(), DomainError> {
        let location = find_item(project, self.item)?;
        let item = location.get(project);
        check_revision(item, self.expected_revision)?;
        let mut updated = item.clone();
        if !self.patch.apply(&mut updated.kind) {
            return Err(DomainError::PatchKindMismatch {
                item: self.item,
                item_kind: item.kind.label(),
                patch_kind: self.patch.label(),
            });
        }
        place_item(project, location, location.track, updated, effect)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Color, ItemKind, MediaClipPatch, TemplateParamValue, TemplatePatch, TextPatch,
        ViolationCode,
        operation::testing::{V2, apply_err, apply_ok, clip, example, find, lock},
        test_support::millis,
    };

    fn update(item: u64, patch: ItemPatch) -> UpdateItem {
        UpdateItem {
            item: ItemId::from_test_index(item),
            patch,
            expected_revision: None,
        }
    }

    #[test]
    fn changes_only_the_given_fields() {
        let patch = ItemPatch::Text(TextPatch {
            content: Some("Salut".to_owned()),
            color: Some(Color::BLACK),
            ..TextPatch::default()
        });
        let applied = apply_ok(&example(), update(3, patch));
        let ItemKind::Text(text) = &find(&applied.project, 3).kind else {
            unreachable!()
        };
        assert_eq!(text.content, "Salut");
        assert_eq!(text.style.color, crate::Animatable::constant(Color::BLACK));
        assert_eq!(text.style.font_family, "Inter");
    }

    #[test]
    fn sets_and_resets_template_parameters() {
        let patch = ItemPatch::MotionTemplate(TemplatePatch {
            template_id: None,
            params: [
                (
                    "name".to_owned(),
                    Some(TemplateParamValue::Text {
                        value: "Bob".to_owned(),
                    }),
                ),
                ("showLine".to_owned(), None),
            ]
            .into(),
        });
        let applied = apply_ok(&example(), update(5, patch));
        let ItemKind::MotionTemplate(template) = &find(&applied.project, 5).kind else {
            unreachable!()
        };
        assert_eq!(
            template.params.get("name"),
            Some(&TemplateParamValue::Text {
                value: "Bob".to_owned()
            })
        );
        assert!(!template.params.contains_key("showLine"));
    }

    #[test]
    fn slips_and_speeds_media_within_their_asset() {
        let slip = ItemPatch::MediaClip(MediaClipPatch {
            source_start: Some(millis(30_000)),
            ..MediaClipPatch::default()
        });
        let applied = apply_ok(&example(), update(1, slip));
        assert_eq!(clip(find(&applied.project, 1)).source_start, millis(30_000));
        let too_fast = ItemPatch::MediaClip(MediaClipPatch {
            speed: Some(10.0),
            ..MediaClipPatch::default()
        });
        let DomainError::InvalidDocument { violations } =
            apply_err(&example(), update(1, too_fast))
        else {
            panic!("expected a validation error");
        };
        assert_eq!(violations[0].code, ViolationCode::SourceRangeExceedsMedia);
    }

    #[test]
    fn refuses_a_patch_of_another_kind_and_locked_tracks() {
        let patch = ItemPatch::MediaClip(MediaClipPatch::default());
        let error = apply_err(&example(), update(3, patch));
        assert_eq!(
            error.to_string(),
            "item 00000000-0003-7000-8000-000000000000 is a text; send a text patch, not a media \
             clip patch"
        );
        let patch = ItemPatch::Text(TextPatch::default());
        assert!(matches!(
            apply_err(&lock(example(), V2), update(3, patch)),
            DomainError::TrackLocked { .. }
        ));
    }
}
