//! The snapshot of an export (pure): the sequence as it is when the export starts, and the span
//! to render. Later edits never reach it (the renderer draws this snapshot, never its mirror).

use std::path::{Path, PathBuf};

use se_domain::{
    Asset, AssetId, DerivedFile, DerivedStatus, Flicks, Project, SequenceId, TimeRange,
};
use se_protocol::ProjectSnapshot;

use crate::{media_service::media_paths, project_view::project_snapshot};

/// What an export renders.
#[derive(Debug, Clone, PartialEq)]
pub struct ExportScene {
    /// That sequence, its tracks, items and markers, every asset and the project's header.
    pub snapshot: ProjectSnapshot,
    /// The sequence's name.
    pub sequence_name: String,
    /// The sequence's frame rate.
    pub frame_rate: se_domain::FrameRate,
    /// The span rendered `[start, end)`.
    pub span: TimeRange,
}

/// The snapshot of `sequence_id` in `project` and the span to render: `range`, or the sequence
/// up to the end of its last item.
///
/// # Errors
/// A message when the sequence does not exist, is empty, or `range` is shorter than one frame.
pub fn export_scene(
    project: &Project,
    sequence_id: SequenceId,
    range: Option<TimeRange>,
) -> Result<ExportScene, String> {
    let sequence = project
        .sequences
        .iter()
        .find(|sequence| sequence.id == sequence_id)
        .ok_or_else(|| format!("sequence {sequence_id} does not exist in this project"))?;
    let span = match range {
        Some(range) => range,
        None => TimeRange::from_start_end(Flicks::ZERO, sequence.content_end())
            .map_err(|error| error.to_string())?,
    };
    if span.is_empty() {
        return Err(if range.is_some() {
            "the in/out span is empty: mark an in and an out point apart".to_owned()
        } else {
            "the sequence is empty: place clips on it before exporting".to_owned()
        });
    }
    if span.duration() < sequence.format.frame_rate.frame_duration() {
        return Err("the span to export is shorter than one frame".to_owned());
    }
    let mut snapshot = project_snapshot(project);
    snapshot.sequences.retain(|entity| entity.id == sequence_id);
    snapshot
        .tracks
        .retain(|track| track.sequence_id == sequence_id);
    let tracks: Vec<_> = snapshot.tracks.iter().map(|track| track.id).collect();
    snapshot
        .items
        .retain(|item| tracks.contains(&item.track_id));
    snapshot
        .markers
        .retain(|marker| marker.sequence_id == sequence_id);
    Ok(ExportScene {
        snapshot,
        sequence_name: sequence.name.clone(),
        frame_rate: sequence.format.frame_rate,
        span,
    })
}

/// The files of an asset an export may read (its snapshot, so removing the asset from the
/// project later does not stop the export).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportMediaFiles {
    /// The original file.
    pub original: PathBuf,
    /// Its mezzanine, if it was ready when the export started.
    pub mezzanine: Option<PathBuf>,
}

/// The files of every asset of `assets`, derived copies resolved in `project_dir`.
#[must_use]
pub fn media_files_of(assets: &[Asset], project_dir: &Path) -> Vec<(AssetId, ExportMediaFiles)> {
    let ready = |file: &Option<DerivedFile>| {
        file.as_ref()
            .filter(|file| file.status == DerivedStatus::Ready)
            .and_then(|file| media_paths::resolve_relative(project_dir, &file.relative_path))
    };
    assets
        .iter()
        .map(|asset| {
            (
                asset.id,
                ExportMediaFiles {
                    original: PathBuf::from(&asset.original_path),
                    mezzanine: ready(&asset.derived.mezzanine),
                },
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use se_domain::test_support::example_project;

    use super::*;

    #[test]
    fn the_snapshot_holds_one_sequence_and_its_content() {
        let project = example_project();
        let sequence = &project.sequences[0];
        let scene = export_scene(&project, sequence.id, None).unwrap();
        assert_eq!(scene.snapshot.sequences.len(), 1);
        assert_eq!(scene.snapshot.sequences[0].id, sequence.id);
        assert_eq!(scene.sequence_name, sequence.name);
        let item_count: usize = sequence.tracks.iter().map(|track| track.items.len()).sum();
        assert_eq!(scene.snapshot.items.len(), item_count);
        assert!(
            scene
                .snapshot
                .tracks
                .iter()
                .all(|track| track.sequence_id == sequence.id)
        );
        assert_eq!(scene.snapshot.assets.len(), project.assets.len());
        assert_eq!(scene.span.start(), Flicks::ZERO);
        assert_eq!(scene.span.end(), sequence.content_end());
    }

    #[test]
    fn a_range_is_kept_and_empty_ones_are_refused() {
        let project = example_project();
        let sequence_id = project.sequences[0].id;
        let range =
            TimeRange::from_start_end(Flicks::one_second(), Flicks::from_millis(2500)).unwrap();
        assert_eq!(
            export_scene(&project, sequence_id, Some(range))
                .unwrap()
                .span,
            range
        );
        let tiny = TimeRange::new(Flicks::ZERO, Flicks::new(10).unwrap()).unwrap();
        assert!(
            export_scene(&project, sequence_id, Some(tiny))
                .unwrap_err()
                .contains("one frame")
        );
        let unknown = SequenceId::from_test_index(999);
        assert!(
            export_scene(&project, unknown, None)
                .unwrap_err()
                .contains("does not exist")
        );
    }

    #[test]
    fn empty_sequences_cannot_be_exported() {
        let mut project = example_project();
        for track in &mut project.sequences[0].tracks {
            track.items.clear();
            track.transitions.clear();
        }
        let sequence_id = project.sequences[0].id;
        assert!(
            export_scene(&project, sequence_id, None)
                .unwrap_err()
                .contains("empty")
        );
    }

    #[test]
    fn media_files_resolve_ready_mezzanines_only() {
        let project = example_project();
        let files = media_files_of(&project.assets, Path::new("/projects/A.seproj"));
        assert_eq!(files.len(), project.assets.len());
        for ((_, files), asset) in files.iter().zip(&project.assets) {
            assert_eq!(files.original, PathBuf::from(&asset.original_path));
            let ready = asset
                .derived
                .mezzanine
                .as_ref()
                .is_some_and(|file| file.status == DerivedStatus::Ready);
            assert_eq!(files.mezzanine.is_some(), ready);
        }
    }
}
