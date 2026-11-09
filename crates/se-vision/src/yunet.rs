//! The YuNet face detector (ADR-0037): the bundled model, its input tensor and the decoding of
//! its outputs into face boxes. Pure: inference itself lives in `detector.rs`.
//!
//! Model: `face_detection_yunet_2026may.onnx` of the OpenCV Zoo (MIT licence, Shiqi Yu; source
//! commit `26cc381e4d2594bb9f47a26eb8fd96c94a13660d`, SHA-256 [`MODEL_SHA256`]), with dynamic input
//! dimensions. Input `input`: `[1, 3, H, W]` float BGR in 0–255, H and W multiples of 32. Outputs,
//! for each stride 8, 16 and 32 over a `(W / stride) × (H / stride)` grid: `cls_<s>` and
//! `obj_<s>` (`[1, cells, 1]`), `bbox_<s>` (`[1, cells, 4]`: centre offsets in cells, log sizes),
//! `kps_<s>` (landmarks, unused). The decoding reproduces OpenCV's `FaceDetectorYN` exactly
//! (checked against OpenCV 5.0 on the test portrait).

use se_app::ports::SampledFrame;
use se_domain::FaceBox;

/// Name stored with detections: results of another model are computed again.
pub const MODEL_NAME: &str = "yunet-2026may";

/// The model, embedded in the binary (229,738 bytes).
pub const MODEL_BYTES: &[u8] = include_bytes!("../models/face_detection_yunet_2026may.onnx");

/// SHA-256 of [`MODEL_BYTES`], as published by the OpenCV Zoo (checked by a test).
pub const MODEL_SHA256: &str = "ebafce4e3c118d6554634be5c27ab333b4c047a9a8c3faf1d7cf93101c22f0f0";

/// The strides of the model's three outputs.
pub const STRIDES: [u32; 3] = [8, 16, 32];

/// Faces below this confidence are dropped.
pub const SCORE_THRESHOLD: f32 = 0.6;

/// Overlap (intersection over union) above which the less confident of two faces is dropped.
pub const NMS_THRESHOLD: f32 = 0.3;

/// At most this many faces per picture (the most confident).
pub const MAX_FACES: usize = 20;

/// The input of one picture: its padded size and the tensor's values (NCHW, BGR, 0–255).
#[derive(Debug, Clone, PartialEq)]
pub struct InputTensor {
    /// Padded width (a multiple of 32).
    pub width: u32,
    /// Padded height (a multiple of 32).
    pub height: u32,
    /// `3 × height × width` values, plane by plane (blue, green, red).
    pub values: Vec<f32>,
}

/// `value` rounded up to a multiple of 32 (at least 32).
#[must_use]
pub const fn padded(value: u32) -> u32 {
    if value == 0 {
        32
    } else {
        value.div_ceil(32) * 32
    }
}

/// The tensor of `frame`: its BGR pixels as planes, zero-padded right and below to multiples of
/// 32. `None` when the pixel buffer does not hold `width × height × 3` bytes.
#[must_use]
pub fn input_tensor(frame: &SampledFrame) -> Option<InputTensor> {
    let width = usize::try_from(frame.width).ok()?;
    let height = usize::try_from(frame.height).ok()?;
    if frame.pixels.len() != width.checked_mul(height)?.checked_mul(3)? || width == 0 || height == 0
    {
        return None;
    }
    let padded_width = padded(frame.width);
    let padded_height = padded(frame.height);
    let plane_width = usize::try_from(padded_width).ok()?;
    let plane = plane_width * usize::try_from(padded_height).ok()?;
    let mut values = vec![0.0_f32; plane * 3];
    for (row, line) in frame.pixels.chunks_exact(width * 3).enumerate() {
        for (column, pixel) in line.as_chunks::<3>().0.iter().enumerate() {
            let offset = row * plane_width + column;
            for (channel, &byte) in pixel.iter().enumerate() {
                if let Some(value) = values.get_mut(channel * plane + offset) {
                    *value = f32::from(byte);
                }
            }
        }
    }
    Some(InputTensor {
        width: padded_width,
        height: padded_height,
        values,
    })
}

/// The scores and boxes of one stride (flat slices of the model's outputs).
#[derive(Debug, Clone, Copy)]
pub struct StrideOutputs<'a> {
    /// 8, 16 or 32.
    pub stride: u32,
    /// Class score per cell.
    pub cls: &'a [f32],
    /// Objectness per cell.
    pub obj: &'a [f32],
    /// Four box values per cell.
    pub bbox: &'a [f32],
}

