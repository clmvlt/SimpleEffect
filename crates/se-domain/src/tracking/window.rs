//! [`CropWindow`]: the part of the picture a filling clip keeps, and the focus that frames a
//! subject in it. Mirrors the engine's `fillCrop` (`packages/engine/src/evaluate/reframe.ts`):
//! the frame's aspect, as large as the picture allows, around a focus kept at the frame's centre.

use crate::spatial::Vec2;

/// Size of the window a filling clip shows, normalised in the upright picture (`1` = the whole
/// width or height).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CropWindow {
    /// Width of the window, in `(0, 1]`.
    pub width: f64,
    /// Height of the window, in `(0, 1]`.
    pub height: f64,
}

impl CropWindow {
    /// The window of a picture of aspect `picture_aspect` (width / height) filling a frame of
    /// aspect `frame_aspect`; the whole picture when either aspect is unusable.
    #[must_use]
    pub fn new(picture_aspect: f64, frame_aspect: f64) -> Self {
        let usable = |aspect: f64| aspect.is_finite() && aspect > 0.0;
        if !usable(picture_aspect) || !usable(frame_aspect) {
            return Self {
                width: 1.0,
                height: 1.0,
            };
        }
        Self {
            width: (frame_aspect / picture_aspect).min(1.0),
            height: (picture_aspect / frame_aspect).min(1.0),
        }
    }

    /// The focus nearest to `focus` whose window stays inside the picture. The engine clamps the
    /// window anyway; clamping the focus keeps every keyframe meaningful (moving it moves the
    /// picture) and avoids smoothing towards unreachable places.
    #[must_use]
    pub fn clamp(&self, focus: Vec2) -> Vec2 {
        Vec2::new(
            clamp_axis(focus.x, self.width),
            clamp_axis(focus.y, self.height),
        )
    }

    /// The focus that puts `eyes` (a point of the picture) on the frame's vertical centre line,
    /// `eye_line` (a share of the frame's height) below its top — as far as the picture allows.
    #[must_use]
    pub fn focus_for(&self, eyes: Vec2, eye_line: f64) -> Vec2 {
        self.clamp(Vec2::new(eyes.x, eyes.y + (0.5 - eye_line) * self.height))
    }

    /// The largest difference between two focuses, in shares of the window's width or height
    /// (how far the visible picture moves, relative to the frame).
    #[must_use]
    pub fn distance(&self, a: Vec2, b: Vec2) -> f64 {
        ((a.x - b.x) / self.width)
            .abs()
            .max(((a.y - b.y) / self.height).abs())
    }
}

/// `value` kept within `[extent / 2, 1 − extent / 2]` (the centre when the window spans the
/// whole axis).
fn clamp_axis(value: f64, extent: f64) -> f64 {
    let half = extent / 2.0;
    if half >= 0.5 || !value.is_finite() {
        0.5
    } else {
        value.clamp(half, 1.0 - half)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LANDSCAPE: f64 = 16.0 / 9.0;
    const PORTRAIT: f64 = 9.0 / 16.0;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn a_landscape_picture_in_a_portrait_frame_keeps_its_height() {
        let window = CropWindow::new(LANDSCAPE, PORTRAIT);
        assert!(close(window.width, 81.0 / 256.0), "{window:?}");
        assert!(close(window.height, 1.0));
        let square = CropWindow::new(LANDSCAPE, 1.0);
        assert!(close(square.width, 9.0 / 16.0));
        let portrait = CropWindow::new(PORTRAIT, LANDSCAPE);
        assert!(close(portrait.width, 1.0));
        assert!(close(portrait.height, 81.0 / 256.0));
        assert_eq!(
            CropWindow::new(f64::NAN, 1.0),
            CropWindow {
                width: 1.0,
                height: 1.0
            }
        );
    }

    #[test]
    fn focuses_stay_where_the_window_fits() {
        let window = CropWindow::new(LANDSCAPE, PORTRAIT);
        let half = window.width / 2.0;
        assert_eq!(window.clamp(Vec2::new(0.0, 0.0)), Vec2::new(half, 0.5));
        assert_eq!(
            window.clamp(Vec2::new(1.2, 0.9)),
            Vec2::new(1.0 - half, 0.5)
        );
        assert_eq!(window.clamp(Vec2::new(0.4, 0.2)), Vec2::new(0.4, 0.5));
    }

    #[test]
    fn the_eye_line_matters_when_the_window_is_shorter_than_the_picture() {
        let window = CropWindow::new(PORTRAIT, LANDSCAPE);
        // Eyes at 0.4 of the picture, wanted on the upper third of the frame.
        let focus = window.focus_for(Vec2::new(0.5, 0.4), 1.0 / 3.0);
        let top = focus.y - window.height / 2.0;
        assert!(close(top + window.height / 3.0, 0.4), "{focus:?}");
        // Eyes near the top: the window cannot go above the picture.
        let high = window.focus_for(Vec2::new(0.5, 0.05), 1.0 / 3.0);
        assert!(close(high.y, window.height / 2.0));
        // A full-height window ignores the eye line.
        let full = CropWindow::new(LANDSCAPE, PORTRAIT).focus_for(Vec2::new(0.3, 0.1), 0.2);
        assert_eq!(full, Vec2::new(0.3, 0.5));
    }

    #[test]
    fn distances_are_relative_to_the_window() {
        let window = CropWindow::new(LANDSCAPE, PORTRAIT);
        let moved = window.distance(
            Vec2::new(0.5, 0.5),
            Vec2::new(0.5 + window.width / 10.0, 0.5),
        );
        assert!(close(moved, 0.1));
    }
}
