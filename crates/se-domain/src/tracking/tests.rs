//! Scenarios of automatic reframing on simulated detections (16:9 interviews into 9:16).

use super::*;
use crate::{
    analysis::{FaceBox, SubjectFrame},
    document::{Easing, Keyframe},
};

const LANDSCAPE: f64 = 16.0 / 9.0;
const PORTRAIT: f64 = 9.0 / 16.0;
/// Half the width of a 9:16 window in a 16:9 picture.
const HALF_WINDOW: f64 = 81.0 / 512.0;

fn face(centre_x: f64) -> FaceBox {
    FaceBox {
        x: centre_x - 0.06,
        y: 0.25,
        width: 0.12,
        height: 0.16,
        score: 0.92,
    }
}

/// Samples every 200 ms over `seconds`, the face given by `at` (seconds → centre x, or none).
fn detection(seconds: u32, at: impl Fn(f64) -> Option<f64>) -> SubjectDetection {
    SubjectDetection {
        model: "test".to_owned(),
        interval: Flicks::from_millis(200),
        frames: (0..=seconds * 5)
            .map(|index| {
                let time = Flicks::from_millis(index * 200);
                SubjectFrame {
                    time,
                    faces: at(time.to_seconds_f64()).map(face).into_iter().collect(),
                }
            })
            .collect(),
    }
}

fn timing(start_ms: u32, duration_ms: u32, speed: f64) -> ClipTiming {
    ClipTiming {
        source_start: Flicks::from_millis(start_ms),
        speed,
        duration: Flicks::from_millis(duration_ms),
        frame_rate: FrameRate::new(25, 1).unwrap(),
    }
}

fn track(detection: &SubjectDetection, cuts: &[Flicks], clip: ClipTiming) -> TrackedFocus {
    track_subject(&TrackingRequest {
        detection,
        scene_changes: cuts,
        picture_aspect: LANDSCAPE,
        frame_aspect: PORTRAIT,
        clip,
        settings: TrackingSettings::default(),
    })
}

fn keyframes(focus: &Animatable<Vec2>) -> Vec<Keyframe<Vec2>> {
    match focus {
        Animatable::Keyframed { keyframes } => keyframes.clone(),
        Animatable::Static { value } => vec![Keyframe {
            time: Flicks::ZERO,
            value: *value,
            easing: Easing::Linear,
        }],
    }
}

/// The focus x at item time `seconds` (linear between keyframes, holds respected).
fn x_at(focus: &Animatable<Vec2>, seconds: f64) -> f64 {
    let keyframes = keyframes(focus);
    let time = Flicks::from_seconds_f64(seconds).unwrap();
    let after = keyframes.partition_point(|keyframe| keyframe.time <= time);
    let Some(previous) = after.checked_sub(1).and_then(|index| keyframes.get(index)) else {
        return keyframes[0].value.x;
    };
    match keyframes.get(after) {
        Some(next) if previous.easing != Easing::Hold => {
            let elapsed = time.checked_sub(previous.time).unwrap().to_seconds_f64();
            let share = elapsed
                / next
                    .time
                    .checked_sub(previous.time)
                    .unwrap()
                    .to_seconds_f64();
            let progress = match previous.easing {
                Easing::CubicBezier { y1, y2, .. } => {
                    let rest = 1.0 - share;
                    3.0 * rest * share * (rest * y1 + share * y2) + share.powi(3)
                }
                _ => share,
            };
            (next.value.x - previous.value.x).mul_add(progress, previous.value.x)
        }
        _ => previous.value.x,
    }
}

#[test]
fn a_steady_interview_keeps_the_face_centred_with_few_keyframes() {
    // The speaker sits left of centre and sways a little.
    let detection = detection(10, |t| Some(0.32 + 0.01 * (t * 3.0).sin()));
    let result = track(&detection, &[], timing(0, 10_000, 1.0));
    assert_eq!(result.shots, 1);
    assert_eq!(result.samples, 51);
    assert_eq!(result.samples_with_subject, 51);
    assert!(keyframes(&result.focus).len() <= 3, "{:?}", result.focus);
    for second in 0..10 {
        let x = x_at(&result.focus, f64::from(second));
        assert!((x - 0.32).abs() < 0.03, "{second}: {x}");
    }
}

#[test]
fn a_moving_subject_is_followed_smoothly() {
    // Walks from 0.3 to 0.7 between 2 s and 6 s.
    let detection = detection(8, |t| Some(0.3 + 0.4 * ((t - 2.0) / 4.0).clamp(0.0, 1.0)));
    let result = track(&detection, &[], timing(0, 8000, 1.0));
    let xs: Vec<f64> = (0..=16)
        .map(|step| x_at(&result.focus, f64::from(step) * 0.5))
        .collect();
    assert!(
        (xs[0] - 0.3).abs() < 0.03 && (xs[16] - 0.7).abs() < 0.03,
        "{xs:?}"
    );
    assert!(
        xs.windows(2).all(|pair| pair[1] >= pair[0] - 1e-9),
        "{xs:?}"
    );
    // No step larger than a fifth of the window per half second (no jolt).
    assert!(
        xs.windows(2)
            .all(|pair| pair[1] - pair[0] < 0.2 * 2.0 * HALF_WINDOW),
        "{xs:?}"
    );
    assert!(keyframes(&result.focus).len() < 20);
}

