//! The scope check of AI edits (ADR-0029): everything an agent's transaction touches must lie in
//! its task's scope — the sequence, the time range (`null`: the whole sequence) and the tracks
//! (`null`: every track) — **before and after** the edit. Reads are never restricted.
//!
//! The check runs on the whole planned transaction, after the planners: it compares the touched
//! entities in the document before and after, so it also catches what an edit drags along
//! (linked partners, items shifted by a ripple edit, transitions dropped with a moved item).
//! Its messages are read by agents: they name the faulty entity, its place, the scope, and
//! what to do instead.

use se_domain::{
    CaptionStyleId, Item, ItemId, ItemKind, Marker, MarkerId, Project, Sequence, TouchedEntities,
    Track, TrackId, Transition, format_clock,
};
use se_protocol::TaskScope;

use crate::AppError;

/// A planned transaction to check: the document before and after it, what it touched, and
/// whether it shifts time (ripple edits get a hint).
pub(crate) struct PlannedChange<'a> {
    /// The document the transaction starts from.
    pub(crate) before: &'a Project,
    /// The document it leads to.
    pub(crate) after: &'a Project,
    /// What it added, changed or removed.
    pub(crate) touched: &'a TouchedEntities,
    /// Whether it removes or opens time (ripple delete, insert gap).
    pub(crate) shifts_time: bool,
}

/// Refuses `change` unless everything it touches lies in `scope`, before and after.
///
/// # Errors
/// [`AppError::OutOfScope`] naming the first entity outside the scope.
pub(crate) fn check_scope(scope: &TaskScope, change: &PlannedChange<'_>) -> Result<(), AppError> {
    if !change.touched.assets.is_empty() || change.touched.project {
        return Err(out_of_scope(
            "AI tasks cannot change media files or project settings".to_owned(),
        ));
    }
    let checker = Checker { scope, change };
    for &item in &change.touched.items {
        checker.item(item)?;
    }
    for &track in &change.touched.tracks {
        checker.track(track)?;
    }
    for &sequence in &change.touched.sequences {
        checker.sequence(sequence)?;
    }
    for &marker in &change.touched.markers {
        checker.marker(marker)?;
    }
    for &style in &change.touched.caption_styles {
        checker.caption_style(style)?;
    }
    Ok(())
}

/// Whether `item`, on `track` of `sequence`, lies entirely in `scope`.
pub(crate) fn item_in_scope(
    scope: &TaskScope,
    sequence: &Sequence,
    track: &Track,
    item: &Item,
) -> bool {
    sequence.id == scope.sequence_id
        && track_allowed(scope, track.id)
        && scope
            .range
            .is_none_or(|range| range.contains_range(item.timeline_range))
}

/// Whether the task may change items of track `track` (of its sequence).
pub(crate) fn track_allowed(scope: &TaskScope, track: TrackId) -> bool {
    scope
        .tracks
        .as_ref()
        .is_none_or(|tracks| tracks.contains(&track))
}

/// The scope in words: `00:00:51.000–00:00:54.000 of « Main edit », tracks V1, V2`.
pub(crate) fn describe_scope(scope: &TaskScope, project: &Project) -> String {
    let sequence = project
        .sequences
        .iter()
        .find(|sequence| sequence.id == scope.sequence_id);
    let name = sequence.map_or_else(|| scope.sequence_id.to_string(), |s| s.name.clone());
    let time = scope.range.map_or_else(
        || format!("the whole of « {name} »"),
        |range| {
            format!(
                "{}–{} of « {name} »",
                format_clock(range.start()),
                format_clock(range.end())
            )
        },
    );
    let tracks = match (&scope.tracks, sequence) {
        (None, _) => "every track".to_owned(),
        (Some(ids), sequence) => {
            let names: Vec<String> = ids
                .iter()
                .map(|id| {
                    sequence
                        .and_then(|s| s.tracks.iter().find(|track| track.id == *id))
                        .map_or_else(|| id.to_string(), |track| track.name.clone())
                })
                .collect();
            format!("tracks {}", names.join(", "))
        }
    };
    format!("{time}, {tracks}")
}

fn out_of_scope(reason: String) -> AppError {
    AppError::OutOfScope { reason }
}

/// Where an item is.
struct Placed<'a> {
    sequence: &'a Sequence,
    track: &'a Track,
    item: &'a Item,
}

fn find_item(project: &Project, id: ItemId) -> Option<Placed<'_>> {
    project.sequences.iter().find_map(|sequence| {
        sequence.tracks.iter().find_map(|track| {
            let item = track.items.iter().find(|item| item.id == id)?;
            Some(Placed {
                sequence,
                track,
                item,
            })
        })
    })
}

