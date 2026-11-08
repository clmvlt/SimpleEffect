//! Reshaping an item to a new extent: what trims, splits, ripple deletes and inserted gaps do to
//! its keyframes, fades and media.

use crate::{
    Animatable, AnimatableProperty, CaptionWord, DomainError, Flicks, Item, ItemId, ItemKind,
    Keyframe, MediaClip, PropertyMut, TimeRange,
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
/// Caption words stay at their instants too: a word belongs to the content holding its middle
/// (see [`retime_words`]).
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
    retime_properties(&mut kind, offset, duration);
    match &mut kind {
        ItemKind::MediaClip(clip) => reshape_clip(clip, offset, duration, reshape, item.id)?,
        ItemKind::Caption(caption) => {
            let ends_with_item = reshape.content.end() == item.timeline_range.end();
            caption.words = retime_words(&caption.words, offset, duration, ends_with_item);
        }
        _ => {}
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

/// Every animatable property of `kind` retimed (see [`retime`]): transforms, volume, text
/// styles, outlines, the reframe focus…
fn retime_properties(kind: &mut ItemKind, offset: Flicks, duration: Flicks) {
    for property in AnimatableProperty::ALL {
        match kind.property_mut(property) {
            Some(PropertyMut::Number(value)) => *value = retime(value, offset, duration),
            Some(PropertyMut::Pixels(value)) => *value = retime(value, offset, duration),
            Some(PropertyMut::Point(value)) => *value = retime(value, offset, duration),
            Some(PropertyMut::Color(value)) => *value = retime(value, offset, duration),
            None => {}
        }
    }
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

/// The words of a caption seen from a content starting `offset` after the item's start and
/// lasting `duration`: a word belongs to the content holding its middle (half-open, except that a
/// content ending with the item also keeps a word whose middle is the item's end), and its times
/// are clamped inside the content. Clamping keeps words sorted and apart.
fn retime_words(
    words: &[CaptionWord],
    offset: Flicks,
    duration: Flicks,
    ends_with_item: bool,
) -> Vec<CaptionWord> {
    let clamp = |time: Flicks| {
        time.checked_sub(offset)
            .map_or(Flicks::ZERO, |local| local.clamp(Flicks::ZERO, duration))
    };
    words
        .iter()
        .filter(|word| {
            let Some(middle) = word.middle().checked_sub(offset) else {
                return false;
            };
            Flicks::ZERO <= middle && (middle < duration || (ends_with_item && middle == duration))
        })
        .map(|word| CaptionWord {
            start: clamp(word.start),
            end: clamp(word.end),
            ..word.clone()
        })
        .collect()
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
    fn every_animatable_property_follows_a_trim() {
        let size = |points: &[(i64, f64)]| -> Animatable<crate::DesignPixels> {
            match keyframed(points) {
                Animatable::Keyframed { keyframes } => Animatable::Keyframed {
                    keyframes: keyframes
                        .into_iter()
                        .map(|keyframe| Keyframe {
                            value: crate::DesignPixels(keyframe.value),
                            time: keyframe.time,
                            easing: keyframe.easing,
                        })
                        .collect(),
                },
                Animatable::Static { value } => Animatable::constant(crate::DesignPixels(value)),
            }
        };
        let mut text = crate::TextItem::new("Titre");
        text.style.font_size = size(&[(0, 40.0), (3000, 80.0)]);
        text.transform.blur = size(&[(0, 8.0), (3000, 0.0)]);
        let item = crate::test_support::item(1, range_ms(0, 4000), ItemKind::Text(text));
        let trimmed = reshape(&item, &Reshape::trim(range_ms(1000, 3000))).unwrap();
        let ItemKind::Text(text) = trimmed.kind else {
            unreachable!()
        };
        assert_eq!(text.style.font_size, size(&[(2000, 80.0)]));
        assert_eq!(text.transform.blur, size(&[(2000, 0.0)]));

        let mut clip = MediaClip::new(crate::AssetId::from_test_index(1));
        let focus = |x: f64, at: i64| Keyframe {
            time: millis(at),
            value: crate::Vec2::new(x, 0.5),
            easing: crate::Easing::Linear,
        };
        clip.reframe = crate::Reframe::Fill {
            focus: Animatable::Keyframed {
                keyframes: vec![focus(0.2, 0), focus(0.8, 3000)],
            },
        };
        let item = crate::test_support::item(2, range_ms(0, 4000), ItemKind::MediaClip(clip));
        let trimmed = reshape(&item, &Reshape::trim(range_ms(1000, 3000))).unwrap();
        let ItemKind::MediaClip(clip) = trimmed.kind else {
            unreachable!()
        };
        assert_eq!(
            clip.reframe.focus(),
            Some(&Animatable::Keyframed {
                keyframes: vec![focus(0.8, 2000)]
            })
        );
    }

    #[test]
    fn caption_words_go_to_the_part_holding_their_middle() {
        let word =
            |text: &str, start: i64, end: i64| CaptionWord::new(text, millis(start), millis(end));
        let caption = crate::CaptionItem {
            words: vec![
                word("un", 0, 400),
                word("deux", 500, 1100),
                word("trois", 1200, 2000),
            ],
            style_id: crate::CaptionStyleId::from_test_index(1),
        };
        let item = crate::test_support::item(1, range_ms(0, 2000), ItemKind::Caption(caption));
        let (left, right) = cut_pieces(
            &item,
            millis(900),
            millis(900),
            millis(900),
            ItemId::from_test_index(2),
        )
        .unwrap();
        let words = |item: Item| match item.kind {
            ItemKind::Caption(caption) => caption.words,
            _ => unreachable!(),
        };
        // « deux » has its middle at 800 ms: it stays on the left, clamped at the cut.
        assert_eq!(words(left), [word("un", 0, 400), word("deux", 500, 900)]);
        assert_eq!(words(right), [word("trois", 300, 1100)]);
        let extended = reshape(&item, &Reshape::trim(range_ms(-1000, 3000))).unwrap();
        assert_eq!(words(extended)[0], word("un", 1000, 1400));
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
