//! The subject of a shot: faces linked across samples into tracks, the main one chosen by
//! persistence, size and centrality, and the point to frame in each sample — the main subject's
//! eyes, or the middle of the group when the shot's significant faces fit in the window together
//! (group framing, decided once per shot so the frame never hesitates between the two).

use super::{as_float, window::CropWindow};
use crate::{
    analysis::{FaceBox, SubjectFrame},
    spatial::Vec2,
};

/// Samples a face may go unseen (turned away, hidden) and still continue its track: 1 s at the
/// usual 5 samples per second.
const MAX_GAP: usize = 5;

/// How far, in face widths, a face may move between two of its samples and stay the same face.
const LINK_DISTANCE: f64 = 1.5;

/// A track is significant (part of a group) when it is seen in this share of the shot's samples…
const GROUP_PRESENCE: f64 = 0.25;

/// …and its faces are at least this share of the main subject's size.
const GROUP_SIZE: f64 = 0.5;

/// Faces of a group fit when they span at most this share of the window (room on both sides).
const GROUP_ROOM: f64 = 0.85;

/// A shot is framed as a group when its faces fit together in this share of the samples where
/// several of them are seen.
const GROUP_AGREEMENT: f64 = 0.8;

/// One face followed across the samples of a shot.
#[derive(Debug, Clone)]
struct FaceTrack {
    /// `(sample index, face)` in time order, at most one per sample.
    faces: Vec<(usize, FaceBox)>,
}

impl FaceTrack {
    fn last(&self) -> Option<&(usize, FaceBox)> {
        self.faces.last()
    }

    /// Persistence × size × centrality × confidence, summed over the track's faces.
    fn weight(&self) -> f64 {
        self.faces.iter().map(|(_, face)| face_weight(face)).sum()
    }

    /// Mean of the square roots of its faces' areas (a linear size).
    fn mean_size(&self) -> f64 {
        let count = as_float(self.faces.len().max(1));
        self.faces
            .iter()
            .map(|(_, face)| face.area().sqrt())
            .sum::<f64>()
            / count
    }

    fn face_at(&self, sample: usize) -> Option<&FaceBox> {
        self.faces
            .binary_search_by_key(&sample, |(index, _)| *index)
            .ok()
            .and_then(|position| self.faces.get(position))
            .map(|(_, face)| face)
    }
}

fn face_weight(face: &FaceBox) -> f64 {
    let centre = face.centre();
    let offset = (centre.x - 0.5).hypot(centre.y - 0.5) / std::f64::consts::FRAC_1_SQRT_2;
    let centrality = 1.0 - 0.5 * offset.min(1.0);
    face.area().sqrt() * centrality * face.score.clamp(0.0, 1.0)
}

/// How likely `next` continues the face `previous` (0: not at all).
fn affinity(previous: &FaceBox, next: &FaceBox) -> f64 {
    let overlap = previous.overlap(next);
    let a = previous.centre();
    let b = next.centre();
    let size = previous.width.max(next.width).max(f64::EPSILON);
    let distance = (a.x - b.x).hypot(a.y - b.y);
    let nearness = (1.0 - distance / size / LINK_DISTANCE).max(0.0);
    overlap.max(nearness * 0.5)
}

/// Links the faces of `frames` into tracks, greedily in time order (most confident faces first).
fn link_faces(frames: &[SubjectFrame]) -> Vec<FaceTrack> {
    let mut tracks: Vec<FaceTrack> = Vec::new();
    for (sample, frame) in frames.iter().enumerate() {
        let mut taken = vec![false; tracks.len()];
        for face in &frame.faces {
            let best = tracks
                .iter()
                .enumerate()
                .filter_map(|(index, track)| {
                    let &(seen, ref last) = track.last()?;
                    let open = !taken.get(index).copied().unwrap_or(true)
                        && seen < sample
                        && sample - seen <= MAX_GAP;
                    let score = affinity(last, face);
                    (open && score > 0.0).then_some((index, score))
                })
                .max_by(|a, b| a.1.total_cmp(&b.1));
            match best {
                Some((index, _)) => {
                    if let (Some(track), Some(flag)) = (tracks.get_mut(index), taken.get_mut(index))
                    {
                        track.faces.push((sample, *face));
                        *flag = true;
                    }
                }
                None => tracks.push(FaceTrack {
                    faces: vec![(sample, *face)],
                }),
            }
        }
    }
    tracks
}

/// The point to frame in each sample of a shot (`None` where no face of interest is seen).
pub(super) fn shot_targets(
    frames: &[SubjectFrame],
    window: CropWindow,
    group_framing: bool,
) -> Vec<Option<Vec2>> {
    let tracks = link_faces(frames);
    let Some(main) = tracks
        .iter()
        .max_by(|a, b| a.weight().total_cmp(&b.weight()))
    else {
        return vec![None; frames.len()];
    };
    let group: Vec<&FaceTrack> = if group_framing {
        significant_tracks(&tracks, main, frames.len())
    } else {
        Vec::new()
    };
    let framed_together = group.len() > 1 && fits_together(&group, frames.len(), window);
    (0..frames.len())
        .map(|sample| {
            if framed_together {
                group_point(&group, sample)
            } else {
                main.face_at(sample).map(FaceBox::eyes)
            }
        })
        .collect()
}

