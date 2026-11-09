//! [`SubjectDetection`]: the faces found in an asset's pictures (prompt 26, ADR-0037), read by
//! automatic reframing ([`crate::tracking`]).

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{spatial::Vec2, time::Flicks};

/// Full result of a subject detection (stored in the project database): pictures sampled at a
/// regular interval over the whole asset, each with the faces it shows.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct SubjectDetection {
    /// The detector that found the faces (`yunet-2026may`): a result of another model is run
    /// again.
    pub model: String,
    /// Time between two samples.
    pub interval: Flicks,
    /// The samples, by increasing source time.
    pub frames: Vec<SubjectFrame>,
}

impl SubjectDetection {
    /// The samples whose time lies in `[start, end]`, as a slice of [`SubjectDetection::frames`].
    #[must_use]
    pub fn frames_between(&self, start: Flicks, end: Flicks) -> &[SubjectFrame] {
        let first = self.frames.partition_point(|frame| frame.time < start);
        let last = self.frames.partition_point(|frame| frame.time <= end);
        self.frames.get(first..last.max(first)).unwrap_or_default()
    }
}

/// One sampled picture and the faces it shows.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct SubjectFrame {
    /// Source time of the picture (media time 0 is the first frame).
    pub time: Flicks,
    /// The faces found, most confident first.
    pub faces: Vec<FaceBox>,
}

/// A face found in a picture: its box normalised in the upright picture (`(0, 0)` top-left,
/// `(1, 1)` bottom-right, like a reframe focus) and the detector's confidence.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct FaceBox {
    /// Left edge.
    pub x: f64,
    /// Top edge.
    pub y: f64,
    /// Width.
    pub width: f64,
    /// Height.
    pub height: f64,
    /// Confidence in `[0, 1]`.
    pub score: f64,
}

impl FaceBox {
    /// Share of a face box's height, from its top, where the eyes sit (the box runs from the
    /// forehead to the chin).
    pub const EYE_HEIGHT: f64 = 0.4;

    /// The centre of the box.
    #[must_use]
    pub fn centre(&self) -> Vec2 {
        Vec2::new(self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    /// The point between the eyes (horizontal centre, [`FaceBox::EYE_HEIGHT`] down the box).
    #[must_use]
    pub fn eyes(&self) -> Vec2 {
        Vec2::new(
            self.x + self.width / 2.0,
            self.y + self.height * Self::EYE_HEIGHT,
        )
    }

    /// The box's area, as a share of the picture's.
    #[must_use]
    pub fn area(&self) -> f64 {
        self.width.max(0.0) * self.height.max(0.0)
    }

    /// Intersection over union with `other` (0 when apart, 1 when equal).
    #[must_use]
    pub fn overlap(&self, other: &Self) -> f64 {
        let width = (self.x + self.width).min(other.x + other.width) - self.x.max(other.x);
        let height = (self.y + self.height).min(other.y + other.height) - self.y.max(other.y);
        let inter = width.max(0.0) * height.max(0.0);
        let union = self.area() + other.area() - inter;
        if union > 0.0 { inter / union } else { 0.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn face(x: f64, y: f64, size: f64) -> FaceBox {
        FaceBox {
            x,
            y,
            width: size,
            height: size,
            score: 0.9,
        }
    }

    #[test]
    fn boxes_have_a_centre_eyes_and_an_overlap() {
        let a = face(0.2, 0.2, 0.2);
        let centre = a.centre();
        assert!((centre.x - 0.3).abs() < 1e-12 && (centre.y - 0.3).abs() < 1e-12);
        assert!((a.eyes().y - 0.28).abs() < 1e-12);
        assert!((a.overlap(&a) - 1.0).abs() < 1e-12);
        assert!(a.overlap(&face(0.6, 0.6, 0.2)).abs() < 1e-12);
        let half = a.overlap(&face(0.3, 0.2, 0.2));
        assert!((half - 1.0 / 3.0).abs() < 1e-12, "{half}");
    }

    #[test]
    fn frames_between_is_inclusive() {
        let detection = SubjectDetection {
            model: "test".to_owned(),
            interval: Flicks::from_millis(200),
            frames: (0_u32..10)
                .map(|index| SubjectFrame {
                    time: Flicks::from_millis(index * 200),
                    faces: Vec::new(),
                })
                .collect(),
        };
        let times = |start, end| {
            detection
                .frames_between(Flicks::from_millis(start), Flicks::from_millis(end))
                .iter()
                .map(|frame| frame.time.get() / 705_600)
                .collect::<Vec<_>>()
        };
        assert_eq!(times(400, 800), [400, 600, 800]);
        assert_eq!(times(450, 500), Vec::<i64>::new());
        assert_eq!(times(900, 100), Vec::<i64>::new());
    }
}
