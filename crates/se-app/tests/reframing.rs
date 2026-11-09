//! Automatic reframing over the example project (prompt 26): clips follow their subject once
//! their media's faces are analysed (in the background, then at once when analysed), versions
//! made with reframing, refusals when detection is unavailable, failed analyses reported, and
//! agents reframing their scope or adding a reframed version that reverts with their task.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use se_app::{
    AgentContext, AppError,
    agent_edits::{AgentEdit, ItemRef, ReframeClips, ReframedVersion},
    reframing::{SAMPLE_INTERVAL, video_clips},
    test_support::{MediaTestbed, probed_video, settle},
};
use se_domain::{
    FaceBox, Flicks, Format, FormatAdaptation, FormatPreset, FrameRate, ItemId, ItemKind, Project,
    ProjectId, Reframe, SequenceId, TrackingSettings,
    test_support::{example_ids, example_project},
};
use se_protocol::{CoreEvent, ErrorCode, ProtocolError, SubmitTaskRequest, TaskScope};

const PROJECT_PATH: &str = "/films/Example.seproj";

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
    // The interview lasts 60 s: sample all of it (its second clip plays 20–28 s); its proxy is
    // ready, so that is what gets sampled.
    services.sampler.set_default_duration(60);
    services
        .repository
        .insert(PathBuf::from(PROJECT_PATH), project.clone(), Vec::new());
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

    async fn clips(&self) -> Vec<ItemId> {
        video_clips(&self.project().await.sequences[0])
    }

    /// Lets the queued analyses run and the background waiter poll.
    async fn let_analyses_run(&self) {
        for _ in 0..10 {
            settle().await;
            tokio::time::sleep(Duration::from_millis(400)).await;
        }
    }

    fn finished_events(&self) -> Vec<(Vec<ItemId>, usize)> {
        self.services
            .events
            .take()
            .into_iter()
            .filter_map(|event| match event {
                CoreEvent::AutoReframeFinished {
                    applied_item_ids,
                    failures,
                    ..
                } => Some((applied_item_ids, failures.len())),
                _ => None,
            })
            .collect()
    }

    async fn agent(&self, range: Option<se_domain::TimeRange>) -> AgentContext {
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
                prompt: "Recadre".to_owned(),
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
            run: 1,
            scope,
        }
    }
}

fn reframe_of(project: &Project, item: ItemId) -> Reframe {
    match &project
        .items()
        .find(|candidate| candidate.id == item)
        .unwrap()
        .kind
    {
        ItemKind::MediaClip(clip) => clip.reframe.clone(),
        _ => panic!("a media clip"),
    }
}

/// 9:16 at the example sequence's 29.97 fps (its frame rate is locked once it holds items).
fn vertical() -> Format {
    Format::from_preset(FormatPreset::Vertical9x16, FrameRate::FPS_29_97).unwrap()
}

#[tokio::test(start_paused = true)]
async fn clips_follow_their_subject_once_their_faces_are_analysed() {
    let bed = bed().await;
    let clips = bed.clips().await;
    let before = bed.project().await;
    let settings = TrackingSettings {
        smoothing: 0.7,
        ..TrackingSettings::default()
    };
    bed.services.sampler.hold();
    let report = bed
        .services
        .reframing
        .reframe_items(bed.project_id, &clips, settings)
        .await
        .unwrap();
    assert_eq!(report.applied_item_ids, Vec::<ItemId>::new());
    assert_eq!(report.waiting_asset_ids, [example_ids::VIDEO_ASSET]);
    assert_eq!(report.skipped, []);

    bed.services.sampler.release();
    bed.let_analyses_run().await;
    let sampled = bed.services.sampler.requests();
    assert_eq!(sampled.len(), 1, "one analysis per content");
    assert_eq!(sampled[0].interval, SAMPLE_INTERVAL);
    assert!(bed.services.detector.detected() > 0);
    let finished = bed.finished_events();
    assert_eq!(finished.len(), 1);
    assert_eq!(finished[0].1, 0, "no failure");
    let mut applied = finished[0].0.clone();
    applied.sort_unstable();
    let mut expected = clips.clone();
    expected.sort_unstable();
    assert_eq!(applied, expected);
    let project = bed.project().await;
    for &clip in &clips {
        let reframe = reframe_of(&project, clip);
        assert_eq!(reframe.tracking(), Some(&settings), "{reframe:?}");
    }

    // One undo step brings the previous framing back.
    bed.services.projects.undo(bed.project_id).await.unwrap();
    let undone = bed.project().await;
    for &clip in &clips {
        assert_eq!(reframe_of(&undone, clip), reframe_of(&before, clip));
    }

    // Analysed now: reframed at once, without a new analysis.
    let report = bed
        .services
        .reframing
        .reframe_items(bed.project_id, &clips, TrackingSettings::default())
        .await
        .unwrap();
    assert_eq!(report.applied_item_ids.len(), clips.len());
    assert_eq!(report.waiting_asset_ids, []);
    assert_eq!(bed.services.sampler.requests().len(), 1);
}

