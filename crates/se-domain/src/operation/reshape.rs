//! Reshaping an item to a new extent: what trims, splits, ripple deletes and inserted gaps do to
//! its keyframes, fades and media.

use crate::{
    Animatable, DomainError, Flicks, Item, ItemId, ItemKind, Keyframe, MediaClip, TimeRange,
    Transform2D,
};

/// The new extent of an item.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Reshape {
    /// The part of the timeline the item covers afterwards, in its current coordinates: inside
    /// its range when cutting, beyond it when extending a trim.
    pub(crate) content: TimeRange,
    /// Where the reshaped item starts (differs from `content.start()` when the content shifts).
    pub(crate) start: Flicks,
    /// Keep the fade-in (an edge moved by a trim keeps its fade; an edge made by a cut has none).
    pub(crate) keep_fade_in: bool,
    /// Keep the fade-out.
    pub(crate) keep_fade_out: bool,
}

impl Reshape {
    /// A trim to `content`: the item stays where its content is and keeps both fades.
    pub(crate) const fn trim(content: TimeRange) -> Self {
        Self {
            content,
            start: content.start(),
            keep_fade_in: true,
            keep_fade_out: true,
        }
    }
}

/// `item` reshaped: timeline range `[start, start + content duration)`, keyframes moved so they
/// stay at the same instants of the content (those falling outside are dropped), media clips
/// playing the same source at the same instants, fades kept as asked and shortened to fit.
///
/// When every keyframe of a property falls outside, the property becomes static with the value
/// it held over the content: the last keyframe before it, or else the first one after it.
///
/// # Errors
/// [`DomainError::TimeOutOfRange`] if a time leaves the supported range.
pub(crate) fn reshape(item: &Item, reshape: &Reshape) -> Result<Item, DomainError> {
    let out_of_range = || DomainError::TimeOutOfRange { item: item.id };
    let offset = reshape
        .content
        .start()
        .checked_sub(item.timeline_range.start())
        .ok_or_else(out_of_range)?;
    let duration = reshape.content.duration();
    let timeline_range = TimeRange::new(reshape.start, duration).map_err(|_| out_of_range())?;
    let mut kind = item.kind.clone();
    if let Some(transform) = kind.transform_mut() {
        retime_transform(transform, offset, duration);
    }
    if let ItemKind::MediaClip(clip) = &mut kind {
        reshape_clip(clip, offset, duration, reshape, item.id)?;
    }
    Ok(Item {
        timeline_range,
        kind,
        ..item.clone()
    })
}

/// `item` moved by `delta` without other change.
///
/// # Errors
/// [`DomainError::TimeOutOfRange`] if the moved range leaves the supported range.
pub(crate) fn shifted(item: &Item, delta: Flicks) -> Result<Item, DomainError> {
    let timeline_range = item
        .timeline_range
        .shifted(delta)
        .map_err(|_| DomainError::TimeOutOfRange { item: item.id })?;
    Ok(Item {
        timeline_range,
        ..item.clone()
    })
}

/// The two parts of `item` cut out of the gap `[left_end, right_start)` (empty for a plain
/// split): the left part keeps the id and the fade-in; the right part, placed at `right_at`, gets
/// `new_id`, revision 0 (new item), no link group and the fade-out.
///
/// # Errors
/// [`DomainError::TimeOutOfRange`] if a time leaves the supported range.
pub(crate) fn cut_pieces(
    item: &Item,
    left_end: Flicks,
    right_start: Flicks,
    right_at: Flicks,
    new_id: ItemId,
) -> Result<(Item, Item), DomainError> {
    let out_of_range = || DomainError::TimeOutOfRange { item: item.id };
    let range = item.timeline_range;
    let left_content =
        TimeRange::from_start_end(range.start(), left_end).map_err(|_| out_of_range())?;
    let right_content =
        TimeRange::from_start_end(right_start, range.end()).map_err(|_| out_of_range())?;
    let left = reshape(
        item,
        &Reshape {
            keep_fade_out: false,
            ..Reshape::trim(left_content)
        },
    )?;
    let right = reshape(
        item,
        &Reshape {
            content: right_content,
            start: right_at,
            keep_fade_in: false,
            keep_fade_out: true,
        },
    )?;
    let right = Item {
        id: new_id,
        revision: 0,
        link_group: None,
        ..right
    };
    Ok((left, right))
}

