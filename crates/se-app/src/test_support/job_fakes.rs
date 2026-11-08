//! Fakes of the job ports: a job runner whose runs are recorded and can be held, failed or
//! completed on demand; a frame extractor; the tools.

use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use se_domain::{
    AnalysisResult, AudioAnalysis, Flicks, LevelEnvelope, LoudnessCurve, SceneDetection,
    SilenceSettings, TimeRange,
};
use se_protocol::{JobKind, MediaToolsStatus};
use tokio::sync::Notify;

use super::{FakeMedia, lock};
use crate::ports::{
    FrameExtractor, FrameRequest, MediaError, MediaJobOutput, MediaJobRunner, MediaJobSpec,
    MediaTask, MediaTools, ProgressSink,
};

/// A level envelope of `duration_ms` (20 ms windows) at −15 dBFS, except at −60 dBFS during
/// `silences` (source milliseconds): what an analysis of speech with those pauses measures.
#[must_use]
pub fn levels_with_silences(duration_ms: i64, silences: &[(i64, i64)]) -> LevelEnvelope {
    let step_ms = 20;
    let values = (0..duration_ms / step_ms)
        .map(|index| {
            let middle = index * step_ms + step_ms / 2;
            let silent = silences
                .iter()
                .any(|(start, end)| *start <= middle && middle < *end);
            if silent { 60 } else { 15 }
        })
        .collect();
    LevelEnvelope {
        step: LevelEnvelope::STEP,
        values,
    }
}

/// The audio analysis every fake run returns: −16 LUFS, one silence from 2 s to 3 s (in the
/// silences and in 10 s of levels).
#[must_use]
pub fn fake_audio_analysis(silence: SilenceSettings) -> AudioAnalysis {
    AudioAnalysis {
        sample_rate: 48_000,
        channels: 2,
        integrated_lufs: Some(-16.0),
        loudness_range_lu: Some(6.0),
        true_peak_dbtp: Some(-1.0),
        sample_peak_dbfs: Some(-1.2),
        short_term: LoudnessCurve {
            step: Flicks::from_millis(500),
            values: vec![None, Some(-17.0)],
        },
        silence_settings: silence,
        silences: TimeRange::from_start_end(Flicks::from_millis(2_000), Flicks::from_millis(3_000))
            .into_iter()
            .collect(),
        levels: Some(levels_with_silences(10_000, &[(2_000, 3_000)])),
    }
}

/// The scene detection every fake run returns: changes at 4 s and 7 s.
#[must_use]
pub fn fake_scene_detection(threshold: f64) -> SceneDetection {
    SceneDetection {
        threshold,
        changes: vec![Flicks::from_millis(4_000), Flicks::from_millis(7_000)],
    }
}

/// Records runs; by default every run succeeds at once: cache jobs mark their outputs ready (see
/// [`FakeJobRunner::outputs_ready`]), transcodes write their file into the [`FakeMedia`] given to
/// [`FakeJobRunner::write_files_to`], analyses return [`fake_audio_analysis`] and
/// [`fake_scene_detection`]. [`FakeJobRunner::hold`] makes runs wait for
/// [`FakeJobRunner::release`]; [`FakeJobRunner::fail_next`] makes the next runs fail.
#[derive(Debug, Default)]
pub struct FakeJobRunner {
    state: Mutex<RunnerState>,
    released: Arc<Notify>,
}

#[derive(Debug, Default)]
struct RunnerState {
    runs: Vec<MediaJobSpec>,
    outputs: BTreeSet<(JobKind, PathBuf)>,
    held: bool,
    failures_left: usize,
    files: Option<Arc<FakeMedia>>,
}

impl FakeJobRunner {
    /// Specs of every run started so far.
    pub fn runs(&self) -> Vec<MediaJobSpec> {
        lock(&self.state).runs.clone()
    }

    /// Kinds of every run started so far, in order.
    pub fn run_kinds(&self) -> Vec<JobKind> {
        lock(&self.state)
            .runs
            .iter()
            .map(|spec| spec.task.kind())
            .collect()
    }

    /// Makes later runs wait for [`FakeJobRunner::release`].
    pub fn hold(&self) {
        lock(&self.state).held = true;
    }

    /// Lets held runs (and later ones) finish.
    pub fn release(&self) {
        lock(&self.state).held = false;
        self.released.notify_waiters();
    }

