//! New items: the clips of dropped media and copies of existing items.

use std::collections::BTreeMap;

use se_domain::{
    Asset, AssetKind, DomainError, Flicks, ImageItem, Item, ItemKind, LinkGroupId, MediaClip,
    Origin, TimeRange,
};

use crate::{AppError, new_ids::NewIds};

/// How long a still image lasts once dropped on a sequence (images have no duration). The
/// renderer previews drops with the same value (`DEFAULT_IMAGE_DURATION` in the timeline's
/// drop placement): change both together.
pub const DEFAULT_IMAGE_DURATION: Flicks = Flicks::from_millis(5_000);

/// The new items of one dropped asset.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct MediaParts {
    /// Its picture (video assets, images), for a video track.
    pub(crate) video: Option<Item>,
    /// Its sound (audio assets, video assets with sound), for an audio track.
    pub(crate) audio: Option<Item>,
    /// How long both last.
    pub(crate) duration: Flicks,
}

/// The items playing `asset` from its start at `start`: a picture and/or a sound item, lasting
/// the whole media (images [`DEFAULT_IMAGE_DURATION`]), linked together when both exist.
///
/// # Errors
/// A video or audio asset whose duration is unknown, times out of range.
pub(crate) fn media_parts(
    asset: &Asset,
    start: Flicks,
    ids: NewIds<'_>,
) -> Result<MediaParts, AppError> {
    let media_duration = asset.media_info.as_ref().and_then(|info| info.duration);
    let has_sound = asset
        .media_info
        .as_ref()
        .is_some_and(|info| !info.audio_streams.is_empty());
    let (duration, picture, sound) = match asset.kind {
        AssetKind::Image => (Some(DEFAULT_IMAGE_DURATION), true, false),
        AssetKind::Video => (media_duration, true, has_sound),
        AssetKind::Audio => (media_duration, false, true),
    };
    let duration = duration.ok_or_else(|| AppError::InvalidEdit {
        reason: format!(
            "asset {} has no known duration (its media could not be read): relink or reimport it",
            asset.id
        ),
    })?;
    let timeline_range =
        TimeRange::new(start, duration).map_err(|error| AppError::InvalidEdit {
            reason: format!("cannot place asset {} there: {error}", asset.id),
        })?;
    let link_group = if picture && sound {
        Some(ids.link_group()?)
    } else {
        None
    };
    let new_item = |kind: ItemKind| -> Result<Item, AppError> {
        Ok(Item {
            id: ids.item()?,
            revision: 0,
            origin: Origin::User,
            timeline_range,
            link_group,
            kind,
        })
    };
    let picture_kind = match asset.kind {
        AssetKind::Image => ItemKind::Image(ImageItem::new(asset.id)),
        AssetKind::Video | AssetKind::Audio => ItemKind::MediaClip(MediaClip::new(asset.id)),
    };
    Ok(MediaParts {
        video: picture.then(|| new_item(picture_kind)).transpose()?,
        audio: sound
            .then(|| new_item(ItemKind::MediaClip(MediaClip::new(asset.id))))
            .transpose()?,
        duration,
    })
}

/// Copies of `items` as new user items: new ids, revision 0, origin `user`. Copies of items that
/// shared a link group share a new group; a copy whose group has no other copy is unlinked.
///
/// # Errors
/// The id generator broke its contract.
pub(crate) fn copies_of(items: Vec<Item>, ids: NewIds<'_>) -> Result<Vec<Item>, AppError> {
    let mut members: BTreeMap<LinkGroupId, usize> = BTreeMap::new();
    for group in items.iter().filter_map(|item| item.link_group) {
        *members.entry(group).or_default() += 1;
    }
    let mut new_groups: BTreeMap<LinkGroupId, LinkGroupId> = BTreeMap::new();
    items
        .into_iter()
        .map(|item| {
            let link_group = match item.link_group {
                Some(group) if members.get(&group).copied().unwrap_or(0) > 1 => {
                    Some(new_group(&mut new_groups, group, ids)?)
                }
                _ => None,
            };
            Ok(Item {
                id: ids.item()?,
                revision: 0,
                origin: Origin::User,
                link_group,
                ..item
            })
        })
        .collect()
}

