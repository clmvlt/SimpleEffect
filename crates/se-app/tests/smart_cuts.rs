//! Smart cuts over the example project (prompt 22, ADR-0032): words, fillers and silences cut
//! as one ripple transaction keeping linked picture and sound in sync, undone in one step,
//! adjustable afterwards, and agents' cuts under the « cut » scope rule.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use se_app::{
    AgentContext, AppError,
    ports::{StoredAnalysis, StoredTranscript},
    smart_cuts::{CutRequest, CutTarget, SilenceCut, SmartCutService},
    test_support::{MediaTestbed, fake_audio_analysis, levels_with_silences, probed_video, settle},
};
use se_domain::{
    AnalysisKind, AnalysisResult, AssetId, ContentFingerprint, Flicks, FrameRate, Item, ItemId,
    ItemKind, Project, ProjectId, RoundingMode, SequenceId, SilenceSettings, TimeRange, TrackId,
    Transcript, TranscriptSegment, TranscriptWord,
    test_support::{example_ids, example_project},
};
use se_protocol::{
    ErrorCode, ItemTrim, ProtocolError, SilenceCutParams, SubmitTaskRequest, TaskScope,
    WordCutPurpose,
};

const PROJECT_PATH: &str = "/films/Example.seproj";
/// V1 0–10 s (source 5–15 s), linked to `SOUND` on A1.
const CLIP: u64 = 1;
/// A1 0–10 s, the interview's sound.
const SOUND: u64 = 8;
/// V2 1–4 s.
const TITLE: u64 = 3;
/// A2 0–14 s.
const MUSIC: u64 = 9;

fn id(index: u64) -> ItemId {
    ItemId::from_test_index(index)
}

fn ms(value: i64) -> Flicks {
    Flicks::new(value * 705_600).unwrap()
}

fn span(start: i64, end: i64) -> TimeRange {
    TimeRange::from_start_end(ms(start), ms(end)).unwrap()
}

/// `time` on the example's 29.97 fps grid.
fn on_frame(time: Flicks, rounding: RoundingMode) -> Flicks {
    let rate = FrameRate::FPS_29_97;
    Flicks::from_frames(time.to_frames(rate, rounding), rate).unwrap()
}

/// The interview's words in **source** time (the clip plays source 5–15 s at 0–10 s).
fn interview_words() -> Vec<(&'static str, i64, i64, u32)> {
    vec![
        ("Bonjour", 5200, 5600, 0),
        ("à", 5650, 5700, 0),
        ("tous.", 5750, 6100, 0),
        ("Euh", 7000, 7300, 1),
        ("je", 7400, 7500, 1),
        ("commence", 7550, 8100, 1),
        ("maintenant.", 8200, 8900, 1),
    ]
}

struct Bed {
    services: MediaTestbed,
    cuts: SmartCutService,
    project_id: ProjectId,
    sequence_id: SequenceId,
    tracks: Vec<TrackId>,
    initial: Project,
}

fn content_key(project: &Project, asset: AssetId) -> String {
    project
        .assets
        .iter()
        .find(|candidate| candidate.id == asset)
        .and_then(|candidate| candidate.fingerprint.as_ref())
        .map(ContentFingerprint::cache_key)
        .unwrap()
}

fn transcript(words: &[(&str, i64, i64, u32)]) -> Transcript {
    let segments = (0..=words.iter().map(|word| word.3).max().unwrap_or(0))
        .map(|index| {
            let inside: Vec<_> = words.iter().filter(|word| word.3 == index).collect();
            TranscriptSegment {
                start: ms(inside[0].1),
                end: ms(inside[inside.len() - 1].2),
            }
        })
        .collect();
    Transcript {
        language: "fr".to_owned(),
        model: "base".to_owned(),
        words: words
            .iter()
            .map(|(text, start, end, segment)| TranscriptWord {
                text: (*text).to_owned(),
                start: ms(*start),
                end: ms(*end),
                confidence: 0.9,
                segment: *segment,
            })
            .collect(),
        segments,
    }
}