    /// Makes the next `count` runs fail.
    pub fn fail_next(&self, count: usize) {
        lock(&self.state).failures_left = count;
    }

    /// Marks the outputs of a `kind` job in `dir` as cached.
    pub fn add_output(&self, kind: JobKind, dir: &Path) {
        lock(&self.state).outputs.insert((kind, dir.to_path_buf()));
    }

    /// Makes successful transcodes write their output file into `files`.
    pub fn write_files_to(&self, files: Arc<FakeMedia>) {
        lock(&self.state).files = Some(files);
    }

    /// Forgets the cached outputs (the cache folder was cleaned).
    pub fn clear_outputs(&self) {
        lock(&self.state).outputs.clear();
    }
}

#[async_trait]
impl MediaJobRunner for FakeJobRunner {
    async fn outputs_ready(&self, kind: JobKind, output_dir: &Path) -> bool {
        lock(&self.state)
            .outputs
            .contains(&(kind, output_dir.to_path_buf()))
    }

    async fn run(
        &self,
        spec: &MediaJobSpec,
        progress: &dyn ProgressSink,
    ) -> Result<MediaJobOutput, MediaError> {
        let released = self.released.notified();
        let held = {
            let mut state = lock(&self.state);
            state.runs.push(spec.clone());
            state.held
        };
        progress.report(0.5);
        if held {
            released.await;
        }
        let mut state = lock(&self.state);
        if state.failures_left > 0 {
            state.failures_left -= 1;
            return Err(MediaError::Failed("ffmpeg exited with status 1".to_owned()));
        }
        Ok(match &spec.task {
            MediaTask::Thumbnails | MediaTask::WaveformPeaks => {
                state
                    .outputs
                    .insert((spec.task.kind(), spec.output_dir.clone()));
                MediaJobOutput::Files
            }
            MediaTask::Transcode(plan) => {
                if let Some(files) = &state.files {
                    files.add_written(plan.output.clone());
                }
                MediaJobOutput::Files
            }
            MediaTask::AudioAnalysis { silence } => MediaJobOutput::Analysis(
                AnalysisResult::AudioAnalysis(fake_audio_analysis(*silence)),
            ),
            MediaTask::SceneDetection { threshold } => MediaJobOutput::Analysis(
                AnalysisResult::SceneDetection(fake_scene_detection(*threshold)),
            ),
        })
    }
}

/// Records frame requests and writes their output into a [`FakeMedia`].
#[derive(Debug)]
pub struct FakeFrameExtractor {
    files: Arc<FakeMedia>,
    requests: Mutex<Vec<FrameRequest>>,
}

impl FakeFrameExtractor {
    /// An extractor writing into `files`.
    #[must_use]
    pub fn new(files: Arc<FakeMedia>) -> Self {
        Self {
            files,
            requests: Mutex::default(),
        }
    }

    /// Every request so far.
    pub fn requests(&self) -> Vec<FrameRequest> {
        lock(&self.requests).clone()
    }
}

#[async_trait]
impl FrameExtractor for FakeFrameExtractor {
    async fn extract_frame(&self, request: &FrameRequest) -> Result<(), MediaError> {
        lock(&self.requests).push(request.clone());
        self.files.add_written(request.output.clone());
        Ok(())
    }
}

/// Records the tools folder; its status lists no hardware encoder.
#[derive(Debug, Default)]
pub struct FakeMediaTools {
    dirs: Mutex<Vec<Option<PathBuf>>>,
}

impl FakeMediaTools {
    /// Every folder set so far.
    pub fn dirs(&self) -> Vec<Option<PathBuf>> {
        lock(&self.dirs).clone()
    }
}

#[async_trait]
impl MediaTools for FakeMediaTools {
    fn set_tools_dir(&self, dir: Option<PathBuf>) {
        lock(&self.dirs).push(dir);
    }

    async fn status(&self) -> MediaToolsStatus {
        MediaToolsStatus {
            ffmpeg_path: Some("/bin/ffmpeg".to_owned()),
            ffprobe_path: Some("/bin/ffprobe".to_owned()),
            ffmpeg_version: Some("9.0.2".to_owned()),
            hardware_encoders: Vec::new(),
            problem: None,
        }
    }
}