/// The copy group of `group`, created on first use.
fn new_group(
    new_groups: &mut BTreeMap<LinkGroupId, LinkGroupId>,
    group: LinkGroupId,
    ids: NewIds<'_>,
) -> Result<LinkGroupId, AppError> {
    if let Some(&existing) = new_groups.get(&group) {
        return Ok(existing);
    }
    let created = ids.link_group()?;
    new_groups.insert(group, created);
    Ok(created)
}

/// `item` moved to start at `start`, keeping its duration.
///
/// # Errors
/// The new range leaves the supported time range.
pub(crate) fn starting_at(item: Item, start: Flicks) -> Result<Item, AppError> {
    let timeline_range = TimeRange::new(start, item.timeline_range.duration())
        .map_err(|_| DomainError::TimeOutOfRange { item: item.id })?;
    Ok(Item {
        timeline_range,
        ..item
    })
}

#[cfg(test)]
mod tests {
    use se_domain::{
        AssetId, ItemId,
        test_support::{example_ids, example_project},
    };

    use super::*;
    use crate::{edit_plan::testing::clip, test_support::SequentialIds};

    fn asset(id: AssetId) -> Asset {
        example_project()
            .assets
            .into_iter()
            .find(|asset| asset.id == id)
            .unwrap()
    }

    #[test]
    fn a_video_with_sound_gives_two_linked_items_of_its_length() {
        let ids = SequentialIds::default();
        let parts =
            media_parts(&asset(example_ids::VIDEO_ASSET), Flicks::ZERO, NewIds(&ids)).unwrap();
        let (video, audio) = (parts.video.unwrap(), parts.audio.unwrap());
        assert_eq!(video.link_group, audio.link_group);
        assert!(video.link_group.is_some());
        assert_eq!(
            parts.duration,
            Flicks::new(60 * Flicks::PER_SECOND).unwrap()
        );
        assert_eq!(video.timeline_range, audio.timeline_range);
        assert_eq!((video.revision, video.origin), (0, Origin::User));
    }

    #[test]
    fn sounds_and_images_give_one_item() {
        let ids = SequentialIds::default();
        let music =
            media_parts(&asset(example_ids::AUDIO_ASSET), Flicks::ZERO, NewIds(&ids)).unwrap();
        assert!(music.video.is_none() && music.audio.is_some());
        assert_eq!(music.audio.unwrap().link_group, None);
        let logo =
            media_parts(&asset(example_ids::IMAGE_ASSET), Flicks::ZERO, NewIds(&ids)).unwrap();
        assert!(matches!(
            logo.video.unwrap().kind,
            ItemKind::Image(ImageItem { .. })
        ));
        assert_eq!(logo.duration, DEFAULT_IMAGE_DURATION);
    }

    #[test]
    fn an_unknown_duration_is_refused() {
        let ids = SequentialIds::default();
        let mut video = asset(example_ids::VIDEO_ASSET);
        if let Some(info) = video.media_info.as_mut() {
            info.duration = None;
        }
        assert!(media_parts(&video, Flicks::ZERO, NewIds(&ids)).is_err());
    }

    #[test]
    fn copies_get_new_ids_and_keep_links_among_themselves() {
        let ids = SequentialIds::default();
        let copies = copies_of(
            vec![
                clip(1, 0, 2, Some(7)),
                clip(2, 0, 2, Some(7)),
                clip(3, 4, 6, Some(8)),
            ],
            NewIds(&ids),
        )
        .unwrap();
        assert!(
            copies
                .iter()
                .all(|copy| copy.id != ItemId::from_test_index(1))
        );
        assert_eq!(copies[0].link_group, copies[1].link_group);
        assert_ne!(copies[0].link_group, Some(LinkGroupId::from_test_index(7)));
        assert_eq!(copies[2].link_group, None, "a lone copy is unlinked");
    }
}
