//! Shot changes read from the faces themselves: scene detection misses cuts between similar
//! pictures (two cameras on one set: a score of 0.27 for a threshold of 0.3 in the check of
//! prompt 26), yet the frame must jump with the cut instead of drifting from one shot's subject
//! to the next one's. A cut is inferred between two consecutive samples when every lasting face
//! of the first one stops there and only new lasting faces appear in the second one.

use super::subjects::{FaceTrack, link_faces};
use crate::{analysis::SubjectFrame, time::Flicks};

/// Samples a face must be seen in to count (a shorter track is a detection glitch): 0.6 s at
/// the usual 5 samples per second.
const LASTING_SAMPLES: usize = 3;

/// Cuts inferred from the faces of `frames` (consecutive samples of one clip), in source time:
/// the middle between the last sample of the faces that stop and the first one of those that
/// start, so the frame is at most half an interval early or late. A jump with a scene change
/// already between its two samples adds nothing.
pub(super) fn subject_cuts(frames: &[SubjectFrame], scene_changes: &[Flicks]) -> Vec<Flicks> {
    let lasting: Vec<FaceTrack> = link_faces(frames)
        .into_iter()
        .filter(|track| track.faces.len() >= LASTING_SAMPLES)
        .collect();
    let mut cuts = Vec::new();
    for (index, pair) in frames.windows(2).enumerate() {
        let [before, after] = pair else {
            continue;
        };
        let already_cut = scene_changes
            .iter()
            .any(|&change| change > before.time && change <= after.time);
        if !already_cut && faces_jump(&lasting, index) {
            let middle = i64::midpoint(before.time.get(), after.time.get());
            cuts.extend(Flicks::new(middle).ok());
        }
    }
    cuts
}

/// Whether the lasting faces seen in sample `before` all stop there while those seen in the next
/// sample all start there (at least one of each).
fn faces_jump(lasting: &[FaceTrack], before: usize) -> bool {
    let after = before + 1;
    let (mut stopping, mut starting) = (0_usize, 0_usize);
    for track in lasting {
        let first = track.faces.first().map(|(sample, _)| *sample);
        let last = track.faces.last().map(|(sample, _)| *sample);
        let (Some(first), Some(last)) = (first, last) else {
            continue;
        };
        let seen_before = track.faces.iter().any(|(sample, _)| *sample == before);
        let seen_after = track.faces.iter().any(|(sample, _)| *sample == after);
        if (seen_before && last != before) || (seen_after && first != after) {
            return false;
        }
        stopping += usize::from(seen_before);
        starting += usize::from(seen_after);
    }
    stopping > 0 && starting > 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::FaceBox;

    const NO_CUTS: [Flicks; 0] = [];

    fn face(x: f64) -> FaceBox {
        FaceBox {
            x,
            y: 0.3,
            width: 0.1,
            height: 0.13,
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

    #[test]
    fn a_subject_replaced_by_another_far_away_is_a_cut_between_the_samples() {
        // One face drifting right for 6 samples, then another face far on the right.
        let samples = frames((0..12).map(|index| {
            if index < 6 {
                vec![face(0.2 + 0.01 * f64::from(index))]
            } else {
                vec![face(0.8)]
            }
        }));
        assert_eq!(subject_cuts(&samples, &[]), [Flicks::from_millis(1100)]);
    }

    #[test]
    fn a_scene_change_between_the_samples_is_kept_alone() {
        let samples = frames((0..12).map(|index| vec![face(if index < 6 { 0.2 } else { 0.8 })]));
        assert_eq!(
            subject_cuts(&samples, &[Flicks::from_millis(1080)]),
            NO_CUTS
        );
        // A scene change elsewhere does not hide the jump.
        assert_eq!(
            subject_cuts(&samples, &[Flicks::from_millis(400)]),
            [Flicks::from_millis(1100)]
        );
    }

    #[test]
    fn glitches_and_lost_faces_are_not_cuts() {
        // A false face for one sample while the subject is missed, then the subject again.
        let glitch = frames((0..12).map(|index| vec![face(if index == 5 { 0.8 } else { 0.2 })]));
        assert_eq!(subject_cuts(&glitch, &[]), NO_CUTS);
        // The subject turns away for a second: nothing starts.
        let lost = frames((0..12).map(|index| {
            if (4..8).contains(&index) {
                Vec::new()
            } else {
                vec![face(0.2)]
            }
        }));
        assert_eq!(subject_cuts(&lost, &[]), NO_CUTS);
        // Somebody joins while the subject stays: nothing stops.
        let joined = frames((0..12).map(|index| {
            let mut faces = vec![face(0.2)];
            if index >= 6 {
                faces.push(face(0.7));
            }
            faces
        }));
        assert_eq!(subject_cuts(&joined, &[]), NO_CUTS);
    }

    #[test]
    fn a_two_shot_cut_to_a_single_shot_is_a_cut() {
        let samples = frames((0..12).map(|index| {
            if index < 6 {
                vec![face(0.3), face(0.6)]
            } else {
                vec![face(0.85)]
            }
        }));
        assert_eq!(subject_cuts(&samples, &[]), [Flicks::from_millis(1100)]);
    }
}
