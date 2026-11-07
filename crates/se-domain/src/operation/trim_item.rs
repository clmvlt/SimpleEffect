//! [`TrimItem`]: moves one edge of an item.

use std::fmt;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{
    Edit, Effect,
    checks::check_revision,
    lookup::find_item,
    placement::place_item,
    reshape::{Reshape, reshape},
};
use crate::{DomainError, Flicks, Item, ItemId, ItemKind, MediaClip, Project, TimeRange};

/// One edge of an item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum ItemEdge {
    /// The first instant.
    Start,
    /// The instant after the last one.
    End,
}

impl ItemEdge {
    /// Where this edge must stay relative to the other one, for messages.
    pub(crate) const fn must_stay(self) -> &'static str {
        match self {
            Self::Start => "before",
            Self::End => "after",
        }
    }
}

impl fmt::Display for ItemEdge {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Start => "start",
            Self::End => "end",
        })
    }
}

/// Moves the start or the end of an item to `to`; the other edge stays.
///
/// A media clip keeps playing the same source at the same instants (trimming the start moves its
/// `source_start`) and cannot play beyond its asset. Keyframes stay at their instants; those left
/// outside are dropped. Fades keep their length, shortened if the item becomes too short.
/// Inverse: [`super::RestoreItem`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct TrimItem {
    /// The item trimmed.
    pub item: ItemId,
    /// The edge moved.
    pub edge: ItemEdge,
    /// The new position of that edge in the sequence.
    pub to: Flicks,
    /// Revision the caller read, if it wants the trim refused when the item changed since.
    pub expected_revision: Option<u64>,
}

impl Edit for TrimItem {
    fn apply_to(&self, project: &mut Project, effect: &mut Effect) -> Result<(), DomainError> {
        let location = find_item(project, self.item)?;
        let item = location.get(project);
        check_revision(item, self.expected_revision)?;
        let content = self.content(item.timeline_range)?;
        if let ItemKind::MediaClip(clip) = &item.kind {
            self.check_media_bounds(project, item, clip, content)?;
        }
        let trimmed = reshape(item, &Reshape::trim(content))?;
        place_item(project, location, location.track, trimmed, effect)
    }
}

impl TrimItem {
    /// The item's extent after the trim.
    fn content(&self, range: TimeRange) -> Result<TimeRange, DomainError> {
        let (start, end, other_edge) = match self.edge {
            ItemEdge::Start => (self.to, range.end(), range.end()),
            ItemEdge::End => (range.start(), self.to, range.start()),
        };
        if start >= end {
            return Err(DomainError::TrimCollapsesItem {
                item: self.item,
                edge: self.edge,
                to: self.to,
                other_edge,
            });
        }
        TimeRange::from_start_end(start, end)
            .map_err(|_| DomainError::TimeOutOfRange { item: self.item })
    }

    /// Fails if the trimmed clip would play before the start or after the end of its asset.
    fn check_media_bounds(
        &self,
        project: &Project,
        item: &Item,
        clip: &MediaClip,
        content: TimeRange,
    ) -> Result<(), DomainError> {
        let out_of_range = || DomainError::TimeOutOfRange { item: self.item };
        let offset = content
            .start()
            .checked_sub(item.timeline_range.start())
            .ok_or_else(out_of_range)?;
        let source_start = offset
            .checked_mul_f64(clip.speed)
            .and_then(|source_offset| clip.source_start.checked_add(source_offset))
            .ok_or_else(out_of_range)?;
        let media_duration = project
            .assets
            .iter()
            .find(|asset| asset.id == clip.asset_id)
            .and_then(|asset| asset.media_info.as_ref())
            .and_then(|info| info.duration);
        let exceeds = match (self.edge, media_duration) {
            (ItemEdge::Start, _) => source_start.is_negative(),
            (ItemEdge::End, Some(media_duration)) => content
                .duration()
                .checked_mul_f64(clip.speed)
                .and_then(|source_duration| source_start.checked_add(source_duration))
                .is_none_or(|source_end| source_end > media_duration),
            (ItemEdge::End, None) => false,
        };
        if !exceeds {
            return Ok(());
        }
        Err(DomainError::TrimBeyondSource {
            item: self.item,
            edge: self.edge,
            to: self.to,
            limit: self.media_limit(item, clip, media_duration),
        })
    }

    /// The furthest position of the trimmed edge that stays inside the media, rounded to the
    /// millisecond on the safe side (shown to users and agents, never used for the check).
    fn media_limit(&self, item: &Item, clip: &MediaClip, media_duration: Option<Flicks>) -> Flicks {
        let start = item.timeline_range.start().to_seconds_f64();
        let source_start = clip.source_start.to_seconds_f64();
        match self.edge {
            ItemEdge::Start => {
                rounded_to_millisecond(start - source_start / clip.speed, true).max(Flicks::ZERO)
            }
            ItemEdge::End => {
                let available = media_duration.map_or(0.0, Flicks::to_seconds_f64) - source_start;
                rounded_to_millisecond(start + available / clip.speed, false)
            }
        }
    }
}

