//! A testbed with the example project open and an MCP server over it, plus readers of results.

#![allow(dead_code, reason = "each test file uses part of the shared bed")]

use std::path::{Path, PathBuf};

use rmcp::model::{CallToolResult, ContentBlock};
use se_app::{
    AgentContext,
    ports::StoredAnalysis,
    test_support::{MediaTestbed, settle},
};
use se_domain::{
    AnalysisResult, AssetId, AudioAnalysis, Flicks, LoudnessCurve, Project, ProjectId,
    SceneDetection, SilenceSettings, TaskId, TimeRange, test_support::example_project,
};
use se_mcp::{McpDeps, McpServer};
use se_protocol::TaskScope;
use serde_json::Value;

/// Where the example project lives in the fake repository.
pub const PROJECT_PATH: &str = "/films/Example.seproj";

/// The example project open over in-memory ports, and a server calling tools as one task.
pub struct Bed {
    pub services: MediaTestbed,
    pub server: McpServer,
    pub agent: AgentContext,
    pub project: Project,
}

impl Bed {
    /// The example project, open, with its media files present and its jobs settled.
    pub async fn open() -> Self {
        let bed = MediaTestbed::new(Path::new("/films"));
        let project = example_project();
        for (content, asset) in (1..).zip(&project.assets) {
            bed.files.add(
                asset.original_path.clone(),
                content,
                se_app::test_support::probed_video(),
            );
        }
        bed.repository
            .insert(PathBuf::from(PROJECT_PATH), project.clone(), Vec::new());
        let session = bed
            .projects
            .open_project(Path::new(PROJECT_PATH))
            .await
            .expect("the example project opens");
        settle().await;
        let project_id = session.snapshot.project.id;
        let server = McpServer::new(McpDeps {
            projects: bed.projects.clone(),
            media: bed.media.clone(),
            renders: bed.renders.clone(),
            template_checks: bed.template_checks.clone(),
            tokens: bed.tokens.clone(),
            edits: bed.edits.clone(),
            transcription: bed.transcription.clone(),
            smart_cuts: bed.smart_cuts.clone(),
            captions: bed.captions.clone(),
        });
        let agent = AgentContext {
            project_id,
            task_id: TaskId::from_test_index(77),
            run: 0,
            scope: TaskScope {
                sequence_id: project.sequences[0].id,
                range: None,
                tracks: None,
            },
        };
        Self {
            services: bed,
            server,
            agent,
            project,
        }
    }

    /// The project's id.
    pub fn project_id(&self) -> ProjectId {
        self.agent.project_id
    }

