//! Captions over the example project (prompt 23): generated from the interview's transcript on a
//! new caption track, regenerated over a range, edited (text, emphasis, merge, split, shift),
//! restyled with copy on write, exchanged as SRT, carried along by smart cuts, and made by
//! agents inside their scope — each user action one undo step.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use se_app::{
    AgentContext, AppError,
    agent_edits::{AgentEdit, ItemRef, NewAgentCaptions},
    ports::{StoredTranscript, SubtitleCue},
    test_support::{MediaTestbed, probed_video, settle},
};
use se_domain::{
    CaptionAnimation, CaptionItem, CaptionStyleId, CaptionStylePatch, ContentFingerprint,
    DesignPixels, Flicks, Item, ItemId, ItemKind, Origin, Project, ProjectId, SequenceId,
    TimeRange, Track, Transcript, TranscriptSegment, TranscriptWord,
    test_support::{example_ids, example_project},
};
use se_protocol::{
    CaptionStyleChoice, CaptionStyleEdit, CaptionTextEdit, ErrorCode, GenerateCaptionsRequest,
    ImportSubtitlesRequest, ProtocolError, SubmitTaskRequest, TaskScope, WordCutPurpose,
};

const PROJECT_PATH: &str = "/films/Example.seproj";

fn ms(value: i64) -> Flicks {
    Flicks::new(value * 705_600).unwrap()
}

fn span(start: i64, end: i64) -> TimeRange {
    TimeRange::from_start_end(ms(start), ms(end)).unwrap()
}

/// The interview in source time (the first clip plays source 5–15 s at 0–10 s).
fn transcript() -> Transcript {
    let words = [
        ("Bonjour", 5200, 5600, 0),
        ("à", 5650, 5700, 0),
        ("tous.", 5750, 6100, 0),
        ("Euh", 7000, 7300, 1),
        ("je", 7400, 7500, 1),
        ("commence", 7550, 8100, 1),
        ("maintenant.", 8200, 8900, 1),
    ];
    Transcript {
        language: "fr".to_owned(),
        model: "base".to_owned(),
        words: words
            .iter()
            .map(|&(text, start, end, segment)| TranscriptWord {
                text: text.to_owned(),
                start: ms(start),
                end: ms(end),
                confidence: 0.9,
                segment,
            })
            .collect(),
        segments: vec![
            TranscriptSegment {
                start: ms(5200),
                end: ms(6100),
            },
            TranscriptSegment {
                start: ms(7000),
                end: ms(8900),
            },
        ],
    }
}

