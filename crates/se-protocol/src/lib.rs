//! # se-protocol
//!
//! Data transfer objects exchanged between `se-core` and its clients:
//! - the startup handshake line printed by `se-core` on stdout ([`CoreHandshake`]);
//! - the JSON messages of the `/rpc` WebSocket: [`ClientMessage`] (requests) and
//!   [`ServerMessage`] (responses and pushed events).
//!
//! Every type derives `ts_rs::TS`: `pnpm codegen` (the `se-codegen` binary) exports them to
//! `packages/protocol/src/generated/`, so the TypeScript side never hand-writes them (ADR-0010,
//! ADR-0012). After any change here, run `pnpm codegen` and commit the generated files.
//!
//! Allowed workspace dependencies: `se-domain` (see `docs/architecture.md` §3 and §6).
//! DTOs only: no logic beyond (de)serialisation.

mod ai;
mod audio_editing;
mod command;
mod command_result;
mod editing;
mod envelope;
mod error;
mod event;
mod fonts;
mod frame_renders;
mod graphics;
mod handshake;
mod jobs;
mod media;
mod media_settings;
mod project_entities;
mod project_session;
mod tasks;

pub use ai::{
    AgentEvent, AgentFailureKind, AiSettings, AiStatus, AuthStatus, EffortLevel,
    InstallationStatus, ModelOption, ProviderCapabilities, ProviderId, ProviderInfo,
};
pub use audio_editing::{ItemUpdate, NewTransition};
pub use command::Command;
pub use command_result::{CommandResult, CoreInfo};
pub use editing::{
    ItemPlacement, ItemTrim, PastedItem, PlaceMediaRequest, PlacementMode, TrackTarget,
};
pub use envelope::{
    ClientMessage, EventEnvelope, RequestEnvelope, RequestId, ResponseEnvelope, ResponseOutcome,
    ServerMessage,
};
pub use error::{ErrorCode, ProtocolError};
pub use event::CoreEvent;
pub use fonts::{FontFace, FontFamily, FontSource};
pub use frame_renders::FrameRenderRequest;
pub use graphics::{
    AddGraphicRequest, AnimationPresetInfo, AnimationPresetSample, ItemAnimation, NewGraphic,
    PlacedGraphic,
};
pub use handshake::CoreHandshake;
pub use jobs::{JobKind, JobProgress, JobSnapshot, JobState};
pub use media::{
    DecodabilityReport, DuplicateMedia, ImportFailure, ImportMediaRequest, ImportReport,
    RelinkAssetRequest, RelinkOutcome, ThumbnailSheet,
};
pub use media_settings::{
    DecoderCapabilities, DerivedKind, GenerateDerivedRequest, MediaSettings, MediaToolsStatus,
    ProjectCacheInfo, ProxyResolution,
};
pub use project_entities::{
    EntityDeletes, EntityUpserts, ItemEntity, MarkerEntity, ProjectHeader, ProjectPatch,
    ProjectSnapshot, SequenceEntity, TrackEntity,
};
pub use project_session::{
    CreateProjectRequest, FormatPresetDimensions, HistoryState, NewProjectDefaults, ProjectHistory,
    ProjectSession, RecentProject,
};
pub use tasks::{
    RunKind, SubmitTaskRequest, Task, TaskEventRecord, TaskRevertReport, TaskRun, TaskScope,
    TaskStatus, TaskUsage,
};
