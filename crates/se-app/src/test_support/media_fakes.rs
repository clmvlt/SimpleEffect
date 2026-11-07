//! Fakes of the media ports: files and probes from in-memory tables, and a job runner whose runs
//! are recorded and can be held, failed or completed on demand.

use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use se_domain::{AssetKind, AudioStreamInfo, ContentFingerprint, MediaInfo, VideoStreamInfo};
use se_protocol::JobKind;
use tokio::sync::Notify;

use super::lock;
use crate::ports::{
    MediaError, MediaFiles, MediaJobRunner, MediaJobSpec, MediaProber, ProbedMedia, ProgressSink,
};

/// A fake file: its fingerprint and what probing it gives.
type FakeFile = (ContentFingerprint, Result<ProbedMedia, MediaError>);

/// Files that exist, with their fingerprints and probe results.
#[derive(Debug, Default)]
pub struct FakeMedia {
    files: Mutex<BTreeMap<PathBuf, FakeFile>>,
}

impl FakeMedia {
    /// Adds (or replaces) a file whose content is identified by `content` (same number = same
    /// content) and whose probe gives `probed`.
    pub fn add(&self, path: impl Into<PathBuf>, content: u64, probed: ProbedMedia) {
        lock(&self.files).insert(path.into(), (fingerprint(content), Ok(probed)));
    }

    /// Adds a file the prober cannot read.
    pub fn add_unreadable(&self, path: impl Into<PathBuf>, content: u64) {
        let path = path.into();
        let error = MediaError::Unsupported {
            path: path.clone(),
            reason: "not media".to_owned(),
        };
        lock(&self.files).insert(path, (fingerprint(content), Err(error)));
    }

    /// Removes a file (renamed or deleted by the user).
    pub fn remove(&self, path: &Path) {
        lock(&self.files).remove(path);
    }

    fn entry(&self, path: &Path) -> Option<FakeFile> {
        lock(&self.files).get(path).cloned()
    }
}

/// The fingerprint of fake content number `content`.
#[must_use]
pub fn fingerprint(content: u64) -> ContentFingerprint {
    ContentFingerprint::new(content * 1000, 1, format!("{content:016x}"))
        .unwrap_or_else(|error| unreachable!("{error}"))
}

#[async_trait]
impl MediaFiles for FakeMedia {
    async fn is_file(&self, path: &Path) -> bool {
        self.entry(path).is_some()
    }

    async fn fingerprint(&self, path: &Path) -> Result<ContentFingerprint, MediaError> {
        self.entry(path)
            .map(|(fingerprint, _)| fingerprint)
            .ok_or_else(|| MediaError::NotFound {
                path: path.to_path_buf(),
            })
    }
}

#[async_trait]
impl MediaProber for FakeMedia {
    async fn probe(&self, path: &Path) -> Result<ProbedMedia, MediaError> {
        self.entry(path).map_or_else(
            || {
                Err(MediaError::NotFound {
                    path: path.to_path_buf(),
                })
            },
            |(_, probed)| probed,
        )
    }
}

/// A probed video with stereo sound (10 s, 1080p).
#[must_use]
pub fn probed_video() -> ProbedMedia {
    ProbedMedia {
        kind: AssetKind::Video,
        info: MediaInfo {
            duration: se_domain::Flicks::from_seconds_f64(10.0).ok(),
            container: "mov,mp4,m4a,3gp,3g2,mj2".to_owned(),
            video_stream: Some(VideoStreamInfo {
                codec: "h264".to_owned(),
                profile: Some("High".to_owned()),
                level: Some(40),
                codec_tag: Some("avc1".to_owned()),
                pixel_format: Some("yuv420p".to_owned()),
                bit_depth: Some(8),
                chroma_subsampling: Some("4:2:0".to_owned()),
                width: 1920,
                height: 1080,
                frame_rate: Some(se_domain::FrameRate::FPS_30),
                rotation: 0,
            }),
            audio_streams: vec![stereo()],
        },
    }
}

/// A probed sound file (10 s, stereo).
#[must_use]
pub fn probed_audio() -> ProbedMedia {
    let mut probed = probed_video();
    probed.kind = AssetKind::Audio;
    "wav".clone_into(&mut probed.info.container);
    probed.info.video_stream = None;
    probed
}

fn stereo() -> AudioStreamInfo {
    AudioStreamInfo {
        codec: "aac".to_owned(),
        profile: Some("LC".to_owned()),
        channels: 2,
        sample_rate: 48_000,
    }
}

/// Records runs; by default every run succeeds at once and writes its outputs (see
/// [`FakeJobRunner::outputs_ready`]). [`FakeJobRunner::hold`] makes runs wait for
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
}

impl FakeJobRunner {
    /// Specs of every run started so far.
    pub fn runs(&self) -> Vec<MediaJobSpec> {
        lock(&self.state).runs.clone()
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
    ) -> Result<(), MediaError> {
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
        state.outputs.insert((spec.kind, spec.output_dir.clone()));
        Ok(())
    }
}
