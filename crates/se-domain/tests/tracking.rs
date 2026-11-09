//! Automatic reframing (prompt 26): properties of [`track_subject`] on random detections, shot
//! changes, clips and settings — keyframes inside the item and on increasing times, focuses
//! inside the picture, curved easings without overshoot, a hold before every cut, and a tracked
//! clip that the domain accepts.

#![allow(clippy::expect_used, clippy::panic)]

use proptest::prelude::*;
use se_domain::{
    Animatable, ClipTiming, CropWindow, Easing, FaceBox, Flicks, FrameRate, ItemKind, Reframe,
    SubjectDetection, SubjectFrame, TrackingRequest, TrackingSettings, Vec2,
    test_support::example_project, track_subject, validate_project,
};

/// A face moving along a random walk, sometimes missing, with a random second face.
#[derive(Debug, Clone)]
struct Scene {
    start_x: f64,
    steps: Vec<f64>,
    missing_every: u32,
    second_face: bool,
}

fn scenes() -> impl Strategy<Value = Scene> {
    (
        0.0_f64..1.0,
        prop::collection::vec(-0.03_f64..0.03, 150),
        2_u32..12,
        any::<bool>(),
    )
        .prop_map(|(start_x, steps, missing_every, second_face)| Scene {
            start_x,
            steps,
            missing_every,
            second_face,
        })
}

/// 30 s of samples every 200 ms.
fn detection(scene: &Scene) -> SubjectDetection {
    let mut x = scene.start_x;
    let frames = scene
        .steps
        .iter()
        .zip(0_u32..)
        .map(|(step, index)| {
            x = (x + step).clamp(0.0, 0.9);
            let mut faces = Vec::new();
            if index % scene.missing_every != 0 {
                faces.push(FaceBox {
                    x,
                    y: 0.2,
                    width: 0.1,
                    height: 0.14,
                    score: 0.9,
                });
            }
            if scene.second_face {
                faces.push(FaceBox {
                    x: 0.9 - x,
                    y: 0.3,
                    width: 0.06,
                    height: 0.08,
                    score: 0.8,
                });
            }
            SubjectFrame {
                time: Flicks::from_millis(index * 200),
                faces,
            }
        })
        .collect();
    SubjectDetection {
        model: "test".to_owned(),
        interval: Flicks::from_millis(200),
        frames,
    }
}

fn settings() -> impl Strategy<Value = TrackingSettings> {
    (0.0_f64..=1.0, 0.2_f64..=0.8, any::<bool>(), 0.0_f64..=0.5).prop_map(
        |(smoothing, eye_line, group_framing, dead_zone)| TrackingSettings {
            smoothing,
            eye_line,
            group_framing,
            dead_zone,
        },
    )
}

fn frame_rates() -> impl Strategy<Value = FrameRate> {
    prop::sample::select(vec![
        FrameRate::new(24, 1).expect("rate"),
        FrameRate::new(25, 1).expect("rate"),
        FrameRate::new(30_000, 1001).expect("rate"),
        FrameRate::new(60, 1).expect("rate"),
    ])
}