#[test]
fn the_frame_jumps_to_the_next_subject_on_the_cut() {
    // Shot 1: face on the left until 3 s; shot 2: face on the right.
    let detection = detection(6, |t| Some(if t < 3.0 { 0.25 } else { 0.75 }));
    let cut = Flicks::from_millis(3000);
    let result = track(&detection, &[cut], timing(0, 6000, 1.0));
    assert_eq!(result.shots, 2);
    let frames = keyframes(&result.focus);
    let at_cut = frames
        .iter()
        .position(|keyframe| keyframe.time == cut)
        .expect("a keyframe on the cut's first frame");
    assert_eq!(frames[at_cut - 1].easing, Easing::Hold);
    assert!((x_at(&result.focus, 2.96) - 0.25).abs() < 0.02);
    assert!((x_at(&result.focus, 3.0) - 0.75).abs() < 0.02);
    // No drift towards the next subject before the cut.
    assert!(
        frames[..at_cut]
            .iter()
            .all(|keyframe| keyframe.value.x < 0.3)
    );
}

#[test]
fn a_cut_missed_by_scene_detection_is_read_from_the_faces() {
    // The picture changes between the samples at 3.0 s and 3.2 s; scene detection saw nothing.
    let detection = detection(6, |t| Some(if t < 3.1 { 0.25 } else { 0.75 }));
    let result = track(&detection, &[], timing(0, 6000, 1.0));
    assert_eq!(result.shots, 2);
    let frames = keyframes(&result.focus);
    // Halfway between the samples (3.1 s), on the next frame at 25 fps.
    let cut = Flicks::from_millis(3120);
    let at_cut = frames
        .iter()
        .position(|keyframe| keyframe.time == cut)
        .expect("a keyframe on the inferred cut");
    assert_eq!(frames[at_cut - 1].easing, Easing::Hold);
    assert!((x_at(&result.focus, 3.0) - 0.25).abs() < 0.02);
    assert!((x_at(&result.focus, 3.2) - 0.75).abs() < 0.02);
}

#[test]
fn a_face_at_the_edge_keeps_the_window_inside_the_picture() {
    let detection = detection(4, |_| Some(0.04));
    let result = track(&detection, &[], timing(0, 4000, 1.0));
    let value = keyframes(&result.focus)[0].value;
    assert!((value.x - HALF_WINDOW).abs() < 1e-4, "{value:?}");
    assert_eq!(value.y, 0.5);
}

#[test]
fn without_faces_the_frame_stays_centred() {
    let detection = detection(4, |_| None);
    let result = track(&detection, &[], timing(0, 4000, 1.0));
    assert_eq!(result.focus, Animatable::constant(Vec2::new(0.5, 0.5)));
    assert_eq!(result.samples_with_subject, 0);
}

#[test]
fn speed_and_source_start_map_samples_to_the_item() {
    // The clip plays 4 s → 12 s of the source at 2× (4 s on the timeline); the face moves at 8 s.
    let detection = detection(14, |t| Some(if t < 8.0 { 0.3 } else { 0.7 }));
    let cut = Flicks::from_millis(8000);
    let result = track(&detection, &[cut], timing(4000, 4000, 2.0));
    let frames = keyframes(&result.focus);
    assert!(
        frames
            .iter()
            .any(|keyframe| keyframe.time == Flicks::from_millis(2000))
    );
    assert_eq!(
        frames.last().map(|keyframe| keyframe.time),
        Some(Flicks::from_millis(4000))
    );
}

#[test]
fn a_portrait_picture_in_a_landscape_frame_uses_the_eye_line() {
    let detection = detection(2, |_| Some(0.5));
    let result = track_subject(&TrackingRequest {
        detection: &detection,
        scene_changes: &[],
        picture_aspect: PORTRAIT,
        frame_aspect: LANDSCAPE,
        clip: timing(0, 2000, 1.0),
        settings: TrackingSettings::default(),
    });
    let value = keyframes(&result.focus)[0].value;
    let window = CropWindow::new(PORTRAIT, LANDSCAPE);
    let eyes = 0.25 + 0.16 * FaceBox::EYE_HEIGHT;
    let top = value.y - window.height / 2.0;
    assert!(
        (top + 0.33 * window.height - eyes).abs() < 1e-3,
        "{value:?}"
    );
}