/// The example project with the interview transcribed, its sound analysed with silences at
/// source 7.6–9.0 s (sequence 2.6–4.0 s) and the music analysed without silence.
async fn bed() -> Bed {
    let services = MediaTestbed::new(Path::new("/films"));
    services.tasks.start().await.unwrap();
    let project = example_project();
    for (content, asset) in (1..).zip(&project.assets) {
        services
            .files
            .add(asset.original_path.clone(), content, probed_video());
    }
    services
        .repository
        .insert(PathBuf::from(PROJECT_PATH), project.clone(), Vec::new());
    let store = |asset: AssetId, silences: &[(i64, i64)]| {
        let mut analysis = fake_audio_analysis(SilenceSettings::SPEECH);
        analysis.levels = Some(levels_with_silences(60_000, silences));
        services.repository.set_analysis(
            PROJECT_PATH,
            StoredAnalysis {
                asset_id: asset,
                content_key: content_key(&project, asset),
                created_at: 0,
                result: AnalysisResult::AudioAnalysis(analysis),
            },
        );
    };
    store(example_ids::VIDEO_ASSET, &[(7600, 9000), (11_000, 11_200)]);
    store(example_ids::AUDIO_ASSET, &[]);
    services.repository.set_transcript(
        PROJECT_PATH,
        StoredTranscript {
            asset_id: example_ids::VIDEO_ASSET,
            content_key: content_key(&project, example_ids::VIDEO_ASSET),
            created_at: 0,
            backend: "cpu".to_owned(),
            transcript: transcript(&interview_words()),
        },
    );
    let session = services
        .projects
        .open_project(Path::new(PROJECT_PATH))
        .await
        .unwrap();
    settle().await;
    let cuts = services.smart_cuts.clone();
    let project_id = session.snapshot.project.id;
    let initial = services.projects.project(project_id).await.unwrap();
    Bed {
        cuts,
        project_id,
        sequence_id: project.sequences[0].id,
        tracks: project.sequences[0].tracks.iter().map(|t| t.id).collect(),
        initial: initial.as_ref().clone(),
        services,
    }
}

impl Bed {
    async fn project(&self) -> Project {
        self.services
            .projects
            .project(self.project_id)
            .await
            .unwrap()
            .as_ref()
            .clone()
    }

    fn silences(&self, track_ids: Option<Vec<TrackId>>) -> SilenceCut {
        SilenceCut {
            project_id: self.project_id,
            sequence_id: self.sequence_id,
            range: None,
            params: SilenceCutParams::default(),
            track_ids,
            all_tracks: false,
        }
    }

    /// A finished task of `scope` and the context of its agent.
    async fn agent(&self, range: Option<TimeRange>, tracks: Option<Vec<TrackId>>) -> AgentContext {
        let scope = TaskScope {
            sequence_id: self.sequence_id,
            range,
            tracks,
        };
        let task = self
            .services
            .tasks
            .submit(SubmitTaskRequest {
                project_id: self.project_id,
                prompt: "Cut".to_owned(),
                scope: scope.clone(),
                provider: None,
                model: None,
                effort: None,
            })
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;
        AgentContext {
            project_id: self.project_id,
            task_id: task.id,
            run: 0,
            scope,
        }
    }
}

/// Items of the track at `index` (0: V1 … 4: A2) as (id, start, end).
fn track_items(project: &Project, index: usize) -> Vec<&Item> {
    project.sequences[0].tracks[index].items.iter().collect()
}

/// `project` without revisions, to compare contents.
fn contents(mut project: Project) -> Project {
    project.revision = 0;
    for track in project.sequences.iter_mut().flat_map(|s| &mut s.tracks) {
        track.items.iter_mut().for_each(|item| item.revision = 0);
    }
    project
}