fn keyframes(focus: &Animatable<Vec2>) -> Vec<se_domain::Keyframe<Vec2>> {
    match focus {
        Animatable::Keyframed { keyframes } => keyframes.clone(),
        Animatable::Static { .. } => Vec::new(),
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(400))]

    #[test]
    fn tracked_focuses_are_valid_keyframes_inside_the_picture(
        scene in scenes(),
        cut_times in prop::collection::btree_set(1_u32..29, 0..5),
        start_ms in 0_u32..10_000,
        duration_ms in 400_u32..15_000,
        speed in prop::sample::select(vec![0.5, 1.0, 1.5, 2.0]),
        frame_rate in frame_rates(),
        portrait in any::<bool>(),
        settings in settings(),
    ) {
        let detection = detection(&scene);
        let cuts: Vec<Flicks> = cut_times.iter().map(|second| Flicks::from_millis(second * 1000)).collect();
        let (picture_aspect, frame_aspect) = if portrait { (9.0 / 16.0, 16.0 / 9.0) } else { (16.0 / 9.0, 9.0 / 16.0) };
        let duration = Flicks::from_millis(duration_ms);
        let clip = ClipTiming { source_start: Flicks::from_millis(start_ms), speed, duration, frame_rate };
        let result = track_subject(&TrackingRequest {
            detection: &detection,
            scene_changes: &cuts,
            picture_aspect,
            frame_aspect,
            clip,
            settings,
        });
        let window = CropWindow::new(picture_aspect, frame_aspect);
        let values: Vec<Vec2> = match &result.focus {
            Animatable::Static { value } => vec![*value],
            Animatable::Keyframed { keyframes } => keyframes.iter().map(|keyframe| keyframe.value).collect(),
        };
        for value in &values {
            let clamped = window.clamp(*value);
            prop_assert!((clamped.x - value.x).abs() <= 1e-4 && (clamped.y - value.y).abs() <= 1e-4, "{value:?}");
        }
        let frames = keyframes(&result.focus);
        prop_assert!(frames.windows(2).all(|pair| pair[0].time < pair[1].time));
        prop_assert!(frames.iter().all(|keyframe| keyframe.time >= Flicks::ZERO && keyframe.time <= duration));
        for keyframe in &frames {
            if let Easing::CubicBezier { x1, y1, x2, y2 } = keyframe.easing {
                prop_assert!((0.0..=1.0).contains(&x1) && (0.0..=1.0).contains(&x2));
                prop_assert!((0.0..=1.0).contains(&y1) && (0.0..=1.0).contains(&y2), "{:?}", keyframe.easing);
            }
        }
        // Every cut inside the item: a keyframe on its first frame, held before it.
        if !frames.is_empty() {
            let source_end = f64::from(duration_ms).mul_add(speed, f64::from(start_ms));
            for cut in &cut_times {
                let cut_ms = f64::from(cut * 1000);
                if cut_ms <= f64::from(start_ms) || cut_ms >= source_end {
                    continue;
                }
                let item = Flicks::from_seconds_f64((cut_ms - f64::from(start_ms)) / 1000.0 / speed).expect("time");
                let first_frame = Flicks::from_frames(item.to_frames(frame_rate, se_domain::RoundingMode::Ceil), frame_rate).expect("frame").min(duration);
                if first_frame == Flicks::ZERO || first_frame == duration {
                    continue;
                }
                let at = frames.iter().position(|keyframe| keyframe.time == first_frame);
                prop_assert!(at.is_some(), "no keyframe on the cut at {first_frame:?}");
                let before = at.and_then(|index| index.checked_sub(1)).and_then(|index| frames.get(index));
                prop_assert_eq!(before.map(|keyframe| keyframe.easing), Some(Easing::Hold));
            }
        }
        prop_assert!(result.samples_with_subject <= result.samples);
    }

    #[test]
    fn a_tracked_clip_is_a_valid_document(scene in scenes(), settings in settings()) {
        let mut project = example_project();
        let item = &mut project.sequences[0].tracks[0].items[1];
        let duration = item.timeline_range.duration();
        let ItemKind::MediaClip(clip) = &mut item.kind else {
            panic!("the example's second V1 item is a media clip");
        };
        let detection = detection(&scene);
        let result = track_subject(&TrackingRequest {
            detection: &detection,
            scene_changes: &[Flicks::from_millis(23_000)],
            picture_aspect: 16.0 / 9.0,
            frame_aspect: 9.0 / 16.0,
            clip: ClipTiming {
                source_start: clip.source_start,
                speed: clip.speed,
                duration,
                frame_rate: FrameRate::new(25, 1).expect("rate"),
            },
            settings,
        });
        clip.reframe = Reframe::Tracked { focus: result.focus, tracking: settings };
        prop_assert_eq!(validate_project(&project), vec![]);
    }
}