#[tokio::test(start_paused = true)]
async fn texts_and_sounds_are_skipped_with_their_reason() {
    let bed = bed().await;
    let everything: Vec<ItemId> = bed.project().await.items().map(|item| item.id).collect();
    let report = bed
        .services
        .reframing
        .reframe_items(bed.project_id, &everything, TrackingSettings::default())
        .await
        .unwrap();
    assert_ne!(report.skipped, []);
    assert_eq!(
        report.skipped.len() + bed.clips().await.len(),
        everything.len()
    );
    let unknown = bed
        .services
        .reframing
        .reframe_items(
            bed.project_id,
            &[ItemId::from_test_index(999)],
            TrackingSettings::default(),
        )
        .await;
    assert!(unknown.is_err());
    let broken = TrackingSettings {
        smoothing: 2.0,
        ..TrackingSettings::default()
    };
    let refused = bed
        .services
        .reframing
        .reframe_items(bed.project_id, &everything, broken)
        .await
        .unwrap_err();
    assert!(refused.to_string().contains("smoothing"), "{refused}");
}

#[tokio::test(start_paused = true)]
async fn a_version_with_automatic_reframing_follows_the_subject() {
    let bed = bed().await;
    bed.services.sampler.hold();
    let (copy, report) = bed
        .services
        .reframing
        .create_version(
            bed.project_id,
            bed.sequence_id,
            "Vertical",
            Some(vertical()),
            FormatAdaptation::default(),
            TrackingSettings::default(),
        )
        .await
        .unwrap();
    assert_eq!(report.waiting_asset_ids, [example_ids::VIDEO_ASSET]);
    bed.services.sampler.release();
    bed.let_analyses_run().await;
    let project = bed.project().await;
    let version = project
        .sequences
        .iter()
        .find(|sequence| sequence.id == copy)
        .unwrap();
    assert_eq!(version.format, vertical());
    let clips = video_clips(version);
    assert_ne!(clips, []);
    let sampled = bed.services.sampler.requests();
    assert!(sampled[0].source.ends_with("proxy-540.mp4"), "{sampled:?}");
    for clip in clips {
        let Reframe::Tracked { focus, .. } = reframe_of(&project, clip) else {
            panic!("a tracked clip");
        };
        // The fake face sits left of centre: the 9:16 window follows it there.
        let value = match focus {
            se_domain::Animatable::Static { value } => value,
            se_domain::Animatable::Keyframed { keyframes } => keyframes[0].value,
        };
        assert!(value.x < 0.45, "{value:?}");
    }
    // The source sequence is untouched.
    let source = &project.sequences[0];
    assert!(
        video_clips(source)
            .iter()
            .all(|&clip| reframe_of(&project, clip).tracking().is_none())
    );
}