    /// Makes the bed's agent a real task of the task service (submitted with the bed's scope,
    /// its replayed run finished), so its edits open a changeset the task can revert.
    pub async fn run_task(&mut self) {
        let task = self
            .services
            .tasks
            .submit(se_protocol::SubmitTaskRequest {
                project_id: self.project_id(),
                prompt: "Edit the sequence".to_owned(),
                scope: self.agent.scope.clone(),
                provider: None,
                model: None,
                effort: None,
            })
            .await
            .expect("the task is accepted");
        for _ in 0..100 {
            let tasks = self.services.tasks.list(self.project_id()).await.unwrap();
            if tasks
                .iter()
                .any(|t| t.id == task.id && t.status == se_protocol::TaskStatus::Completed)
            {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        self.agent.task_id = task.id;
    }

    /// Calls `name` while a fake renderer answers its template check with `outcome` (prompt 24).
    pub async fn call_with_check(
        &self,
        name: &str,
        arguments: Value,
        outcome: se_protocol::TemplateCheckOutcome,
    ) -> CallToolResult {
        let checks = self.services.template_checks.clone();
        let renderer = self.services.renderer.clone();
        let already = renderer.template_checks().len();
        let answer = async move {
            for _ in 0..1000 {
                if let Some(request) = renderer.template_checks().get(already) {
                    checks
                        .deliver(&request.request_id, outcome)
                        .expect("the check waits");
                    return;
                }
                tokio::time::sleep(std::time::Duration::from_millis(1)).await;
            }
            panic!("no template check was asked");
        };
        let (result, ()) = tokio::join!(self.call(name, arguments), answer);
        result
    }

    /// The current document.
    pub async fn document(&self) -> Project {
        self.services
            .projects
            .project(self.project_id())
            .await
            .expect("the project is open")
            .as_ref()
            .clone()
    }

    /// Calls `name` with `arguments` as the bed's task.
    pub async fn call(&self, name: &str, arguments: Value) -> CallToolResult {
        let Value::Object(arguments) = arguments else {
            panic!("arguments must be an object");
        };
        self.server
            .call(self.agent.clone(), name, arguments)
            .await
            .expect("the tool exists")
    }

    /// Calls `name` and returns its first text block as JSON, asserting success.
    pub async fn json(&self, name: &str, arguments: Value) -> Value {
        let result = self.call(name, arguments).await;
        assert_ne!(
            result.is_error,
            Some(true),
            "{name} failed: {:?}",
            texts(&result)
        );
        first_json(&result)
    }

    /// Calls `name` and returns the error code, asserting failure.
    pub async fn error_code(&self, name: &str, arguments: Value) -> String {
        let result = self.call(name, arguments).await;
        assert_eq!(
            result.is_error,
            Some(true),
            "{name} succeeded: {:?}",
            texts(&result)
        );
        first_json(&result)["error"]["code"]
            .as_str()
            .expect("errors carry a code")
            .to_owned()
    }

    /// Stores an audio analysis of `asset` with `silences` (source milliseconds).
    pub fn set_audio_analysis(&self, asset: AssetId, silences: &[(i64, i64)]) {
        let analysis = AudioAnalysis {
            sample_rate: 48_000,
            channels: 2,
            integrated_lufs: Some(-15.94),
            loudness_range_lu: Some(6.0),
            true_peak_dbtp: Some(-1.04),
            sample_peak_dbfs: Some(-1.2),
            short_term: LoudnessCurve {
                step: Flicks::from_millis(500),
                values: (0..40)
                    .map(|index| Some(-30.0 + f64::from(index % 10)))
                    .collect(),
            },
            silence_settings: SilenceSettings::SPEECH,
            silences: silences
                .iter()
                .map(|(start, end)| ms_range(*start, *end))
                .collect(),
            levels: Some(se_app::test_support::levels_with_silences(60_000, silences)),
        };
        self.store(asset, AnalysisResult::AudioAnalysis(analysis));
    }

    /// Stores a scene detection of `asset` with changes at `changes` (source milliseconds).
    pub fn set_scenes(&self, asset: AssetId, changes: &[i64]) {
        let detection = SceneDetection {
            threshold: 0.3,
            changes: changes.iter().map(|ms| ms_time(*ms)).collect(),
        };
        self.store(asset, AnalysisResult::SceneDetection(detection));
    }

    /// Stores a transcript of `asset` (current content) made of `words`: (text, source start ms,
    /// source end ms, segment).
    pub fn set_transcript(&self, asset: AssetId, words: &[(&str, i64, i64, u32)]) {
        let millis = |ms: i64| Flicks::new(ms * 705_600).unwrap();
        let segment_count = words.iter().map(|word| word.3 + 1).max().unwrap_or(0);
        let segments = (0..segment_count)
            .map(|index| {
                let inside: Vec<_> = words.iter().filter(|word| word.3 == index).collect();
                se_domain::TranscriptSegment {
                    start: millis(inside[0].1),
                    end: millis(inside[inside.len() - 1].2),
                }
            })
            .collect();
        let transcript = se_domain::Transcript {
            language: "fr".to_owned(),
            model: "base".to_owned(),
            words: words
                .iter()
                .map(|(text, start, end, segment)| se_domain::TranscriptWord {
                    text: (*text).to_owned(),
                    start: millis(*start),
                    end: millis(*end),
                    confidence: if *text == "chaîne !" { 0.3 } else { 0.9 },
                    segment: *segment,
                })
                .collect(),
            segments,
        };
        self.services.repository.set_transcript(
            PROJECT_PATH,
            se_app::ports::StoredTranscript {
                asset_id: asset,
                content_key: self.content_key(asset),
                created_at: 0,
                backend: "cpu".to_owned(),
                transcript,
            },
        );
    }

    fn content_key(&self, asset: AssetId) -> String {
        self.project
            .assets
            .iter()
            .find(|candidate| candidate.id == asset)
            .and_then(|candidate| candidate.fingerprint.as_ref())
            .map(se_domain::ContentFingerprint::cache_key)
            .expect("example assets have fingerprints")
    }

    fn store(&self, asset: AssetId, result: AnalysisResult) {
        let content_key = self
            .project
            .assets
            .iter()
            .find(|candidate| candidate.id == asset)
            .and_then(|candidate| candidate.fingerprint.as_ref())
            .map(se_domain::ContentFingerprint::cache_key)
            .expect("example assets have fingerprints");
        self.services.repository.set_analysis(
            PROJECT_PATH,
            StoredAnalysis {
                asset_id: asset,
                content_key,
                created_at: 0,
                result,
            },
        );
    }
}

/// `ms` milliseconds.
pub fn ms_time(ms: i64) -> Flicks {
    Flicks::new(ms * (Flicks::PER_SECOND / 1000)).unwrap()
}

/// `[start, end)` in milliseconds.
pub fn ms_range(start: i64, end: i64) -> TimeRange {
    TimeRange::from_start_end(ms_time(start), ms_time(end)).unwrap()
}

/// The text blocks of a result.
pub fn texts(result: &CallToolResult) -> Vec<String> {
    result
        .content
        .iter()
        .filter_map(|block| match block {
            ContentBlock::Text(text) => Some(text.text.clone()),
            _ => None,
        })
        .collect()
}

/// The first text block of a result, parsed as JSON.
pub fn first_json(result: &CallToolResult) -> Value {
    let text = texts(result).into_iter().next().expect("a text block");
    serde_json::from_str(&text).expect("the text is JSON")
}

/// The timecodes of a list of ranges (`[["00:00:01.000", "00:00:02.000"], …]`).
pub fn range_codes(ranges: &Value) -> Vec<(String, String)> {
    ranges
        .as_array()
        .expect("a list of ranges")
        .iter()
        .map(|range| {
            (
                range["start"]["timecode"].as_str().unwrap().to_owned(),
                range["end"]["timecode"].as_str().unwrap().to_owned(),
            )
        })
        .collect()
}