#[tokio::test(start_paused = true)]
async fn removing_a_word_cuts_picture_and_sound_together_and_undoes_in_one_step() {
    let bed = bed().await;
    // « Euh » plays at 2.0–2.3 s: 900 ms of pause before it, 100 ms after it.
    let summary = bed
        .cuts
        .remove_words(
            bed.project_id,
            bed.sequence_id,
            vec![span(2000, 2300)],
            WordCutPurpose::Words,
            false,
        )
        .await
        .unwrap();
    assert_eq!(summary.word_count, 1);
    assert_eq!(summary.cuts.len(), 1);
    let cut = summary.cuts[0];
    assert_eq!(cut.start(), on_frame(ms(1970), RoundingMode::Nearest));
    assert_eq!(cut.end(), on_frame(ms(2350), RoundingMode::Nearest));
    assert_eq!(
        summary.track_ids,
        [bed.tracks[0], bed.tracks[3]],
        "the speech's track A1 and its linked picture V1"
    );
    let project = bed.project().await;
    let (v1, a1) = (track_items(&project, 0), track_items(&project, 3));
    assert_eq!(v1.len(), 3, "the clip in two parts, then the fast clip");
    assert_eq!(a1.len(), 2);
    for (picture, sound) in v1.iter().zip(&a1) {
        assert_eq!(picture.timeline_range, sound.timeline_range);
        assert_eq!(picture.link_group, sound.link_group);
        assert!(picture.link_group.is_some());
    }
    assert_ne!(v1[0].link_group, v1[1].link_group);
    assert_eq!(v1[0].timeline_range.end(), cut.start());
    let ItemKind::MediaClip(right) = &v1[1].kind else {
        panic!("a clip")
    };
    assert_eq!(right.source_start, ms(5000).checked_add(cut.end()).unwrap());
    assert_eq!(
        track_items(&project, 4)[0].timeline_range,
        span(0, 14_000),
        "the music is not linked: it keeps playing"
    );
    assert_eq!(
        track_items(&project, 1)[0].timeline_range,
        span(1000, 4000),
        "graphics are not cut by default"
    );
    let session = bed
        .services
        .projects
        .project_session(bed.project_id)
        .await
        .unwrap();
    assert_eq!(session.history.undo_label.as_deref(), Some("Remove words"));
    bed.services.projects.undo(bed.project_id).await.unwrap();
    assert_eq!(contents(bed.project().await), contents(bed.initial.clone()));
}

#[tokio::test(start_paused = true)]
async fn every_track_is_cut_when_asked() {
    let bed = bed().await;
    bed.cuts
        .remove_words(
            bed.project_id,
            bed.sequence_id,
            vec![span(2000, 2300)],
            WordCutPurpose::Words,
            true,
        )
        .await
        .unwrap();
    let project = bed.project().await;
    assert_eq!(
        track_items(&project, 1).len(),
        3,
        "the title is split around the cut"
    );
    assert_eq!(track_items(&project, 4).len(), 2, "the music too");
    assert!(project.items().any(|item| item.id == id(TITLE)));
    assert!(project.items().any(|item| item.id == id(MUSIC)));
}