/// `seconds` rounded up or down to the millisecond (after removing sub-microsecond noise).
fn rounded_to_millisecond(seconds: f64, up: bool) -> Flicks {
    let milliseconds = (seconds * 1e6).round() / 1e3;
    let milliseconds = if up {
        milliseconds.ceil()
    } else {
        milliseconds.floor()
    };
    Flicks::from_seconds_f64(milliseconds / 1e3).unwrap_or(Flicks::ZERO)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Animatable, Easing, EasingPreset, Keyframe,
        operation::testing::{V1, apply_err, apply_ok, clip, example, find, items_of},
        test_support::{millis, range_ms},
    };

    fn trim(item: u64, edge: ItemEdge, to_ms: i64) -> TrimItem {
        TrimItem {
            item: ItemId::from_test_index(item),
            edge,
            to: millis(to_ms),
            expected_revision: None,
        }
    }

    /// The example with clip 2 (`[10 s, 14 s)`, 2×) playing its asset from 4 s.
    fn clip_two_from_four_seconds() -> Project {
        let mut project = example();
        if let ItemKind::MediaClip(clip) = &mut items_of(&mut project, V1)[1].kind {
            clip.source_start = millis(4000);
        }
        project
    }

    #[test]
    fn trimming_the_start_moves_the_source_start() {
        let applied = apply_ok(&example(), trim(2, ItemEdge::Start, 11_000));
        let item = find(&applied.project, 2);
        assert_eq!(item.timeline_range, range_ms(11_000, 3000));
        assert_eq!(clip(item).source_start, millis(22_000));
    }

    #[test]
    fn trims_stop_at_the_media() {
        let project = clip_two_from_four_seconds();
        let error = apply_err(&project, trim(2, ItemEdge::Start, 7000));
        assert_eq!(
            error,
            DomainError::TrimBeyondSource {
                item: ItemId::from_test_index(2),
                edge: ItemEdge::Start,
                to: millis(7000),
                limit: millis(8000),
            }
        );
        let mut project = project;
        items_of(&mut project, V1).remove(0);
        project.sequences[0].tracks[0].transitions.clear();
        let applied = apply_ok(&project, trim(2, ItemEdge::Start, 8000));
        assert_eq!(clip(find(&applied.project, 2)).source_start, Flicks::ZERO);
        // Source 4 s + 2 × duration ≤ 60 s: the end can reach 10 s + 28 s.
        let error = apply_err(&project, trim(2, ItemEdge::End, 38_001));
        assert!(
            matches!(error, DomainError::TrimBeyondSource { limit, .. } if limit == millis(38_000)),
            "{error}"
        );
        apply_ok(&project, trim(2, ItemEdge::End, 38_000));
    }

    #[test]
    fn edges_cannot_cross() {
        let error = apply_err(&example(), trim(3, ItemEdge::End, 1000));
        assert_eq!(
            error,
            DomainError::TrimCollapsesItem {
                item: ItemId::from_test_index(3),
                edge: ItemEdge::End,
                to: millis(1000),
                other_edge: millis(1000),
            }
        );
        assert!(
            error
                .to_string()
                .contains("must stay after its other edge at 00:00:01.000")
        );
    }

    #[test]
    fn extending_into_a_neighbour_is_an_overlap() {
        let error = apply_err(&example(), trim(3, ItemEdge::End, 5500));
        assert!(matches!(error, DomainError::Overlap { .. }), "{error}");
    }

    #[test]
    fn keyframes_stay_at_their_instants() {
        // Text 3 `[1 s, 4 s)` fades in with keyframes at 0 and 0.5 s.
        let applied = apply_ok(&example(), trim(3, ItemEdge::Start, 1250));
        let transform = find(&applied.project, 3).kind.transform().unwrap().clone();
        let ease_out = Easing::Preset {
            name: EasingPreset::EaseOut,
        };
        assert_eq!(
            transform.opacity,
            Animatable::Keyframed {
                keyframes: vec![Keyframe {
                    time: millis(250),
                    value: 1.0,
                    easing: ease_out
                }]
            }
        );
        let applied = apply_ok(&example(), trim(3, ItemEdge::Start, 2000));
        let transform = find(&applied.project, 3).kind.transform().unwrap().clone();
        assert_eq!(transform.opacity, Animatable::constant(1.0));
        assert_eq!(
            find(&applied.project, 3).revision,
            1,
            "apply keeps revisions"
        );
    }

    #[test]
    fn fades_shrink_with_the_item() {
        // Clip 1 fades in for 0.5 s.
        let applied = apply_ok(&example(), trim(1, ItemEdge::End, 300));
        assert_eq!(clip(find(&applied.project, 1)).fade_in, millis(300));
    }
}