/// A face in pixels of the padded input, before normalisation.
#[derive(Debug, Clone, Copy, PartialEq)]
struct PixelFace {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    score: f32,
}

/// The faces of a picture of `frame_width × frame_height` pixels whose padded tensor was
/// `padded_width` wide, from the model's outputs: scored, kept above [`SCORE_THRESHOLD`],
/// de-duplicated (non-maximum suppression), the [`MAX_FACES`] most confident first, normalised in
/// the picture and clamped to it.
#[must_use]
pub fn decode(
    outputs: &[StrideOutputs<'_>],
    padded_width: u32,
    frame_width: u32,
    frame_height: u32,
) -> Vec<FaceBox> {
    let mut faces: Vec<PixelFace> = outputs
        .iter()
        .flat_map(|stride| stride_faces(stride, padded_width))
        .collect();
    faces.sort_by(|a, b| b.score.total_cmp(&a.score));
    let mut kept: Vec<PixelFace> = Vec::new();
    for face in faces {
        if kept.len() == MAX_FACES {
            break;
        }
        if kept
            .iter()
            .all(|other| overlap(&face, other) <= NMS_THRESHOLD)
        {
            kept.push(face);
        }
    }
    kept.into_iter()
        .filter_map(|face| normalised(face, frame_width, frame_height))
        .collect()
}

/// The faces of one stride's grid scoring at least [`SCORE_THRESHOLD`].
fn stride_faces(outputs: &StrideOutputs<'_>, padded_width: u32) -> Vec<PixelFace> {
    let columns = usize::try_from((padded_width / outputs.stride).max(1)).unwrap_or(1);
    let size = small_float(outputs.stride);
    outputs
        .cls
        .iter()
        .zip(outputs.obj)
        .zip(outputs.bbox.as_chunks::<4>().0)
        .enumerate()
        .filter_map(|(cell, ((&cls, &obj), &[dx, dy, log_width, log_height]))| {
            let score = (cls.clamp(0.0, 1.0) * obj.clamp(0.0, 1.0)).sqrt();
            if score < SCORE_THRESHOLD {
                return None;
            }
            let column = small_float(u32::try_from(cell % columns).ok()?);
            let row = small_float(u32::try_from(cell / columns).ok()?);
            let width = log_width.exp() * size;
            let height = log_height.exp() * size;
            Some(PixelFace {
                x: (column + dx) * size - width / 2.0,
                y: (row + dy) * size - height / 2.0,
                width,
                height,
                score,
            })
        })
        .collect()
}

/// `value` as a float: grid positions and strides stay far below 2^24, where `f32` is exact.
fn small_float(value: u32) -> f32 {
    u16::try_from(value).map_or(f32::MAX, f32::from)
}

/// Intersection over union of two faces.
fn overlap(a: &PixelFace, b: &PixelFace) -> f32 {
    let width = (a.x + a.width).min(b.x + b.width) - a.x.max(b.x);
    let height = (a.y + a.height).min(b.y + b.height) - a.y.max(b.y);
    let inter = width.max(0.0) * height.max(0.0);
    let union = a.width * a.height + b.width * b.height - inter;
    if union > 0.0 { inter / union } else { 0.0 }
}

/// `face` normalised in a `width × height` picture, clamped to it and rounded to four decimals;
/// `None` when nothing of it lies inside.
fn normalised(face: PixelFace, width: u32, height: u32) -> Option<FaceBox> {
    let (width, height) = (f64::from(width), f64::from(height));
    let left = (f64::from(face.x) / width).clamp(0.0, 1.0);
    let top = (f64::from(face.y) / height).clamp(0.0, 1.0);
    let right = (f64::from(face.x + face.width) / width).clamp(0.0, 1.0);
    let bottom = (f64::from(face.y + face.height) / height).clamp(0.0, 1.0);
    (right > left && bottom > top).then(|| FaceBox {
        x: round(left),
        y: round(top),
        width: round(right - left),
        height: round(bottom - top),
        score: round(f64::from(face.score)),
    })
}

fn round(value: f64) -> f64 {
    (value * 10_000.0).round() / 10_000.0
}

#[cfg(test)]
mod tests {
    use se_domain::Flicks;
    use sha2::{Digest, Sha256};

    use super::*;

    #[test]
    fn the_embedded_model_is_the_published_one() {
        assert_eq!(hex::encode(Sha256::digest(MODEL_BYTES)), MODEL_SHA256);
    }

    #[test]
    fn sizes_are_padded_to_multiples_of_32() {
        assert_eq!(padded(640), 640);
        assert_eq!(padded(360), 384);
        assert_eq!(padded(1), 32);
        assert_eq!(padded(0), 32);
    }

    #[test]
    fn the_tensor_holds_bgr_planes_padded_with_zeros() {
        // 2 × 1 picture: blue-ish then red-ish pixel.
        let frame = SampledFrame {
            time: Flicks::ZERO,
            width: 2,
            height: 1,
            pixels: vec![200, 10, 20, 5, 15, 250],
        };
        let tensor = input_tensor(&frame).unwrap();
        assert_eq!((tensor.width, tensor.height), (32, 32));
        let plane = 32 * 32;
        assert_eq!(tensor.values.len(), plane * 3);
        assert_eq!(&tensor.values[0..3], &[200.0, 5.0, 0.0]);
        assert_eq!(tensor.values[plane], 10.0);
        assert_eq!(tensor.values[2 * plane + 1], 250.0);
        assert_eq!(tensor.values[32], 0.0);
        let broken = SampledFrame {
            pixels: vec![0; 5],
            ..frame
        };
        assert_eq!(input_tensor(&broken), None);
    }

    /// One stride-32 grid of 2 × 2 cells over a 64 × 64 input.
    fn grid(cls: [f32; 4], obj: [f32; 4], bbox: [[f32; 4]; 4]) -> (Vec<f32>, Vec<f32>, Vec<f32>) {
        (
            cls.to_vec(),
            obj.to_vec(),
            bbox.iter().flatten().copied().collect(),
        )
    }

    #[test]
    fn cells_become_faces_like_opencv_decodes_them() {
        let (cls, obj, bbox) = grid(
            [0.0, 0.81, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [[0.0; 4], [0.5, 0.5, 0.0, 0.0], [0.0; 4], [0.0; 4]],
        );
        let outputs = [StrideOutputs {
            stride: 32,
            cls: &cls,
            obj: &obj,
            bbox: &bbox,
        }];
        let faces = decode(&outputs, 64, 64, 64);
        // Cell (column 1, row 0), centre at (1.5 × 32, 0.5 × 32), size 32 × 32.
        assert_eq!(
            faces,
            [FaceBox {
                x: 0.5,
                y: 0.0,
                width: 0.5,
                height: 0.5,
                score: 0.9
            }]
        );
    }

    #[test]
    fn overlapping_faces_keep_the_most_confident_and_low_scores_are_dropped() {
        let (cls, obj, bbox) = grid(
            [0.9, 0.95, 0.3, 0.8],
            [1.0, 1.0, 1.0, 1.0],
            [
                [0.9, 0.5, 0.0, 0.0],
                [0.1, 0.5, 0.0, 0.0],
                [0.5, 0.5, 0.0, 0.0],
                [0.5, 0.5, 0.0, 0.0],
            ],
        );
        let outputs = [StrideOutputs {
            stride: 32,
            cls: &cls,
            obj: &obj,
            bbox: &bbox,
        }];
        let faces = decode(&outputs, 64, 64, 64);
        let scores: Vec<f64> = faces.iter().map(|face| face.score).collect();
        // Cells 0 and 1 overlap almost entirely: only cell 1 (0.95) stays; cell 2 is too weak.
        assert_eq!(scores, [0.9747, 0.8944]);
    }

    #[test]
    fn faces_are_normalised_in_the_unpadded_picture_and_clamped() {
        let face = PixelFace {
            x: -10.0,
            y: 300.0,
            width: 100.0,
            height: 100.0,
            score: 0.9,
        };
        let normalised = normalised(face, 640, 360).unwrap();
        assert_eq!(normalised.x, 0.0);
        assert_eq!(normalised.width, 0.1406);
        assert_eq!(normalised.y, 0.8333);
        assert_eq!(normalised.height, 0.1667);
        let outside = PixelFace { x: 700.0, ..face };
        assert_eq!(super::normalised(outside, 640, 360), None);
    }
}
