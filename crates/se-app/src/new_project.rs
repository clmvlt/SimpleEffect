//! Documents created by the app: a new project and its default sequence.

use se_domain::{
    CURRENT_SCHEMA_VERSION, Format, Project, ProjectSettings, Sequence, Track, TrackKind,
    TranscriptionSettings,
};

use crate::{AppError, new_ids::NewIds};

/// Names and kinds of the tracks of a new sequence, bottom video track first.
const DEFAULT_TRACKS: [(&str, TrackKind); 4] = [
    ("V1", TrackKind::Video),
    ("V2", TrackKind::Video),
    ("A1", TrackKind::Audio),
    ("A2", TrackKind::Audio),
];

/// A new, empty sequence named `name` with tracks V1, V2, A1 and A2.
///
/// # Errors
/// [`AppError::Internal`] if the id generator is broken.
pub fn default_sequence(ids: NewIds<'_>, name: &str, format: Format) -> Result<Sequence, AppError> {
    let tracks = DEFAULT_TRACKS
        .into_iter()
        .map(|(track_name, kind)| empty_track(ids, track_name, kind))
        .collect::<Result<_, _>>()?;
    Ok(Sequence {
        id: ids.sequence()?,
        name: name.to_owned(),
        format,
        tracks,
        markers: Vec::new(),
    })
}

/// A new project named `name` whose default format is `format`, holding one default sequence
/// named after the project, at revision 0.
///
/// # Errors
/// [`AppError::Internal`] if the id generator is broken.
pub fn new_project(ids: NewIds<'_>, name: &str, format: Format) -> Result<Project, AppError> {
    Ok(Project {
        id: ids.project()?,
        name: name.to_owned(),
        schema_version: CURRENT_SCHEMA_VERSION,
        revision: 0,
        settings: ProjectSettings {
            default_format: format,
            transcription: TranscriptionSettings::default(),
        },
        assets: Vec::new(),
        sequences: vec![default_sequence(ids, name, format)?],
        templates: Vec::new(),
        brand_kit: None,
        caption_styles: Vec::new(),
    })
}

/// A new, empty, unlocked and audible track.
///
/// # Errors
/// [`AppError::Internal`] if the id generator is broken.
pub fn empty_track(ids: NewIds<'_>, name: &str, kind: TrackKind) -> Result<Track, AppError> {
    Ok(Track {
        id: ids.track()?,
        kind,
        name: name.to_owned(),
        muted: false,
        solo: false,
        locked: false,
        hidden: false,
        volume_db: 0.0,
        pan: 0.0,
        items: Vec::new(),
        transitions: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use se_domain::{FormatPreset, FrameRate, validate_project};

    use super::*;
    use crate::test_support::SequentialIds;

    #[test]
    fn new_projects_are_valid_and_hold_a_default_sequence() {
        let ids = SequentialIds::default();
        let format = Format::from_preset(FormatPreset::Vertical9x16, FrameRate::FPS_30).unwrap();
        let project = new_project(NewIds(&ids), "Short", format).unwrap();
        assert_eq!(validate_project(&project), []);
        assert_eq!(project.revision, 0);
        assert_eq!(project.settings.default_format, format);
        let sequence = &project.sequences[0];
        assert_eq!(sequence.name, "Short");
        assert_eq!(sequence.format, format);
        let tracks: Vec<_> = sequence
            .tracks
            .iter()
            .map(|track| (track.name.as_str(), track.kind))
            .collect();
        assert_eq!(tracks, DEFAULT_TRACKS);
    }
}