struct Bed {
    services: MediaTestbed,
    project_id: ProjectId,
    sequence_id: SequenceId,
}

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
    let key = project
        .assets
        .iter()
        .find(|asset| asset.id == example_ids::VIDEO_ASSET)
        .and_then(|asset| asset.fingerprint.as_ref())
        .map(ContentFingerprint::cache_key)
        .unwrap();
    services.repository.set_transcript(
        PROJECT_PATH,
        StoredTranscript {
            asset_id: example_ids::VIDEO_ASSET,
            content_key: key,
            created_at: 0,
            backend: "cpu".to_owned(),
            transcript: transcript(),
        },
    );
    let session = services
        .projects
        .open_project(Path::new(PROJECT_PATH))
        .await
        .unwrap();
    settle().await;
    Bed {
        project_id: session.snapshot.project.id,
        sequence_id: project.sequences[0].id,
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

    fn request(&self, range: Option<TimeRange>, preset: Option<&str>) -> GenerateCaptionsRequest {
        GenerateCaptionsRequest {
            project_id: self.project_id,
            sequence_id: self.sequence_id,
            range,
            style: preset.map(|preset| CaptionStyleChoice::Preset {
                preset: preset.to_owned(),
            }),
            segmentation: None,
            track_name: None,
        }
    }

    async fn generate(&self, preset: &str) -> Vec<ItemId> {
        self.services
            .captions
            .generate(&self.request(None, Some(preset)))
            .await
            .unwrap()
            .item_ids
    }

    async fn agent(&self, range: Option<TimeRange>) -> AgentContext {
        let scope = TaskScope {
            sequence_id: self.sequence_id,
            range,
            tracks: None,
        };
        let task = self
            .services
            .tasks
            .submit(SubmitTaskRequest {
                project_id: self.project_id,
                prompt: "Sous-titres".to_owned(),
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

fn caption_track(project: &Project) -> &Track {
    project.sequences[0]
        .tracks
        .iter()
        .find(|track| track.name == "Sous-titres")
        .expect("a caption track")
}

fn captions(project: &Project) -> Vec<(&Item, &CaptionItem)> {
    caption_track(project)
        .items
        .iter()
        .filter_map(|item| match &item.kind {
            ItemKind::Caption(caption) => Some((item, caption)),
            _ => None,
        })
        .collect()
}

fn texts(project: &Project) -> Vec<String> {
    captions(project)
        .iter()
        .map(|(_, caption)| caption.text())
        .collect()
}

/// Sequence times (ms, rounded) of the words of every caption.
fn word_times(project: &Project) -> Vec<(String, i64, i64)> {
    captions(project)
        .iter()
        .flat_map(|(item, caption)| {
            let start = item.timeline_range.start();
            caption.words.iter().map(move |word| {
                let at = |time: Flicks| (start.get() + time.get()) / 705_600;
                (word.text.clone(), at(word.start), at(word.end))
            })
        })
        .collect()
}

fn domain_code(error: AppError) -> ErrorCode {
    ProtocolError::from(error).code
}

#[tokio::test(start_paused = true)]
async fn generation_makes_a_caption_track_whose_words_follow_the_speech_and_undoes_at_once() {
    let bed = bed().await;
    let initial = bed.project().await;
    let report = bed
        .services
        .captions
        .generate(&bed.request(None, Some("karaoke")))
        .await
        .unwrap();
    assert_eq!(report.item_ids.len(), 2);
    assert_eq!(report.replaced, 0);
    let project = bed.project().await;
    let tracks: Vec<&str> = project.sequences[0]
        .tracks
        .iter()
        .map(|track| track.name.as_str())
        .collect();
    assert_eq!(tracks, ["V1", "V2", "V3", "Sous-titres", "A1", "A2"]);
    assert_eq!(
        texts(&project),
        ["Bonjour à tous.", "Euh je commence maintenant."]
    );
    assert_eq!(
        word_times(&project),
        [
            ("Bonjour".to_owned(), 200, 600),
            ("à".to_owned(), 650, 700),
            ("tous.".to_owned(), 750, 1100),
            ("Euh".to_owned(), 2000, 2300),
            ("je".to_owned(), 2400, 2500),
            ("commence".to_owned(), 2550, 3100),
            ("maintenant.".to_owned(), 3200, 3900),
        ]
    );
    let style = project
        .caption_styles
        .iter()
        .find(|style| Some(style.id) == report.style_id)
        .unwrap();
    assert_eq!(style.preset.as_deref(), Some("karaoke"));
    assert!(
        captions(&project)
            .iter()
            .all(|(item, _)| item.origin == Origin::User)
    );
    bed.services.projects.undo(bed.project_id).await.unwrap();
    let undone = bed.project().await;
    assert_eq!(undone.sequences, initial.sequences);
    assert_eq!(undone.caption_styles, initial.caption_styles);
}

#[tokio::test(start_paused = true)]
async fn regenerating_a_range_replaces_the_captions_it_crosses_with_their_style() {
    let bed = bed().await;
    let first = bed.generate("karaoke").await;
    let report = bed
        .services
        .captions
        .generate(&bed.request(Some(span(500, 900)), None))
        .await
        .unwrap();
    assert_eq!(report.replaced, 1);
    let project = bed.project().await;
    assert_eq!(
        texts(&project),
        ["Bonjour à tous.", "Euh je commence maintenant."]
    );
    let ids: Vec<ItemId> = captions(&project).iter().map(|(item, _)| item.id).collect();
    assert_ne!(ids[0], first[0], "the crossed caption is new");
    assert_eq!(ids[1], first[1], "the other one stays");
    let styles: Vec<CaptionStyleId> = captions(&project)
        .iter()
        .map(|(_, caption)| caption.style_id)
        .collect();
    assert_eq!(styles[0], styles[1], "the replaced caption's style is kept");
}

#[tokio::test(start_paused = true)]
async fn a_corrected_typo_keeps_the_highlight_on_the_speech() {
    let bed = bed().await;
    let ids = bed.generate("karaoke").await;
    let before = word_times(&bed.project().await);
    let edit = |text: &str| CaptionTextEdit {
        project_id: bed.project_id,
        item_id: ids[0],
        text: text.to_owned(),
    };
    bed.services
        .captions
        .edit_text(&edit("Bonjour a tous."))
        .await
        .unwrap();
    let after = word_times(&bed.project().await);
    assert_eq!(after[1], ("a".to_owned(), 650, 700));
    assert_eq!(
        after
            .iter()
            .map(|word| (word.1, word.2))
            .collect::<Vec<_>>(),
        before
            .iter()
            .map(|word| (word.1, word.2))
            .collect::<Vec<_>>()
    );
    bed.services
        .captions
        .edit_text(&edit("Bonjour à vous tous."))
        .await
        .unwrap();
    let after = word_times(&bed.project().await);
    assert_eq!(after[0], ("Bonjour".to_owned(), 200, 600));
    assert_eq!(after[3], ("tous.".to_owned(), 750, 1100));
    let refused = bed
        .services
        .captions
        .edit_text(&edit("  "))
        .await
        .unwrap_err();
    assert_eq!(domain_code(refused), ErrorCode::EditRejected);
}

#[tokio::test(start_paused = true)]
async fn captions_merge_split_and_shift_in_one_step_each() {
    let bed = bed().await;
    let ids = bed.generate("minimal").await;
    let merged = bed
        .services
        .captions
        .merge(bed.project_id, &ids)
        .await
        .unwrap();
    assert_eq!(merged, ids[0]);
    let project = bed.project().await;
    assert_eq!(
        texts(&project),
        ["Bonjour à tous. Euh je commence maintenant."]
    );
    let right = bed
        .services
        .captions
        .split(bed.project_id, merged, 3)
        .await
        .unwrap();
    let project = bed.project().await;
    assert_eq!(
        texts(&project),
        ["Bonjour à tous.", "Euh je commence maintenant."]
    );
    assert_eq!(captions(&project)[1].0.id, right);
    let before = word_times(&project);
    bed.services
        .captions
        .shift(bed.project_id, &[merged, right], ms(1000))
        .await
        .unwrap();
    let shifted = word_times(&bed.project().await);
    for (old, new) in before.iter().zip(&shifted) {
        assert_eq!((new.1 - old.1, new.2 - old.2), (1000, 1000));
    }
    let refused = bed
        .services
        .captions
        .shift(bed.project_id, &[merged], ms(-60_000))
        .await
        .unwrap_err();
    assert_eq!(domain_code(refused), ErrorCode::EditRejected);
    bed.services.projects.undo(bed.project_id).await.unwrap();
    assert_eq!(word_times(&bed.project().await), before);
}

#[tokio::test(start_paused = true)]
async fn styling_a_selection_copies_a_shared_style_and_unused_styles_go() {
    let bed = bed().await;
    let ids = bed.generate("minimal").await;
    let shared = captions(&bed.project().await)[0].1.style_id;
    let patch = CaptionStylePatch {
        font_size: Some(DesignPixels(90.0)),
        ..CaptionStylePatch::default()
    };
    let styles = bed
        .services
        .captions
        .update_style(&CaptionStyleEdit {
            project_id: bed.project_id,
            item_ids: vec![ids[0]],
            patch,
        })
        .await
        .unwrap();
    let project = bed.project().await;
    assert_ne!(styles, [shared], "the selection got a copy");
    let style_of = |project: &Project, index: usize| {
        let id = captions(project)[index].1.style_id;
        project
            .caption_styles
            .iter()
            .find(|style| style.id == id)
            .cloned()
            .unwrap()
    };
    assert_eq!(style_of(&project, 0).font_size, DesignPixels(90.0));
    assert_eq!(style_of(&project, 1).id, shared);
    assert_ne!(style_of(&project, 1).font_size, DesignPixels(90.0));
    let whole_track = CaptionStylePatch {
        animation: Some(CaptionAnimation::PopPerWord),
        ..CaptionStylePatch::default()
    };
    bed.services
        .captions
        .update_style(&CaptionStyleEdit {
            project_id: bed.project_id,
            item_ids: ids.clone(),
            patch: whole_track,
        })
        .await
        .unwrap();
    let project = bed.project().await;
    assert!(
        (0..2).all(|index| style_of(&project, index).animation == CaptionAnimation::PopPerWord)
    );
    let count = project.caption_styles.len();
    let style = bed
        .services
        .captions
        .apply_preset(bed.project_id, &ids, "box")
        .await
        .unwrap();
    let project = bed.project().await;
    assert!((0..2).all(|index| style_of(&project, index).id == style));
    // The two styles the captions left are gone; the example's own caption keeps its style.
    assert_eq!(project.caption_styles.len(), count - 1);
}

#[tokio::test(start_paused = true)]
async fn subtitles_export_and_import_back_as_editable_captions() {
    let bed = bed().await;
    bed.generate("minimal").await;
    let written = bed
        .services
        .captions
        .export_subtitles(bed.project_id, bed.sequence_id, Path::new("/out/film.srt"))
        .await
        .unwrap();
    // Two generated captions and the example's own caption on V3.
    assert_eq!(written, 3);
    let cues = bed.services.subtitles.written("/out/film.srt").unwrap();
    assert_eq!(
        cues.iter().map(|cue| cue.text.as_str()).collect::<Vec<_>>(),
        [
            "Bonjour à tous.",
            "Euh je commence maintenant.",
            "Merci d'être là"
        ]
    );
    bed.services.subtitles.put(
        "/in/autre.srt",
        vec![SubtitleCue {
            start: ms(500),
            end: ms(1500),
            text: "Une seule\nligne".to_owned(),
        }],
    );
    let report = bed
        .services
        .captions
        .import_subtitles(&ImportSubtitlesRequest {
            project_id: bed.project_id,
            sequence_id: bed.sequence_id,
            path: "/in/autre.srt".to_owned(),
            style: None,
            track_name: None,
        })
        .await
        .unwrap();
    assert_eq!(report.replaced, 1);
    let project = bed.project().await;
    assert_eq!(
        texts(&project),
        ["Une seule ligne", "Euh je commence maintenant."]
    );
    let missing = bed
        .services
        .captions
        .import_subtitles(&ImportSubtitlesRequest {
            project_id: bed.project_id,
            sequence_id: bed.sequence_id,
            path: "/in/absent.srt".to_owned(),
            style: None,
            track_name: None,
        })
        .await
        .unwrap_err();
    assert_eq!(domain_code(missing), ErrorCode::SubtitleFileFailed);
}

#[tokio::test(start_paused = true)]
async fn captions_follow_a_cut_of_the_speech() {
    let bed = bed().await;
    bed.generate("minimal").await;
    let before = word_times(&bed.project().await);
    let commence = before.iter().find(|word| word.0 == "commence").unwrap().1;
    let summary = bed
        .services
        .smart_cuts
        .remove_words(
            bed.project_id,
            bed.sequence_id,
            vec![span(2000, 2300)],
            WordCutPurpose::Words,
            false,
        )
        .await
        .unwrap();
    let project = bed.project().await;
    assert!(summary.track_ids.contains(&caption_track(&project).id));
    let after = word_times(&project);
    assert!(after.iter().all(|word| word.0 != "Euh"), "{after:?}");
    let moved = after.iter().find(|word| word.0 == "commence").unwrap().1;
    let removed = summary.removed_duration.get() / 705_600;
    assert!(
        (commence - moved - removed).abs() <= 1,
        "{commence} {moved} {removed}"
    );
}

#[tokio::test(start_paused = true)]
async fn agents_make_captions_in_their_scope_and_keep_shared_styles_out_of_it() {
    let bed = bed().await;
    let agent = bed.agent(Some(span(0, 1500))).await;
    let (words, _) = bed
        .services
        .captions
        .spoken_words(bed.project_id, bed.sequence_id)
        .await
        .unwrap();
    let edit = AgentEdit::AddCaptions(Box::new(NewAgentCaptions {
        range: span(0, 1500),
        words,
        style: Some(CaptionStyleChoice::Preset {
            preset: "tiktokBold".to_owned(),
        }),
        segmentation: None,
    }));
    let report = bed
        .services
        .edits
        .apply(&agent, "create_captions", &[edit])
        .await
        .unwrap();
    assert_eq!(report.created[0].len(), 1);
    let project = bed.project().await;
    assert_eq!(texts(&project), ["Bonjour à tous."]);
    assert!(
        captions(&project)
            .iter()
            .all(|(item, _)| matches!(item.origin, Origin::AiTask { .. }))
    );
    // The user captions the rest; both share the agent's style only through an explicit choice.
    let user_ids = bed
        .services
        .captions
        .generate(&GenerateCaptionsRequest {
            style: Some(CaptionStyleChoice::Existing {
                style_id: captions(&project)[0].1.style_id,
            }),
            ..bed.request(Some(span(1900, 4000)), None)
        })
        .await
        .unwrap()
        .item_ids;
    let refused = bed
        .services
        .edits
        .apply(
            &agent,
            "update_caption_style",
            &[AgentEdit::UpdateCaptionStyle {
                // Every user of the style: changed in place, so captions outside the scope too.
                items: std::iter::once(report.created[0][0])
                    .chain(user_ids.iter().copied())
                    .map(ItemRef::Id)
                    .collect(),
                patch: CaptionStylePatch {
                    font_size: Some(DesignPixels(70.0)),
                    ..CaptionStylePatch::default()
                },
            }],
        )
        .await
        .unwrap_err();
    assert_eq!(domain_code(refused.error.clone()), ErrorCode::OutOfScope);
    assert!(
        refused.error.to_string().contains("caption style"),
        "{}",
        refused.error
    );
    bed.services
        .edits
        .apply(
            &agent,
            "update_caption_style",
            &[AgentEdit::UpdateCaptionStyle {
                items: vec![ItemRef::Id(report.created[0][0])],
                patch: CaptionStylePatch {
                    font_size: Some(DesignPixels(70.0)),
                    ..CaptionStylePatch::default()
                },
            }],
        )
        .await
        .unwrap();
}

#[tokio::test(start_paused = true)]
async fn presets_rise_above_the_bottom_third_in_vertical_sequences() {
    let bed = bed().await;
    let vertical = se_domain::Format::from_preset(
        se_domain::FormatPreset::Vertical9x16,
        se_domain::FrameRate::FPS_29_97,
    )
    .unwrap();
    bed.services
        .projects
        .set_sequence_format(
            bed.project_id,
            bed.sequence_id,
            vertical,
            se_domain::FormatAdaptation {
                clip_reframe: se_domain::ClipReframePolicy::Fill,
                fit_text_widths: true,
            },
        )
        .await
        .unwrap();
    let report = bed
        .services
        .captions
        .generate(&bed.request(None, Some("minimal")))
        .await
        .unwrap();
    let project = bed.project().await;
    let style = project
        .caption_styles
        .iter()
        .find(|style| Some(style.id) == report.style_id)
        .unwrap();
    assert!((style.position.y - se_app::captions::styles::PORTRAIT_CAPTION_BOTTOM).abs() < 1e-9);
}
