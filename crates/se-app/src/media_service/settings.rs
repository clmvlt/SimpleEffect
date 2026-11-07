//! Media settings and this machine's decoder capabilities, kept as `library.db` settings; the
//! status of the ffmpeg tools.

use std::path::{Path, PathBuf};

use se_protocol::{DecoderCapabilities, MediaSettings, MediaToolsStatus, ProxyResolution};

use super::MediaService;
use crate::{AppError, ports::AppLibraryRepository};

/// Keys of the media settings in `library.db`.
pub mod keys {
    /// Mezzanine CRF (a number).
    pub const MEZZANINE_QUALITY: &str = "media.mezzanineQuality";
    /// Proxy resolution (`540p` or `720p`).
    pub const PROXY_RESOLUTION: &str = "media.proxyResolution";
    /// Automatic proxies (`true` or `false`).
    pub const AUTO_PROXIES: &str = "media.autoProxies";
    /// Folder of ffmpeg and ffprobe chosen by the user (empty: the bundled tools).
    pub const TOOLS_DIR: &str = "media.toolsDir";
    /// Decoder capabilities reported by the renderer, one boolean per key.
    pub const DECODER_HEVC_MAIN10: &str = "media.decoders.hevcMain10";
    /// See [`DECODER_HEVC_MAIN10`].
    pub const DECODER_H264_HIGH: &str = "media.decoders.h264High";
    /// See [`DECODER_HEVC_MAIN10`].
    pub const DECODER_AAC: &str = "media.decoders.aac";
    /// See [`DECODER_HEVC_MAIN10`].
    pub const DECODER_FLAC: &str = "media.decoders.flac";
}

impl MediaService {
    /// The media settings (defaults for values never set or unreadable).
    ///
    /// # Errors
    /// [`AppError::Storage`] when `library.db` cannot be read.
    pub async fn media_settings(&self) -> Result<MediaSettings, AppError> {
        let library = self.deps.library.as_ref();
        let defaults = MediaSettings::default();
        let mezzanine_quality = read(library, keys::MEZZANINE_QUALITY)
            .await?
            .and_then(|value| value.parse::<u8>().ok())
            .filter(|quality| MediaSettings::MEZZANINE_QUALITY_RANGE.contains(quality))
            .unwrap_or(defaults.mezzanine_quality);
        let proxy_resolution = match read(library, keys::PROXY_RESOLUTION).await?.as_deref() {
            Some("720p") => ProxyResolution::P720,
            Some("540p") => ProxyResolution::P540,
            _ => defaults.proxy_resolution,
        };
        let auto_proxies = read_bool(library, keys::AUTO_PROXIES)
            .await?
            .unwrap_or(defaults.auto_proxies);
        let tools_dir = read(library, keys::TOOLS_DIR)
            .await?
            .filter(|dir| !dir.is_empty());
        Ok(MediaSettings {
            mezzanine_quality,
            proxy_resolution,
            auto_proxies,
            tools_dir,
        })
    }

    /// Saves `settings`, points the tools at the chosen folder, and queues what open projects now
    /// need (automatic proxies turned on). Returns the settings as saved.
    ///
    /// # Errors
    /// [`AppError::InvalidSetting`] for a quality out of range or a relative tools folder;
    /// [`AppError::Storage`].
    pub async fn update_media_settings(
        &self,
        settings: MediaSettings,
    ) -> Result<MediaSettings, AppError> {
        let range = MediaSettings::MEZZANINE_QUALITY_RANGE;
        if !range.contains(&settings.mezzanine_quality) {
            return Err(AppError::InvalidSetting {
                name: "mezzanineQuality",
                reason: format!("use a CRF from {} to {}", range.start(), range.end()),
            });
        }
        let tools_dir = settings.tools_dir.as_deref().filter(|dir| !dir.is_empty());
        if tools_dir.is_some_and(|dir| !Path::new(dir).is_absolute()) {
            return Err(AppError::InvalidSetting {
                name: "toolsDir",
                reason: "choose the folder with the folder picker (an absolute path)".to_owned(),
            });
        }
        let library = self.deps.library.as_ref();
        let resolution = match settings.proxy_resolution {
            ProxyResolution::P540 => "540p",
            ProxyResolution::P720 => "720p",
        };
        library
            .set_setting(
                keys::MEZZANINE_QUALITY,
                &settings.mezzanine_quality.to_string(),
            )
            .await?;
        library
            .set_setting(keys::PROXY_RESOLUTION, resolution)
            .await?;
        library
            .set_setting(keys::AUTO_PROXIES, &settings.auto_proxies.to_string())
            .await?;
        library
            .set_setting(keys::TOOLS_DIR, tools_dir.unwrap_or_default())
            .await?;
        self.deps.tools.set_tools_dir(tools_dir.map(PathBuf::from));
        self.schedule_open_projects().await;
        self.media_settings().await
    }

    /// The decoder capabilities last reported, if any.
    ///
    /// # Errors
    /// [`AppError::Storage`].
    pub async fn decoder_capabilities(&self) -> Result<Option<DecoderCapabilities>, AppError> {
        let library = self.deps.library.as_ref();
        let flags = (
            read_bool(library, keys::DECODER_HEVC_MAIN10).await?,
            read_bool(library, keys::DECODER_H264_HIGH).await?,
            read_bool(library, keys::DECODER_AAC).await?,
            read_bool(library, keys::DECODER_FLAC).await?,
        );
        Ok(match flags {
            (Some(hevc_main10), Some(h264_high), Some(aac), Some(flac)) => {
                Some(DecoderCapabilities {
                    hevc_main10,
                    h264_high,
                    aac,
                    flac,
                })
            }
            _ => None,
        })
    }

    /// Records what this machine's WebCodecs decodes (kept across restarts, so decisions made
    /// before the renderer reports use the last report).
    ///
    /// # Errors
    /// [`AppError::Storage`].
    pub async fn report_decoder_capabilities(
        &self,
        capabilities: DecoderCapabilities,
    ) -> Result<(), AppError> {
        if self.decoder_capabilities().await? == Some(capabilities) {
            return Ok(());
        }
        let library = self.deps.library.as_ref();
        for (key, value) in [
            (keys::DECODER_HEVC_MAIN10, capabilities.hevc_main10),
            (keys::DECODER_H264_HIGH, capabilities.h264_high),
            (keys::DECODER_AAC, capabilities.aac),
            (keys::DECODER_FLAC, capabilities.flac),
        ] {
            library.set_setting(key, &value.to_string()).await?;
        }
        Ok(())
    }

    /// Which ffmpeg/ffprobe are used and which hardware encoders work.
    pub async fn tools_status(&self) -> MediaToolsStatus {
        self.deps.tools.status().await
    }
}

async fn read(library: &dyn AppLibraryRepository, key: &str) -> Result<Option<String>, AppError> {
    Ok(library.setting(key).await?)
}

async fn read_bool(
    library: &dyn AppLibraryRepository,
    key: &str,
) -> Result<Option<bool>, AppError> {
    Ok(read(library, key)
        .await?
        .and_then(|value| value.parse::<bool>().ok()))
}
