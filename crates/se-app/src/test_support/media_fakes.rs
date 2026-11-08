//! Fakes of the file ports: media files and probes from in-memory tables, plus the files jobs
//! write (mezzanines, proxies, frames).

use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    sync::Mutex,
};

use async_trait::async_trait;
use se_domain::{AssetKind, AudioStreamInfo, ContentFingerprint, MediaInfo, VideoStreamInfo};

use super::lock;
use crate::ports::{MediaError, MediaFiles, MediaProber, ProbedMedia};

/// A fake file: its fingerprint and what probing it gives.
type FakeFile = (ContentFingerprint, Result<ProbedMedia, MediaError>);

/// Size every fake file reports, in bytes.
pub const FAKE_FILE_BYTES: u64 = 1000;

/// Files that exist, with their fingerprints and probe results, and files written by jobs.
#[derive(Debug, Default)]
pub struct FakeMedia {
    files: Mutex<BTreeMap<PathBuf, FakeFile>>,
    written: Mutex<BTreeSet<PathBuf>>,
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

    /// Adds a file written by a job (not media to import).
    pub fn add_written(&self, path: impl Into<PathBuf>) {
        lock(&self.written).insert(path.into());
    }

    /// Files written by jobs, in path order.
    pub fn written(&self) -> Vec<PathBuf> {
        lock(&self.written).iter().cloned().collect()
    }

    /// Removes a file (renamed or deleted by the user).
    pub fn remove(&self, path: &Path) {
        lock(&self.files).remove(path);
        lock(&self.written).remove(path);
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
        self.entry(path).is_some() || lock(&self.written).contains(path)
    }

    async fn fingerprint(&self, path: &Path) -> Result<ContentFingerprint, MediaError> {
        self.entry(path)
            .map(|(fingerprint, _)| fingerprint)
            .ok_or_else(|| MediaError::NotFound {
                path: path.to_path_buf(),
            })
    }

    async fn directory_size(&self, dir: &Path) -> u64 {
        let files = lock(&self.files)
            .keys()
            .filter(|path| path.starts_with(dir))
            .count();
        let written = lock(&self.written)
            .iter()
            .filter(|path| path.starts_with(dir))
            .count();
        u64::try_from(files + written).unwrap_or(u64::MAX) * FAKE_FILE_BYTES
    }

    async fn remove_directory(&self, dir: &Path) -> Result<(), MediaError> {
        lock(&self.files).retain(|path, _| !path.starts_with(dir));
        lock(&self.written).retain(|path| !path.starts_with(dir));
        Ok(())
    }

    /// Files written by jobs read as `fake:<path>`; added media files as `media:<path>`.
    async fn read_file(&self, path: &Path) -> Result<Vec<u8>, MediaError> {
        if lock(&self.written).contains(path) {
            return Ok(format!("fake:{}", path.display()).into_bytes());
        }
        match self.entry(path) {
            Some(_) => Ok(format!("media:{}", path.display()).into_bytes()),
            None => Err(MediaError::NotFound {
                path: path.to_path_buf(),
            }),
        }
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

/// A probed video with stereo sound (10 s, 1080p H.264 + AAC).
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
            audio_streams: vec![stereo("aac", Some("LC"))],
        },
    }
}

/// A probed ProRes 422 HQ video (10 bits 4:2:2, 1080p) with PCM sound.
#[must_use]
pub fn probed_prores() -> ProbedMedia {
    let mut probed = probed_video();
    "mov,mp4,m4a,3gp,3g2,mj2".clone_into(&mut probed.info.container);
    if let Some(video) = probed.info.video_stream.as_mut() {
        "prores".clone_into(&mut video.codec);
        video.profile = Some("HQ".to_owned());
        video.level = None;
        video.codec_tag = Some("apch".to_owned());
        video.pixel_format = Some("yuv422p10le".to_owned());
        video.bit_depth = Some(10);
        video.chroma_subsampling = Some("4:2:2".to_owned());
    }
    probed.info.audio_streams = vec![stereo("pcm_s16le", None)];
    probed
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

fn stereo(codec: &str, profile: Option<&str>) -> AudioStreamInfo {
    AudioStreamInfo {
        codec: codec.to_owned(),
        profile: profile.map(str::to_owned),
        channels: 2,
        sample_rate: 48_000,
    }
}