/// The tracks seen often enough and large enough (relative to the main subject) to be framed.
fn significant_tracks<'a>(
    tracks: &'a [FaceTrack],
    main: &FaceTrack,
    samples: usize,
) -> Vec<&'a FaceTrack> {
    let main_size = main.mean_size();
    tracks
        .iter()
        .filter(|track| {
            as_float(track.faces.len()) >= GROUP_PRESENCE * as_float(samples)
                && track.mean_size() >= GROUP_SIZE * main_size
        })
        .collect()
}

/// Whether the group's faces fit in the window in most of the samples where several are seen.
fn fits_together(group: &[&FaceTrack], samples: usize, window: CropWindow) -> bool {
    let mut together = 0_usize;
    let mut fitting = 0_usize;
    for sample in 0..samples {
        let faces: Vec<&FaceBox> = group
            .iter()
            .filter_map(|track| track.face_at(sample))
            .collect();
        if faces.len() < 2 {
            continue;
        }
        together += 1;
        let (left, right, top, bottom) = extent(&faces);
        if right - left <= GROUP_ROOM * window.width && bottom - top <= GROUP_ROOM * window.height {
            fitting += 1;
        }
    }
    together > 0 && as_float(fitting) >= GROUP_AGREEMENT * as_float(together)
}

/// The middle of the group's faces seen in `sample` (horizontally), at the mean height of their
/// eyes.
fn group_point(group: &[&FaceTrack], sample: usize) -> Option<Vec2> {
    let faces: Vec<&FaceBox> = group
        .iter()
        .filter_map(|track| track.face_at(sample))
        .collect();
    if faces.is_empty() {
        return None;
    }
    let (left, right, _, _) = extent(&faces);
    let eyes = faces.iter().map(|face| face.eyes().y).sum::<f64>() / as_float(faces.len());
    Some(Vec2::new(f64::midpoint(left, right), eyes))
}

/// Left, right, top and bottom edges of `faces` together.
fn extent(faces: &[&FaceBox]) -> (f64, f64, f64, f64) {
    faces.iter().fold(
        (
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
        ),
        |(left, right, top, bottom), face| {
            (
                left.min(face.x),
                right.max(face.x + face.width),
                top.min(face.y),
                bottom.max(face.y + face.height),
            )
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time::Flicks;

    fn face(x: f64, y: f64, size: f64) -> FaceBox {
        FaceBox {
            x,
            y,
            width: size,
            height: size * 1.3,
            score: 0.9,
        }
    }

    fn frames(faces: impl IntoIterator<Item = Vec<FaceBox>>) -> Vec<SubjectFrame> {
        faces
            .into_iter()
            .zip(0_u32..)
            .map(|(faces, index)| SubjectFrame {
                time: Flicks::from_millis(index * 200),
                faces,
            })
            .collect()
    }

    const PORTRAIT_IN_LANDSCAPE: CropWindow = CropWindow {
        width: 81.0 / 256.0,
        height: 1.0,
    };

    #[test]
    fn the_largest_steadiest_face_is_the_subject() {
        // A big face on the left all along; a small one on the right in a few samples.
        let samples = frames((0..10).map(|index| {
            let mut faces = vec![face(0.2, 0.3, 0.15)];
            if index % 3 == 0 {
                faces.push(face(0.8, 0.3, 0.05));
            }
            faces
        }));
        let targets = shot_targets(&samples, PORTRAIT_IN_LANDSCAPE, false);
        assert!(
            targets
                .iter()
                .all(|target| { target.is_some_and(|point| (point.x - 0.275).abs() < 1e-9) })
        );
    }

    #[test]
    fn faces_keep_their_identity_when_they_cross_or_disappear() {
        // The subject drifts right; it is missing in two samples.
        let samples = frames((0..12).map(|index| {
            if index == 4 || index == 5 {
                return Vec::new();
            }
            vec![face(0.2 + 0.02 * f64::from(index), 0.3, 0.12)]
        }));
        let tracks = link_faces(&samples);
        assert_eq!(tracks.len(), 1);
        let targets = shot_targets(&samples, PORTRAIT_IN_LANDSCAPE, true);
        assert_eq!(targets[4], None);
        assert!(targets[6].is_some());
    }

    #[test]
    fn a_two_shot_that_fits_is_framed_as_a_group() {
        let two = frames((0..10).map(|_| vec![face(0.40, 0.3, 0.06), face(0.54, 0.32, 0.06)]));
        let targets = shot_targets(&two, PORTRAIT_IN_LANDSCAPE, true);
        let point = targets[0].unwrap();
        assert!((point.x - 0.5).abs() < 1e-9, "{point:?}");
        // Without group framing the main (more central) face is followed.
        let single = shot_targets(&two, PORTRAIT_IN_LANDSCAPE, false)[0].unwrap();
        assert!((single.x - 0.43).abs() < 1e-9 || (single.x - 0.57).abs() < 1e-9);
    }

    #[test]
    fn a_two_shot_too_wide_follows_the_main_subject() {
        let wide = frames((0..10).map(|_| vec![face(0.15, 0.3, 0.12), face(0.75, 0.3, 0.08)]));
        let point = shot_targets(&wide, PORTRAIT_IN_LANDSCAPE, true)[0].unwrap();
        assert!((point.x - 0.21).abs() < 1e-9, "{point:?}");
    }

    #[test]
    fn a_shot_without_faces_has_no_target() {
        let empty = frames((0..4).map(|_| Vec::new()));
        assert_eq!(
            shot_targets(&empty, PORTRAIT_IN_LANDSCAPE, true),
            vec![None; 4]
        );
    }
}
