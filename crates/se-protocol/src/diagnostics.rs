//! Diagnostics DTOs (prompt 30): what the window adds to a diagnostics archive, and what the
//! core wrote. The archive holds logs and facts about the machine, never project content or
//! credentials (`docs/architecture.md` §13).

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// What the window knows about itself, added to a diagnostics archive (`exportDiagnostics`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ClientDiagnostics {
    /// Version of the app (`package.json`).
    pub app_version: String,
    /// Runtime components and their versions (Electron, Chromium, Node.js, the engine…).
    pub components: Vec<ComponentVersion>,
    /// The graphics card as WebGL names it, when the window can tell.
    pub gpu: Option<String>,
    /// What WebCodecs can decode and encode on this machine.
    pub codecs: Vec<CodecSupport>,
    /// The interface language (`fr`).
    pub locale: String,
}

/// One component and its version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ComponentVersion {
    /// Its name (`Electron`).
    pub name: String,
    /// Its version (`45.2.3`).
    pub version: String,
}

/// Whether WebCodecs can decode and encode one codec here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct CodecSupport {
    /// The codec (`H.264`, `HEVC`, `VP9`, `AV1`).
    pub codec: String,
    /// Decoding is supported.
    pub decode: bool,
    /// Encoding is supported.
    pub encode: bool,
}

/// The archive `exportDiagnostics` wrote.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticsSummary {
    /// Where it was written (absolute).
    pub path: String,
    /// The files inside, in archive order (`report.json`, `logs/…`).
    pub files: Vec<String>,
    /// Its size in bytes.
    #[ts(type = "number")]
    pub bytes: u64,
}