#[tokio::test(start_paused = true)]
async fn silences_are_previewed_then_cut_and_stay_adjustable() {
    let bed = bed().await;
    let music_never_silent = bed.cuts.preview_silences(bed.silences(None)).await.unwrap();
    assert!(
        music_never_silent.cuts.is_empty(),
        "every listened track must be quiet: the music on A2 never is"
    );
    let speech_only = bed.silences(Some(vec![bed.tracks[3]]));
    let preview = bed
        .cuts
        .preview_silences(speech_only.clone())
        .await
        .unwrap();
    assert_eq!(preview.cuts.len(), 1, "the 0.2 s pause is too short");
    let cut = preview.cuts[0];
    assert_eq!(cut.start(), on_frame(ms(2600 + 120), RoundingMode::Ceil));
    assert_eq!(cut.end(), on_frame(ms(4000 - 120), RoundingMode::Floor));
    assert_eq!(
        bed.project().await,
        bed.initial,
        "a preview changes nothing"
    );
    let done = bed.cuts.remove_silences(speech_only).await.unwrap();
    assert_eq!(done, preview);
    let project = bed.project().await;
    let left = track_items(&project, 0)[0].timeline_range;
    assert_eq!(left.end(), cut.start());
    // The pause left behind can be lengthened with a ripple trim of the cut's left edge.
    let longer = left.end().checked_add(ms(200)).unwrap();
    let trims = [id(CLIP), id(SOUND)].map(|item_id| ItemTrim {
        item_id,
        edge: se_domain::ItemEdge::End,
        to: longer,
    });
    bed.services
        .projects
        .trim_items(bed.project_id, &trims, true)
        .await
        .unwrap();
    let project = bed.project().await;
    assert_eq!(track_items(&project, 0)[0].timeline_range.end(), longer);
    assert_eq!(track_items(&project, 3)[0].timeline_range.end(), longer);
}

#[tokio::test(start_paused = true)]
async fn fillers_are_found_and_removed_under_their_own_label() {
    let bed = bed().await;
    let (occurrences, untranscribed) = bed
        .cuts
        .find_fillers(bed.project_id, bed.sequence_id, None)
        .await
        .unwrap();
    assert_eq!(occurrences.len(), 1);
    assert_eq!(occurrences[0].text, "Euh");
    assert_eq!(occurrences[0].start, ms(2000));
    assert_eq!(occurrences[0].item_id, id(SOUND));
    assert_eq!(
        untranscribed,
        [example_ids::AUDIO_ASSET],
        "the music is not transcribed"
    );
    let ranges = occurrences
        .iter()
        .map(|found| TimeRange::from_start_end(found.start, found.end).unwrap())
        .collect();
    bed.cuts
        .remove_words(
            bed.project_id,
            bed.sequence_id,
            ranges,
            WordCutPurpose::Fillers,
            false,
        )
        .await
        .unwrap();
    let session = bed
        .services
        .projects
        .project_session(bed.project_id)
        .await
        .unwrap();
    assert_eq!(
        session.history.undo_label.as_deref(),
        Some("Remove fillers")
    );
}

#[tokio::test(start_paused = true)]
async fn agents_cut_inside_a_time_limited_scope_and_shift_what_follows() {
    let bed = bed().await;
    let agent = bed.agent(Some(span(0, 5000)), None).await;
    let settings = bed.cuts.settings().await.unwrap();
    let request = CutRequest {
        sequence_id: bed.sequence_id,
        target: CutTarget::Fillers {
            range: None,
            lists: settings.fillers,
            max_duration: settings.filler_max_duration,
        },
        all_tracks: false,
        limit: None,
    };
    let media = bed
        .cuts
        .media(bed.project_id, bed.sequence_id)
        .await
        .unwrap();
    let outcome = bed
        .services
        .edits
        .apply_cuts(&agent, "remove_fillers", &request, &media)
        .await
        .unwrap();
    assert_eq!(outcome.summary.word_count, 1);
    assert!(outcome.revision.is_some());
    let project = bed.project().await;
    let fast = track_items(&project, 0)[2];
    assert!(
        fast.timeline_range.start() < ms(10_000),
        "the clip after the scope moved left with the cut"
    );
    let reverted = bed
        .services
        .tasks
        .revert(bed.project_id, agent.task_id)
        .await
        .unwrap();
    assert!(reverted.reverted);
    assert_eq!(contents(bed.project().await), contents(bed.initial.clone()));
}