fn reshape_clip(
    clip: &mut MediaClip,
    offset: Flicks,
    duration: Flicks,
    reshape: &Reshape,
    item: ItemId,
) -> Result<(), DomainError> {
    clip.volume_db = retime(&clip.volume_db, offset, duration);
    clip.source_start = offset
        .checked_mul_f64(clip.speed)
        .and_then(|source_offset| clip.source_start.checked_add(source_offset))
        .ok_or(DomainError::TimeOutOfRange { item })?;
    let fade_in = if reshape.keep_fade_in {
        clip.fade_in.min(duration)
    } else {
        Flicks::ZERO
    };
    let room_left = duration.checked_sub(fade_in).unwrap_or(Flicks::ZERO);
    clip.fade_out = if reshape.keep_fade_out {
        clip.fade_out.min(room_left)
    } else {
        Flicks::ZERO
    };
    clip.fade_in = fade_in;
    Ok(())
}

fn retime_transform(transform: &mut Transform2D, offset: Flicks, duration: Flicks) {
    transform.position = retime(&transform.position, offset, duration);
    transform.scale = retime(&transform.scale, offset, duration);
    transform.rotation = retime(&transform.rotation, offset, duration);
    transform.anchor = retime(&transform.anchor, offset, duration);
    transform.opacity = retime(&transform.opacity, offset, duration);
}

/// The keyframes of `property` seen from a content starting `offset` after the item's start and
/// lasting `duration` (see [`reshape`]).
fn retime<T: Clone>(property: &Animatable<T>, offset: Flicks, duration: Flicks) -> Animatable<T> {
    let Animatable::Keyframed { keyframes } = property else {
        return property.clone();
    };
    let kept: Vec<Keyframe<T>> = keyframes
        .iter()
        .filter_map(|keyframe| {
            let time = keyframe.time.checked_sub(offset)?;
            (Flicks::ZERO <= time && time <= duration).then(|| Keyframe {
                time,
                ..keyframe.clone()
            })
        })
        .collect();
    if !kept.is_empty() {
        return Animatable::Keyframed { keyframes: kept };
    }
    let held = keyframes
        .iter()
        .rev()
        .find(|keyframe| keyframe.time < offset)
        .or_else(|| keyframes.first());
    held.map_or_else(
        || property.clone(),
        |keyframe| Animatable::constant(keyframe.value.clone()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Easing,
        test_support::{millis, range_ms},
    };

    fn keyframed(points: &[(i64, f64)]) -> Animatable<f64> {
        Animatable::Keyframed {
            keyframes: points
                .iter()
                .map(|&(time_ms, value)| Keyframe {
                    time: millis(time_ms),
                    value,
                    easing: Easing::Linear,
                })
                .collect(),
        }
    }

    #[test]
    fn keyframes_keep_their_instants_and_drop_outside_the_content() {
        let property = keyframed(&[(0, 0.0), (1000, 1.0), (3000, 2.0)]);
        assert_eq!(
            retime(&property, millis(500), millis(2000)),
            keyframed(&[(500, 1.0)])
        );
        assert_eq!(
            retime(&property, millis(-1000), millis(5000)),
            keyframed(&[(1000, 0.0), (2000, 1.0), (4000, 2.0)])
        );
    }

    #[test]
    fn a_property_without_keyframes_left_holds_its_value() {
        let property = keyframed(&[(1000, 1.0), (2000, 2.0)]);
        assert_eq!(
            retime(&property, millis(2500), millis(500)),
            Animatable::constant(2.0)
        );
        assert_eq!(
            retime(&property, Flicks::ZERO, millis(500)),
            Animatable::constant(1.0)
        );
    }

    #[test]
    fn cut_pieces_split_fades_and_media() {
        let mut clip = MediaClip::new(crate::AssetId::from_test_index(1));
        clip.speed = 2.0;
        clip.fade_in = millis(1000);
        clip.fade_out = millis(1000);
        let item = crate::test_support::item(1, range_ms(0, 4000), ItemKind::MediaClip(clip));
        let (left, right) = cut_pieces(
            &item,
            millis(1000),
            millis(3000),
            millis(1000),
            ItemId::from_test_index(2),
        )
        .unwrap();
        assert_eq!(left.timeline_range, range_ms(0, 1000));
        assert_eq!(right.timeline_range, range_ms(1000, 1000));
        let (ItemKind::MediaClip(left), ItemKind::MediaClip(right)) = (left.kind, right.kind)
        else {
            unreachable!()
        };
        assert_eq!((left.fade_in, left.fade_out), (millis(1000), Flicks::ZERO));
        assert_eq!(
            (right.fade_in, right.fade_out),
            (Flicks::ZERO, millis(1000))
        );
        assert_eq!(right.source_start, millis(6000));
    }
}