#[tokio::test(start_paused = true)]
async fn unavailable_detection_is_refused_with_its_reason() {
    let bed = bed().await;
    bed.services
        .detector
        .set_unavailable(Some("the ONNX Runtime library was not found"));
    let error = bed
        .services
        .reframing
        .reframe_items(
            bed.project_id,
            &bed.clips().await,
            TrackingSettings::default(),
        )
        .await
        .unwrap_err();
    assert!(matches!(error, AppError::VisionUnavailable { .. }));
    assert_eq!(
        ProtocolError::from(error).code,
        ErrorCode::VisionUnavailable
    );
    let status = bed.services.reframing.status().await;
    assert!(!status.available);
    assert_eq!(
        status.reason.as_deref(),
        Some("the ONNX Runtime library was not found")
    );
}

#[tokio::test(start_paused = true)]
async fn a_failed_analysis_is_reported_and_leaves_the_clips() {
    let bed = bed().await;
    let before = bed.project().await;
    // Both attempts fail (one automatic retry).
    bed.services
        .sampler
        .set_failure(Some("ffmpeg cannot read clip.mov"));
    let clips = bed.clips().await;
    bed.services
        .reframing
        .reframe_items(bed.project_id, &clips, TrackingSettings::default())
        .await
        .unwrap();
    bed.let_analyses_run().await;
    let finished = bed.finished_events();
    assert_eq!(finished.len(), 1);
    assert_eq!(finished[0], (Vec::new(), 1));
    let after = bed.project().await;
    for &clip in &clips {
        assert_eq!(reframe_of(&after, clip), reframe_of(&before, clip));
    }
}

#[tokio::test(start_paused = true)]
async fn agents_reframe_their_scope_and_add_a_reframed_version() {
    let bed = bed().await;
    // A face drifting right over the clip.
    bed.services.detector.set_faces(Arc::new(|time: Flicks| {
        let x = 0.2 + 0.05 * time.to_seconds_f64().min(10.0);
        vec![FaceBox {
            x,
            y: 0.2,
            width: 0.1,
            height: 0.14,
            score: 0.9,
        }]
    }));
    let round = bed
        .services
        .reframing
        .analyses_for(
            bed.project_id,
            &[example_ids::VIDEO_ASSET],
            Duration::from_secs(30),
        )
        .await
        .unwrap();
    assert!(
        round.waiting.is_empty() && round.failed.is_empty(),
        "{round:?}"
    );
    let analyses: BTreeMap<_, _> = round.ready;
    let agent = bed.agent(None).await;
    let clips = bed.clips().await;
    let report = bed
        .services
        .edits
        .apply(
            &agent,
            "Auto reframe",
            &[AgentEdit::ReframeClips(Box::new(ReframeClips {
                items: clips.iter().map(|&id| ItemRef::Id(id)).collect(),
                settings: TrackingSettings::default(),
                analyses: analyses.clone(),
            }))],
        )
        .await
        .unwrap();
    assert_eq!(report.changed.len(), clips.len());

    let sequences_before = bed.project().await.sequences.len();
    let report = bed
        .services
        .edits
        .apply(
            &agent,
            "Auto reframe",
            &[AgentEdit::AddReframedVersion(Box::new(ReframedVersion {
                name: "Vertical (IA)".to_owned(),
                format: vertical(),
                settings: TrackingSettings::default(),
                analyses,
            }))],
        )
        .await
        .unwrap();
    let project = bed.project().await;
    assert_eq!(project.sequences.len(), sequences_before + 1);
    let version = &project.sequences[1];
    assert_eq!(version.name, "Vertical (IA)");
    assert_ne!(report.created[0], []);
    assert!(
        video_clips(version)
            .iter()
            .all(|&clip| reframe_of(&project, clip).tracking().is_some())
    );

    // Reverting the task removes the version and the reframing.
    bed.services
        .tasks
        .revert(bed.project_id, agent.task_id)
        .await
        .unwrap();
    let reverted = bed.project().await;
    assert_eq!(reverted.sequences.len(), sequences_before);
    assert!(
        clips
            .iter()
            .all(|&clip| reframe_of(&reverted, clip).tracking().is_none())
    );
}