#[tokio::test(start_paused = true)]
async fn agents_cannot_cut_linked_tracks_outside_their_scope() {
    let bed = bed().await;
    let agent = bed.agent(None, Some(vec![bed.tracks[3]])).await;
    let request = CutRequest {
        sequence_id: bed.sequence_id,
        target: CutTarget::Words {
            ranges: vec![span(2000, 2300)],
        },
        all_tracks: false,
        limit: None,
    };
    let media = bed
        .cuts
        .media(bed.project_id, bed.sequence_id)
        .await
        .unwrap();
    let error = bed
        .services
        .edits
        .apply_cuts(&agent, "remove_words", &request, &media)
        .await
        .unwrap_err();
    assert_eq!(
        ProtocolError::from(error.clone()).code,
        ErrorCode::OutOfScope
    );
    assert!(matches!(error, AppError::OutOfScope { reason } if reason.contains("track V1")));
    assert_eq!(bed.project().await, bed.initial);
}

#[tokio::test(start_paused = true)]
async fn agents_silences_stay_in_the_scope_range() {
    let bed = bed().await;
    let agent = bed.agent(Some(span(0, 3000)), None).await;
    let request = CutRequest {
        sequence_id: bed.sequence_id,
        target: CutTarget::Silences {
            range: None,
            params: SilenceCutParams::default(),
            tracks: Some(vec![bed.tracks[3]]),
        },
        all_tracks: false,
        limit: None,
    };
    let media = bed
        .cuts
        .media(bed.project_id, bed.sequence_id)
        .await
        .unwrap();
    let outcome = bed
        .services
        .edits
        .apply_cuts(&agent, "remove_silences", &request, &media)
        .await
        .unwrap();
    let cut = outcome.summary.cuts[0];
    assert_eq!(cut.start(), on_frame(ms(2720), RoundingMode::Ceil));
    assert!(
        cut.end() <= ms(3000),
        "the silence is cut up to the scope's end only"
    );
}

#[tokio::test(start_paused = true)]
async fn settings_round_trip_and_are_checked() {
    let bed = bed().await;
    let mut settings = bed.cuts.settings().await.unwrap();
    assert!(settings.fillers.iter().any(|list| list.language == "fr"));
    settings.silence.threshold_dbfs = -50.0;
    settings.fillers[0].words.push("  genre ".to_owned());
    let stored = bed.cuts.update_settings(&settings).await.unwrap();
    assert!(stored.fillers[0].words.contains(&"genre".to_owned()));
    assert_eq!(bed.cuts.settings().await.unwrap(), stored);
    settings.silence.threshold_dbfs = 0.0;
    assert!(matches!(
        bed.cuts.update_settings(&settings).await,
        Err(AppError::InvalidSetting { .. })
    ));
}

#[tokio::test(start_paused = true)]
async fn an_audio_analysis_without_levels_is_run_again_at_opening() {
    let services = MediaTestbed::new(Path::new("/films"));
    let project = example_project();
    for (content, asset) in (1..).zip(&project.assets) {
        services
            .files
            .add(asset.original_path.clone(), content, probed_video());
    }
    services
        .repository
        .insert(PathBuf::from(PROJECT_PATH), project.clone(), Vec::new());
    let mut old = fake_audio_analysis(SilenceSettings::SPEECH);
    old.levels = None;
    services.repository.set_analysis(
        PROJECT_PATH,
        StoredAnalysis {
            asset_id: example_ids::VIDEO_ASSET,
            content_key: content_key(&project, example_ids::VIDEO_ASSET),
            created_at: 0,
            result: AnalysisResult::AudioAnalysis(old),
        },
    );
    let session = services
        .projects
        .open_project(Path::new(PROJECT_PATH))
        .await
        .unwrap();
    settle().await;
    let analysis = services
        .media
        .analysis(
            session.snapshot.project.id,
            example_ids::VIDEO_ASSET,
            AnalysisKind::AudioAnalysis,
        )
        .await
        .unwrap();
    let Some(AnalysisResult::AudioAnalysis(analysis)) = analysis else {
        panic!("an audio analysis");
    };
    assert!(analysis.levels.is_some());
}
