//! Automatic reframing (prompt 26, ADR-0037): from the faces detected in an asset's pictures
//! ([`SubjectDetection`]) to the focus keyframes of a tracked clip ([`crate::Reframe::Tracked`]).
//!
//! Pure and independent of the detector. The steps, one module each:
//! 1. `shots` — the clip's source span cut at shot changes; samples next to a cut left out.
//! 2. `subjects` — faces linked across a shot's samples into tracks; the main subject
//!    (persistence × size × centrality); group framing when the significant faces fit together.
//! 3. `window` — the part of the picture the frame keeps; the focus that puts the subject's eyes
//!    on the eye line, never letting the window leave the picture.
//! 4. `path` — gaps filled: interpolated inside a shot, held at its edges, centred without faces.
//! 5. `dead_zone`, `smoothing` (One-Euro, forward and backward), `simplify` (Ramer–Douglas–
//!    Peucker within [`simplify::TOLERANCE`]).
//! 6. `keyframes` — item-relative keyframes on the sequence's frames, held before each cut so the
//!    frame jumps to the next shot's subject on the cut's first frame.
//!
//! Interpolating between samples is not the engine's job of interpolating keyframes (the domain
//! still never evaluates animations): it only decides where keyframes go.

mod dead_zone;
mod keyframes;
mod path;
mod shots;
mod simplify;
mod smoothing;
mod subjects;
mod window;

pub use self::window::CropWindow;
use self::{
    dead_zone::held_path,
    keyframes::{ShotPath, focus_animation, on_frame},
    path::{PathPoint, filled},
    shots::split_into_shots,
    simplify::{KeptPoint, TOLERANCE, simplified},
    smoothing::{OneEuroParams, smoothed},
    subjects::shot_targets,
};
use crate::{
    analysis::SubjectDetection,
    document::{Animatable, Easing, Reframe, TrackingSettings},
    spatial::Vec2,
    time::{Flicks, FrameRate, RoundingMode},
};

/// Where a clip sits: what part of its asset it plays and on what timeline.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClipTiming {
    /// Source time of the clip's first frame.
    pub source_start: Flicks,
    /// Playback speed (> 0).
    pub speed: f64,
    /// The item's duration on the timeline.
    pub duration: Flicks,
    /// The sequence's frame rate (keyframes go on its frames).
    pub frame_rate: FrameRate,
}

/// Everything automatic reframing needs for one clip.
#[derive(Debug, Clone, Copy)]
pub struct TrackingRequest<'a> {
    /// The faces found in the clip's asset.
    pub detection: &'a SubjectDetection,
    /// Shot changes of the asset, in source time (scene detection).
    pub scene_changes: &'a [Flicks],
    /// Width over height of the upright picture.
    pub picture_aspect: f64,
    /// Width over height of the sequence's frame.
    pub frame_aspect: f64,
    /// The clip's place.
    pub clip: ClipTiming,
    /// How to follow the subject.
    pub settings: TrackingSettings,
}

/// The result of automatic reframing for one clip.
#[derive(Debug, Clone, PartialEq)]
pub struct TrackedFocus {
    /// The focus to put in [`Reframe::Tracked`].
    pub focus: Animatable<Vec2>,
    /// Number of shots the clip spans.
    pub shots: usize,
    /// Number of samples read.
    pub samples: usize,
    /// Number of samples where the subject (or the group) was seen.
    pub samples_with_subject: usize,
}

/// Computes the focus keyframes that keep the subject framed over the clip of `request`.
#[must_use]
pub fn track_subject(request: &TrackingRequest<'_>) -> TrackedFocus {
    let window = CropWindow::new(request.picture_aspect, request.frame_aspect);
    let clip = request.clip;
    let source_end = clip
        .duration
        .checked_mul_f64(clip.speed)
        .and_then(|span| clip.source_start.checked_add(span))
        .unwrap_or(clip.source_start);
    let shots = split_into_shots(
        request.detection,
        request.scene_changes,
        clip.source_start,
        source_end,
    );
    let params = OneEuroParams::for_smoothing(request.settings.smoothing);
    let centre = window.clamp(Reframe::CENTRE);
    let mut paths: Vec<ShotPath> = Vec::with_capacity(shots.len());
    let (mut samples, mut samples_with_subject) = (0, 0);
    for (index, shot) in shots.iter().enumerate() {
        let targets = shot_targets(shot.frames, window, request.settings.group_framing);
        samples += targets.len();
        samples_with_subject += targets.iter().flatten().count();
        let wanted: Vec<Option<Vec2>> = targets
            .iter()
            .map(|target| target.map(|eyes| window.focus_for(eyes, request.settings.eye_line)))
            .collect();
        let times: Vec<f64> = shot
            .frames
            .iter()
            .map(|frame| item_seconds(frame.time, clip))
            .collect();
        let start = if index == 0 {
            Flicks::ZERO
        } else {
            cut_frame(shot.start, clip)
        };
        let points = shot_points(&times, &wanted, window, request.settings, params);
        let fallback = paths
            .last()
            .and_then(|path| path.points.last())
            .map_or(centre, |point| point.focus);
        paths.push(ShotPath {
            start,
            points: if points.is_empty() {
                vec![KeptPoint {
                    time: start.to_seconds_f64(),
                    focus: fallback,
                    easing: Easing::Linear,
                }]
            } else {
                points
            },
        });
    }
    TrackedFocus {
        focus: focus_animation(&paths, clip.duration, clip.frame_rate),
        shots: shots.len(),
        samples,
        samples_with_subject,
    }
}

/// One shot's kept points: filled, held in the dead zone, smoothed, kept in the picture and
/// simplified.
fn shot_points(
    times: &[f64],
    wanted: &[Option<Vec2>],
    window: CropWindow,
    settings: TrackingSettings,
    params: OneEuroParams,
) -> Vec<KeptPoint> {
    let centre = window.clamp(Reframe::CENTRE);
    let path = filled(times, wanted, centre);
    let held = held_path(&path, window, settings.dead_zone);
    let smooth: Vec<PathPoint> = smoothed(&held, params)
        .into_iter()
        .map(|point| PathPoint {
            time: point.time,
            focus: window.clamp(point.focus),
        })
        .collect();
    simplified(&smooth, window, TOLERANCE)
}

/// Item-relative seconds of the source instant `source`.
fn item_seconds(source: Flicks, clip: ClipTiming) -> f64 {
    let seconds = source
        .checked_sub(clip.source_start)
        .map_or(0.0, Flicks::to_seconds_f64);
    if clip.speed > 0.0 {
        seconds / clip.speed
    } else {
        seconds
    }
}

/// `count` as a float (counts of samples and faces stay far below 2^32).
fn as_float(count: usize) -> f64 {
    f64::from(u32::try_from(count).unwrap_or(u32::MAX))
}

/// The item-relative first frame showing the source instant `cut` or later (exact in flicks at
/// normal speed).
fn cut_frame(cut: Flicks, clip: ClipTiming) -> Flicks {
    let normal_speed = (clip.speed - 1.0).abs() < f64::EPSILON;
    let exact = cut.checked_sub(clip.source_start).filter(|_| normal_speed);
    let time = exact.unwrap_or_else(|| {
        Flicks::from_seconds_f64(item_seconds(cut, clip)).unwrap_or(Flicks::ZERO)
    });
    on_frame(time, clip.frame_rate, RoundingMode::Ceil).min(clip.duration)
}

#[cfg(test)]
mod tests;
