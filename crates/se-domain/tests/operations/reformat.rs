//! Property tests of sequences in other formats (prompt 16, ADR-0026): after random edits, a
//! format change keeps the project valid and undoes exactly, and a version of any sequence is
//! valid next to its original and shares none of its ids.

use std::collections::BTreeSet;

use proptest::prelude::*;
use se_domain::{
    AddSequence, ClipReframePolicy, DomainError, Format, FormatAdaptation, FormatPreset, FrameRate,
    Project, Sequence, apply, apply_all, format_change, reformat::copy::CountingIds,
    sequence_version, test_support::example_project, validate_project,
};

use crate::{
    CASES,
    generators::{Generator, Seed, seed},
};

/// The example project after the operations of `seeds` that succeed.
fn edited(seeds: Vec<Seed>) -> Project {
    let mut generator = Generator::new();
    let mut project = example_project();
    for seed in seeds {
        let Some(operation) = generator.operation(&project, seed) else {
            continue;
        };
        if let Ok(applied) = apply(&project, &operation) {
            project = applied.project;
        }
    }
    project
}

fn adaptation(choice: u32) -> FormatAdaptation {
    let clip_reframe = match choice % 3 {
        0 => ClipReframePolicy::Keep,
        1 => ClipReframePolicy::Fit,
        _ => ClipReframePolicy::Fill,
    };
    FormatAdaptation {
        clip_reframe,
        fit_text_widths: choice.is_multiple_of(2),
    }
}

/// A named format at the frame rate of `sequence` (another rate now and then).
fn format_for(sequence: &Sequence, choice: u32) -> Format {
    let preset =
        FormatPreset::NAMED[usize::try_from(choice).unwrap_or(0) % FormatPreset::NAMED.len()];
    let rate = if choice.is_multiple_of(13) {
        FrameRate::FPS_60
    } else {
        sequence.format.frame_rate
    };
    Format::from_preset(preset, rate).unwrap()
}

/// The sequence of `project` picked by `pick`, if there is one.
fn picked(project: &Project, pick: u32) -> Option<&Sequence> {
    let count = project.sequences.len();
    (count > 0).then(|| &project.sequences[usize::try_from(pick).unwrap_or(0) % count])
}

/// Every id of a sequence (sequence, tracks, items, transitions, markers, link groups).
fn ids_of(sequence: &Sequence) -> BTreeSet<String> {
    let mut ids = BTreeSet::from([sequence.id.to_string()]);
    ids.extend(sequence.markers.iter().map(|marker| marker.id.to_string()));
    for track in &sequence.tracks {
        ids.insert(track.id.to_string());
        ids.extend(
            track
                .transitions
                .iter()
                .map(|transition| transition.id.to_string()),
        );
        for item in &track.items {
            ids.insert(item.id.to_string());
            ids.extend(item.link_group.map(|group| group.to_string()));
        }
    }
    ids
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES / 2))]

    /// A format change (any policy) leaves a valid project in the new format, and its inverse
    /// operations give back exactly the project before it.
    #[test]
    fn format_changes_keep_projects_valid_and_undo_exactly(
        seeds in prop::collection::vec(seed(), 0..12),
        pick in any::<u32>(),
        choice in any::<u32>(),
    ) {
        let project = edited(seeds);
        let Some(sequence) = picked(&project, pick) else { return Ok(()) };
        let format = format_for(sequence, choice);
        let operations = match format_change(&project, sequence.id, format, adaptation(choice / 3)) {
            Ok(operations) => operations,
            Err(DomainError::FrameRateLocked { .. }) => return Ok(()),
            Err(error) => panic!("unexpected refusal: {error}"),
        };
        let applied = apply_all(&project, &operations)
            .unwrap_or_else(|error| panic!("format change refused: {error}"));
        prop_assert_eq!(validate_project(&applied.project), []);
        let changed = applied.project.sequences.iter().find(|candidate| candidate.id == sequence.id);
        prop_assert_eq!(changed.map(|changed| changed.format), Some(format));
        let restored = apply_all(&applied.project, &applied.inverse).unwrap().project;
        prop_assert_eq!(restored, project);
    }

    /// A version of any sequence (any format and policy) is a valid sequence next to its
    /// original, shares none of its ids and leaves the original untouched.
    #[test]
    fn versions_are_valid_and_share_no_id(
        seeds in prop::collection::vec(seed(), 0..12),
        pick in any::<u32>(),
        choice in any::<u32>(),
    ) {
        let project = edited(seeds);
        let Some(source) = picked(&project, pick) else { return Ok(()) };
        let format = format_for(source, choice);
        let version = match sequence_version(
            source,
            format!("{} 9:16", source.name),
            format,
            adaptation(choice / 3),
            &mut CountingIds::after(100_000),
        ) {
            Ok(version) => version,
            Err(DomainError::FrameRateLocked { .. }) => return Ok(()),
            Err(error) => panic!("unexpected refusal: {error}"),
        };
        prop_assert!(ids_of(&version).is_disjoint(&ids_of(source)));
        let added = apply(&project, &AddSequence { index: None, sequence: version }.into())
            .unwrap_or_else(|error| panic!("version refused: {error}"))
            .project;
        prop_assert_eq!(validate_project(&added), []);
        prop_assert_eq!(&added.sequences[..project.sequences.len()], &project.sequences[..]);
    }
}