fn find_track(project: &Project, id: TrackId) -> Option<(&Sequence, &Track)> {
    project.sequences.iter().find_map(|sequence| {
        let track = sequence.tracks.iter().find(|track| track.id == id)?;
        Some((sequence, track))
    })
}

fn find_marker(project: &Project, id: MarkerId) -> Option<(&Sequence, &Marker)> {
    project.sequences.iter().find_map(|sequence| {
        let marker = sequence.markers.iter().find(|marker| marker.id == id)?;
        Some((sequence, marker))
    })
}

/// `text « Abonne-toi » 0193… on V2, 00:00:51.000–00:00:54.000`.
fn describe_item(placed: &Placed<'_>) -> String {
    let range = placed.item.timeline_range;
    format!(
        "{} {} on {} of « {} », {}–{}",
        kind_name(&placed.item.kind),
        placed.item.id,
        placed.track.name,
        placed.sequence.name,
        format_clock(range.start()),
        format_clock(range.end())
    )
}

fn kind_name(kind: &ItemKind) -> String {
    match kind {
        ItemKind::MediaClip(_) => "media clip".to_owned(),
        ItemKind::Text(text) => {
            let excerpt: String = text.content.chars().take(40).collect();
            format!("text « {excerpt} »")
        }
        ItemKind::Shape(_) => "shape".to_owned(),
        ItemKind::Image(_) => "image".to_owned(),
        ItemKind::MotionTemplate(_) => "motion template".to_owned(),
        ItemKind::Caption(caption) => {
            let excerpt: String = caption.text().chars().take(40).collect();
            format!("caption « {excerpt} »")
        }
    }
}

struct Checker<'a> {
    scope: &'a TaskScope,
    change: &'a PlannedChange<'a>,
}

