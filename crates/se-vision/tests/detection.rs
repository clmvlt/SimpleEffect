//! Real face detection with the embedded YuNet model over the ONNX Runtime library of
//! `pnpm fetch-binaries` (`resources/bin/<platform>/onnxruntime/`). Skipped, with a message, when
//! the library is absent.
//!
//! Fixture: `fixtures/portrait.jpg`, NASA's 1969 portrait of Neil Armstrong (public domain, a work
//! of the US government; Wikimedia Commons `File:Neil_Armstrong_pose.jpg`), resized to 480 × 600.
//! Expected box: OpenCV 5.0's `FaceDetectorYN` on the same file and model,
//! `[257.01, 115.40, 76.97, 101.56]` pixels, score 0.9414.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use std::path::PathBuf;

use se_app::ports::{SampledFrame, SubjectDetector};
use se_domain::Flicks;
use se_vision::{RuntimeLocator, YuNetDetector};

const PORTRAIT: &[u8] = include_bytes!("fixtures/portrait.jpg");

/// A locator over this test binary's folders (they lie under the repository's `target/`).
fn locator() -> RuntimeLocator {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(PathBuf::from));
    RuntimeLocator::new(exe_dir)
}

fn runtime_available() -> bool {
    match locator().locate() {
        Ok(_) => true,
        Err(reason) => {
            eprintln!("skipped: {reason}");
            false
        }
    }
}

/// The portrait as a BGR frame.
fn portrait() -> SampledFrame {
    let mut decoder = jpeg_decoder::Decoder::new(PORTRAIT);
    let rgb = decoder.decode().expect("a readable JPEG");
    let info = decoder.info().expect("a decoded JPEG");
    let pixels = rgb
        .as_chunks::<3>()
        .0
        .iter()
        .flat_map(|&[red, green, blue]| [blue, green, red])
        .collect();
    SampledFrame {
        time: Flicks::ZERO,
        width: u32::from(info.width),
        height: u32::from(info.height),
        pixels,
    }
}

#[tokio::test]
async fn finds_the_face_of_the_test_portrait_like_opencv() {
    if !runtime_available() {
        return;
    }
    let detector = YuNetDetector::new(locator(), 2);
    let status = detector.status().await;
    assert!(status.available, "{status:?}");
    assert!(
        status
            .runtime
            .as_deref()
            .is_some_and(|runtime| runtime.starts_with("ONNX Runtime"))
    );

    let blank = SampledFrame {
        time: Flicks::one_second(),
        width: 320,
        height: 180,
        pixels: vec![128; 320 * 180 * 3],
    };
    let detected = detector.detect(vec![portrait(), blank]).await.unwrap();
    assert_eq!(detected.len(), 2);
    assert_eq!(detected[1], [], "a grey picture has no face");
    let [face] = detected[0].as_slice() else {
        panic!("one face expected, got {:?}", detected[0]);
    };
    // OpenCV: [257.01, 115.40, 76.97, 101.56] px of 480 × 600 (JPEG decoders differ slightly).
    let expected = [
        257.01 / 480.0,
        115.40 / 600.0,
        76.97 / 480.0,
        101.56 / 600.0,
    ];
    let found = [face.x, face.y, face.width, face.height];
    for (value, reference) in found.iter().zip(expected) {
        assert!(
            (value - reference).abs() < 0.006,
            "{found:?} vs {expected:?}"
        );
    }
    assert!((face.score - 0.9414).abs() < 0.02, "{}", face.score);
}

#[tokio::test]
async fn a_malformed_frame_is_refused_with_its_size() {
    if !runtime_available() {
        return;
    }
    let detector = YuNetDetector::new(locator(), 1);
    let broken = SampledFrame {
        time: Flicks::ZERO,
        width: 10,
        height: 10,
        pixels: vec![0; 12],
    };
    let error = detector.detect(vec![broken]).await.unwrap_err();
    assert!(error.to_string().contains("10 × 10"), "{error}");
}