impl Checker<'_> {
    fn scope_text(&self) -> String {
        describe_scope(self.scope, self.change.before)
    }

    /// What keeps an edit inside the scope.
    fn advice(&self) -> String {
        let mut advice = match self.scope.range {
            Some(range) => format!(
                "keep every change between {} and {}",
                format_clock(range.start()),
                format_clock(range.end())
            ),
            None => "keep every change on the task's sequence and tracks".to_owned(),
        };
        if self.change.shifts_time && self.scope.range.is_some() {
            advice.push_str(
                "; ripple edits move everything after the edit point, so with a scope limited in \
                 time use delete_items (it leaves a gap) or trim_item instead, or ask the user \
                 for a task covering the rest of the sequence",
            );
        }
        advice
    }

    fn item(&self, id: ItemId) -> Result<(), AppError> {
        if let Some(placed) = find_item(self.change.before, id)
            && !item_in_scope(self.scope, placed.sequence, placed.track, placed.item)
        {
            return Err(out_of_scope(format!(
                "{} is outside this task's scope ({}): only items lying entirely inside the scope \
                 can be changed, so leave it as it is ({})",
                describe_item(&placed),
                self.scope_text(),
                self.advice()
            )));
        }
        if let Some(placed) = find_item(self.change.after, id)
            && !item_in_scope(self.scope, placed.sequence, placed.track, placed.item)
        {
            return Err(out_of_scope(format!(
                "the edit would leave {} outside this task's scope ({}): {}",
                describe_item(&placed),
                self.scope_text(),
                self.advice()
            )));
        }
        Ok(())
    }

    /// A caption style may change only when every caption using it, before and after, lies in
    /// the scope (its look is theirs).
    fn caption_style(&self, style: CaptionStyleId) -> Result<(), AppError> {
        for project in [self.change.before, self.change.after] {
            let users = project.sequences.iter().flat_map(|sequence| {
                sequence.tracks.iter().flat_map(move |track| {
                    track.items.iter().map(move |item| Placed {
                        sequence,
                        track,
                        item,
                    })
                })
            });
            for placed in users {
                let styled = matches!(&placed.item.kind, ItemKind::Caption(caption) if caption.style_id == style);
                if styled && !item_in_scope(self.scope, placed.sequence, placed.track, placed.item)
                {
                    return Err(out_of_scope(format!(
                        "caption style {style} is also used by {}, outside this task's scope                          ({}): change the style of the captions of the scope only (list them in                          `items`: they get a style of their own)",
                        describe_item(&placed),
                        self.scope_text()
                    )));
                }
            }
        }
        Ok(())
    }

    fn track(&self, id: TrackId) -> Result<(), AppError> {
        let before = find_track(self.change.before, id);
        let after = find_track(self.change.after, id);
        match (before, after) {
            (None, Some((sequence, track))) => {
                if sequence.id != self.scope.sequence_id || self.scope.tracks.is_some() {
                    return Err(out_of_scope(format!(
                        "the edit would add track {} to « {} », but this task may only change {}: \
                         use one of its tracks",
                        track.name,
                        sequence.name,
                        self.scope_text()
                    )));
                }
                Ok(())
            }
            (Some((_, track)), None) => Err(out_of_scope(format!(
                "AI tasks cannot remove tracks (the edit would remove track {})",
                track.name
            ))),
            (Some((sequence, old)), Some((_, new))) => {
                if !same_header(old, new) {
                    return Err(out_of_scope(format!(
                        "AI tasks cannot change track headers (name, mute, solo, lock, \
                         visibility, volume, pan): the edit would change track {}",
                        old.name
                    )));
                }
                if sequence.id != self.scope.sequence_id || !track_allowed(self.scope, id) {
                    return Err(out_of_scope(format!(
                        "track {} of « {} » is outside this task's scope ({})",
                        old.name,
                        sequence.name,
                        self.scope_text()
                    )));
                }
                self.transitions(old, new)
            }
            (None, None) => Ok(()),
        }
    }

    /// Every transition added, changed or removed on a track must join two items in scope.
    fn transitions(&self, old: &Track, new: &Track) -> Result<(), AppError> {
        let changed = |from: &Track, to: &Track| -> Vec<Transition> {
            from.transitions
                .iter()
                .filter(|transition| !to.transitions.contains(transition))
                .cloned()
                .collect()
        };
        for transition in changed(old, new).into_iter().chain(changed(new, old)) {
            for item in [transition.left_item, transition.right_item] {
                let placed = find_item(self.change.after, item)
                    .or_else(|| find_item(self.change.before, item));
                if let Some(placed) = placed
                    && !item_in_scope(self.scope, placed.sequence, placed.track, placed.item)
                {
                    return Err(out_of_scope(format!(
                        "the edit would change the transition {} touching {}, which is outside \
                         this task's scope ({}): {}",
                        transition.id,
                        describe_item(&placed),
                        self.scope_text(),
                        self.advice()
                    )));
                }
            }
        }
        Ok(())
    }

    fn sequence(&self, id: se_domain::SequenceId) -> Result<(), AppError> {
        let find = |project: &Project| -> Option<Sequence> {
            project
                .sequences
                .iter()
                .find(|sequence| sequence.id == id)
                .cloned()
        };
        let (before, after) = (find(self.change.before), find(self.change.after));
        let name = before
            .as_ref()
            .or(after.as_ref())
            .map_or_else(|| id.to_string(), |sequence| sequence.name.clone());
        match (before, after) {
            (Some(old), Some(new)) if id == self.scope.sequence_id => {
                if old.name == new.name && old.format == new.format {
                    Ok(())
                } else {
                    Err(out_of_scope(format!(
                        "AI tasks cannot rename or reformat their sequence « {name} »"
                    )))
                }
            }
            _ => Err(out_of_scope(format!(
                "sequence « {name} » is outside this task's scope ({}): AI tasks change only \
                 their own sequence and never add or remove sequences",
                self.scope_text()
            ))),
        }
    }

    fn marker(&self, id: MarkerId) -> Result<(), AppError> {
        let markers = [
            find_marker(self.change.before, id),
            find_marker(self.change.after, id),
        ];
        for (sequence, marker) in markers.into_iter().flatten() {
            let inside = sequence.id == self.scope.sequence_id
                && self
                    .scope
                    .range
                    .is_none_or(|range| range.contains(marker.time));
            if !inside {
                return Err(out_of_scope(format!(
                    "marker « {} » at {} of « {} » is outside this task's scope ({}): {}",
                    marker.label,
                    format_clock(marker.time),
                    sequence.name,
                    self.scope_text(),
                    self.advice()
                )));
            }
        }
        Ok(())
    }
}

/// Whether two versions of a track have the same header (everything but items and transitions).
fn same_header(old: &Track, new: &Track) -> bool {
    old.name == new.name
        && old.kind == new.kind
        && old.muted == new.muted
        && old.solo == new.solo
        && old.locked == new.locked
        && old.hidden == new.hidden
        && old.volume_db.to_bits() == new.volume_db.to_bits()
        && old.pan.to_bits() == new.pan.to_bits()
}
