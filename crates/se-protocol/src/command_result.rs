//! Successful results of commands.

use se_domain::{
    AssetId, CaptionStyleId, ChangesetId, ItemId, JobId, LinkGroupId, MarkerId, SequenceId,
    SubjectDetection, TimeRange, TrackId, Transcript, TransitionId,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    AiSettings, AiStatus, AnimationPresetInfo, AutoReframeReport, CaptionsReport, CutSummary,
    FillerOccurrence, FontFamily, ImportReport, InsertedLibraryTemplate, JobSnapshot,
    LibraryBrandKit, LibraryImportResult, LibraryTemplate, MediaSettings, MediaToolsStatus,
    NewProjectDefaults, PlacedGraphic, ProjectCacheInfo, ProjectSession, RecentProject,
    RelinkOutcome, SmartCutSettings, Task, TaskEventRecord, TaskRevertReport, TemplateUpdateReport,
    TranscriptSearchHit, TranscriptionStatus, VisionStatus,
};

/// The result of a [`crate::Command`], tagged by the **same `kind` as the command** it answers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum CommandResult {
    /// Answer to [`crate::Command::Ping`]: the core is alive and reading requests.
    Ping,
    /// Answer to [`crate::Command::GetCoreInfo`].
    GetCoreInfo(CoreInfo),
    /// Answer to [`crate::Command::GetNewProjectDefaults`].
    GetNewProjectDefaults(NewProjectDefaults),
    /// Answer to [`crate::Command::CreateProject`]: the new project, open.
    CreateProject(ProjectSession),
    /// Answer to [`crate::Command::OpenProject`].
    OpenProject(ProjectSession),
    /// Answer to [`crate::Command::CloseProject`]: saved and closed.
    CloseProject,
    /// Answer to [`crate::Command::ListRecentProjects`].
    ListRecentProjects {
        /// Most recently opened first.
        projects: Vec<RecentProject>,
    },
    /// Answer to [`crate::Command::RenameProject`]: the project as listed after renaming.
    RenameProject(RecentProject),
    /// Answer to [`crate::Command::DuplicateProject`]: the copy, now first in the recent list.
    DuplicateProject(RecentProject),
    /// Answer to [`crate::Command::TrashProject`]: the folder is in the OS trash.
    TrashProject,
    /// Answer to [`crate::Command::GetProjectSnapshot`].
    GetProjectSnapshot(ProjectSession),
    /// Answer to [`crate::Command::CreateSequence`].
    CreateSequence {
        /// Id of the new sequence.
        sequence_id: SequenceId,
    },
    /// Answer to [`crate::Command::RenameSequence`]: committed.
    RenameSequence,
    /// Answer to [`crate::Command::SetSequenceFormat`]: committed.
    SetSequenceFormat,
    /// Answer to [`crate::Command::DuplicateSequence`].
    DuplicateSequence {
        /// Id of the copy.
        sequence_id: SequenceId,
        /// What automatic reframing did, when it was asked (prompt 26).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        reframe: Option<AutoReframeReport>,
    },
    /// Answer to [`crate::Command::RemoveSequence`]: committed.
    RemoveSequence,
    /// Answer to [`crate::Command::Undo`]: committed (the patch arrives as an event).
    Undo,
    /// Answer to [`crate::Command::Redo`]: committed (the patch arrives as an event).
    Redo,
    /// Answer to [`crate::Command::AddTrack`].
    AddTrack {
        /// Id of the new track.
        track_id: TrackId,
    },
    /// Answer to [`crate::Command::RemoveTrack`]: committed.
    RemoveTrack,
    /// Answer to [`crate::Command::UpdateTrack`]: committed.
    UpdateTrack,
    /// Answer to [`crate::Command::ReorderTracks`]: committed.
    ReorderTracks,
    /// Answer to [`crate::Command::AddMarker`].
    AddMarker {
        /// Id of the new marker.
        marker_id: MarkerId,
    },
    /// Answer to [`crate::Command::UpdateMarker`]: committed.
    UpdateMarker,
    /// Answer to [`crate::Command::RemoveMarker`]: committed.
    RemoveMarker,
    /// Answer to [`crate::Command::PlaceMedia`].
    PlaceMedia {
        /// Ids of the new items, in placement order (picture before sound for each asset).
        item_ids: Vec<ItemId>,
    },
    /// Answer to [`crate::Command::MoveItems`]: committed.
    MoveItems,
    /// Answer to [`crate::Command::DuplicateItems`].
    DuplicateItems {
        /// Ids of the copies, in the order of the request.
        item_ids: Vec<ItemId>,
    },
    /// Answer to [`crate::Command::PasteItems`].
    PasteItems {
        /// Ids of the copies, in the order of the request.
        item_ids: Vec<ItemId>,
    },
    /// Answer to [`crate::Command::TrimItems`]: committed.
    TrimItems,
    /// Answer to [`crate::Command::SplitItems`].
    SplitItems {
        /// Ids of the new right parts, in the order of the request.
        item_ids: Vec<ItemId>,
    },
    /// Answer to [`crate::Command::DeleteItems`]: committed.
    DeleteItems,
    /// Answer to [`crate::Command::LinkItems`].
    LinkItems {
        /// The new link group.
        link_group: LinkGroupId,
    },
    /// Answer to [`crate::Command::UnlinkItems`]: committed.
    UnlinkItems,
    /// Answer to [`crate::Command::ResizeGap`]: committed (nothing when the length is unchanged).
    ResizeGap,
    /// Answer to [`crate::Command::UpdateItems`]: committed.
    UpdateItems,
    /// Answer to [`crate::Command::SetItemAnimation`]: committed.
    SetItemAnimation,
    /// Answer to [`crate::Command::AddTransitions`].
    AddTransitions {
        /// Ids of the transitions added, in the order of the request (cuts without room left
        /// out).
        transition_ids: Vec<TransitionId>,
    },
    /// Answer to [`crate::Command::UpdateTransition`]: committed.
    UpdateTransition,
    /// Answer to [`crate::Command::RemoveTransitions`]: committed.
    RemoveTransitions,
    /// Answer to [`crate::Command::SetItemAnimations`]: committed.
    SetItemAnimations,
    /// Answer to [`crate::Command::AddGraphicItem`]: the new item and its track.
    AddGraphicItem(PlacedGraphic),
    /// Answer to [`crate::Command::ApplyAnimationPreset`]: committed.
    ApplyAnimationPreset,
    /// Answer to [`crate::Command::ListAnimationPresets`].
    ListAnimationPresets {
        /// Every preset, in display order.
        presets: Vec<AnimationPresetInfo>,
    },
    /// Answer to [`crate::Command::ListFonts`].
    ListFonts {
        /// Bundled families first, then installed ones, each sorted by name.
        families: Vec<FontFamily>,
    },
    /// Answer to [`crate::Command::ImportMedia`].
    ImportMedia(ImportReport),
    /// Answer to [`crate::Command::RelinkAsset`].
    RelinkAsset {
        /// Whether the asset was relinked or the user must confirm first.
        outcome: RelinkOutcome,
    },
    /// Answer to [`crate::Command::RemoveAsset`]: committed (the patch arrives as an event).
    RemoveAsset,
    /// Answer to [`crate::Command::ListJobs`].
    ListJobs {
        /// Oldest first.
        jobs: Vec<JobSnapshot>,
    },
    /// Answer to [`crate::Command::CancelJob`]: cancelled (`jobCancelled` follows).
    CancelJob,
    /// Answer to [`crate::Command::ReportDecodability`]: recorded.
    ReportDecodability,
    /// Answer to [`crate::Command::ReportDecoderCapabilities`]: recorded.
    ReportDecoderCapabilities,
    /// Answer to [`crate::Command::GenerateDerivedMedia`]: queued (the asset's derived state
    /// becomes pending; job events follow).
    GenerateDerivedMedia,
    /// Answer to [`crate::Command::GetMediaSettings`].
    GetMediaSettings {
        /// The current settings.
        settings: MediaSettings,
    },
    /// Answer to [`crate::Command::UpdateMediaSettings`].
    UpdateMediaSettings {
        /// The settings as saved.
        settings: MediaSettings,
    },
    /// Answer to [`crate::Command::GetMediaToolsStatus`].
    GetMediaToolsStatus {
        /// Tools found and hardware encoders.
        status: MediaToolsStatus,
    },
    /// Answer to [`crate::Command::GetAudioSilences`].
    GetAudioSilences {
        /// The silences in source time, in order; `null` while the sound is not analysed.
        silences: Option<Vec<TimeRange>>,
    },
    /// Answer to [`crate::Command::GetProjectCache`].
    GetProjectCache {
        /// The cache folder and its size.
        cache: ProjectCacheInfo,
    },
    /// Answer to [`crate::Command::ClearProjectCache`].
    ClearProjectCache {
        /// The cache folder and what is left in it (files in use cannot be deleted).
        cache: ProjectCacheInfo,
    },
    /// Answer to [`crate::Command::SubmitTask`]: the task, queued (`taskUpdated` events follow).
    SubmitTask {
        /// The new task.
        task: Task,
    },
    /// Answer to [`crate::Command::CancelTask`]: cancelled (`taskUpdated` follows once its agent
    /// stopped).
    CancelTask,
    /// Answer to [`crate::Command::RetryTask`]: the task with its new run, queued.
    RetryTask {
        /// The task.
        task: Task,
    },
    /// Answer to [`crate::Command::FollowUpTask`]: the task with its new run, queued.
    FollowUpTask {
        /// The task.
        task: Task,
    },
    /// Answer to [`crate::Command::ListTasks`].
    ListTasks {
        /// Oldest first.
        tasks: Vec<Task>,
    },
    /// Answer to [`crate::Command::RevertTask`] (`projectPatched`, `taskChangesetUpdated` and
    /// `taskUpdated` follow when something was undone).
    RevertTask {
        /// What was undone and what was left.
        report: TaskRevertReport,
    },
    /// Answer to [`crate::Command::PreviewTaskRevert`]: the report a revert would give now
    /// (`reverted` says whether anything would be undone). Nothing changed.
    PreviewTaskRevert {
        /// What would be undone and what would be left.
        report: TaskRevertReport,
    },
    /// Answer to [`crate::Command::GetTaskChanges`].
    GetTaskChanges {
        /// The task's open changeset (`null` while it changed nothing or once reverted).
        changeset_id: Option<ChangesetId>,
        /// The items it added or changed that still exist, in id order.
        touched_items: Vec<ItemId>,
    },
    /// Answer to [`crate::Command::GetTaskEvents`].
    GetTaskEvents {
        /// The task's log, in order.
        events: Vec<TaskEventRecord>,
    },
    /// Answer to [`crate::Command::GetAiStatus`].
    GetAiStatus {
        /// The provider in use.
        status: AiStatus,
    },
    /// Answer to [`crate::Command::RefreshAiStatus`] (`aiStatusChanged` is published too).
    RefreshAiStatus {
        /// The provider in use, checked now.
        status: AiStatus,
    },
    /// Answer to [`crate::Command::GetAiSettings`].
    GetAiSettings {
        /// The current settings.
        settings: AiSettings,
    },
    /// Answer to [`crate::Command::UpdateAiSettings`].
    UpdateAiSettings {
        /// The settings as saved.
        settings: AiSettings,
    },
    /// Answer to [`crate::Command::ReportFrameRenderFailure`]: the agent was told.
    ReportFrameRenderFailure,
    /// Answer to [`crate::Command::ReportTemplateCheck`]: the core has the outcome.
    ReportTemplateCheck,
    /// Answer to [`crate::Command::RemoveTemplate`].
    RemoveTemplate,
    /// Answer to [`crate::Command::ListLibraryTemplates`].
    ListLibraryTemplates {
        /// The entries, by name.
        templates: Vec<LibraryTemplate>,
    },
    /// Answer to [`crate::Command::SaveTemplateToLibrary`].
    SaveTemplateToLibrary {
        /// The entry as saved.
        template: LibraryTemplate,
    },
    /// Answer to [`crate::Command::SaveCaptionStyleToLibrary`].
    SaveCaptionStyleToLibrary {
        /// The new entry.
        template: LibraryTemplate,
    },
    /// Answer to [`crate::Command::UpdateLibraryTemplateInfo`].
    UpdateLibraryTemplateInfo {
        /// The entry as saved.
        template: LibraryTemplate,
    },
    /// Answer to [`crate::Command::RemoveLibraryTemplate`].
    RemoveLibraryTemplate,
    /// Answer to [`crate::Command::InsertLibraryTemplate`].
    InsertLibraryTemplate(InsertedLibraryTemplate),
    /// Answer to [`crate::Command::UpdateTemplateFromLibrary`].
    UpdateTemplateFromLibrary(TemplateUpdateReport),
    /// Answer to [`crate::Command::ExportLibraryTemplate`].
    ExportLibraryTemplate,
    /// Answer to [`crate::Command::ImportLibraryTemplates`].
    ImportLibraryTemplates {
        /// One result per file, in order.
        results: Vec<LibraryImportResult>,
    },
    /// Answer to [`crate::Command::ListBrandKits`].
    ListBrandKits {
        /// The kits, by name.
        kits: Vec<LibraryBrandKit>,
    },
    /// Answer to [`crate::Command::CreateBrandKit`].
    CreateBrandKit {
        /// The new kit.
        kit: LibraryBrandKit,
    },
    /// Answer to [`crate::Command::SaveBrandKit`].
    SaveBrandKit {
        /// The kit as saved (new version).
        kit: LibraryBrandKit,
    },
    /// Answer to [`crate::Command::RemoveBrandKit`].
    RemoveBrandKit,
    /// Answer to [`crate::Command::AddBrandKitLogo`].
    AddBrandKitLogo {
        /// The kit with its new logo (saved).
        kit: LibraryBrandKit,
    },
    /// Answer to [`crate::Command::SetProjectBrandKit`].
    SetProjectBrandKit,
    /// Answer to [`crate::Command::AddProjectBrandKitToLibrary`].
    AddProjectBrandKitToLibrary {
        /// The new library kit.
        kit: LibraryBrandKit,
    },
    /// Answer to [`crate::Command::GetTranscriptionStatus`].
    GetTranscriptionStatus {
        /// The status.
        status: TranscriptionStatus,
    },
    /// Answer to [`crate::Command::UpdateTranscriptionPreferences`].
    UpdateTranscriptionPreferences {
        /// The status with the saved preferences.
        status: TranscriptionStatus,
    },
    /// Answer to [`crate::Command::StartTranscriptionDownload`]: the download started.
    StartTranscriptionDownload,
    /// Answer to [`crate::Command::CancelTranscriptionDownload`].
    CancelTranscriptionDownload,
    /// Answer to [`crate::Command::ImportTranscriptionModel`].
    ImportTranscriptionModel {
        /// The id of the imported model.
        model_id: String,
        /// The status with the model installed.
        status: TranscriptionStatus,
    },
    /// Answer to [`crate::Command::RemoveTranscriptionFile`].
    RemoveTranscriptionFile {
        /// The status without the file.
        status: TranscriptionStatus,
    },
    /// Answer to [`crate::Command::TranscribeAssets`].
    TranscribeAssets {
        /// One transcription job per asset, in order.
        job_ids: Vec<JobId>,
    },
    /// Answer to [`crate::Command::GetTranscript`].
    GetTranscript {
        /// The transcript of the asset's current content; `null` while it is not transcribed.
        transcript: Option<Transcript>,
    },
    /// Answer to [`crate::Command::SearchTranscripts`].
    SearchTranscripts {
        /// Best matches first (at most 50).
        hits: Vec<TranscriptSearchHit>,
    },
    /// Answer to [`crate::Command::SetTranscriptionSettings`]: the patch was published.
    SetTranscriptionSettings,
    /// Answer to [`crate::Command::GetSmartCutSettings`].
    GetSmartCutSettings(SmartCutSettings),
    /// Answer to [`crate::Command::UpdateSmartCutSettings`]: the settings as stored.
    UpdateSmartCutSettings(SmartCutSettings),
    /// Answer to [`crate::Command::RemoveWords`]: what was cut (the patch was published first;
    /// no cut and no transaction when nothing matched).
    RemoveWords(CutSummary),
    /// Answer to [`crate::Command::FindFillers`].
    FindFillers {
        /// In sequence order.
        occurrences: Vec<FillerOccurrence>,
        /// Media of the range not transcribed yet.
        untranscribed_assets: Vec<AssetId>,
    },
    /// Answer to [`crate::Command::PreviewSilenceCuts`]: the cuts it would make.
    PreviewSilenceCuts(CutSummary),
    /// Answer to [`crate::Command::RemoveSilences`]: what was cut.
    RemoveSilences(CutSummary),
    /// Answer to [`crate::Command::GenerateCaptions`]: what was made (the patch came first).
    GenerateCaptions(CaptionsReport),
    /// Answer to [`crate::Command::ImportSubtitles`]: what was made.
    ImportSubtitles(CaptionsReport),
    /// Answer to [`crate::Command::ExportSubtitles`]: the file was written.
    ExportSubtitles {
        /// Subtitles written.
        cue_count: u32,
    },
    /// Answer to [`crate::Command::EditCaptionText`]: committed.
    EditCaptionText,
    /// Answer to [`crate::Command::MergeCaptions`]: the caption holding them all.
    MergeCaptions {
        /// The first caption, now spanning the others.
        item_id: ItemId,
    },
    /// Answer to [`crate::Command::SplitCaption`]: the new right part.
    SplitCaption {
        /// The new caption.
        item_id: ItemId,
    },
    /// Answer to [`crate::Command::ShiftCaptions`]: committed.
    ShiftCaptions,
    /// Answer to [`crate::Command::SetCaptionEmphasis`]: committed.
    SetCaptionEmphasis,
    /// Answer to [`crate::Command::UpdateCaptionStyle`]: the styles the captions now use.
    UpdateCaptionStyle {
        /// Styles changed in place or created for the captions.
        style_ids: Vec<CaptionStyleId>,
    },
    /// Answer to [`crate::Command::ApplyCaptionPreset`]: the new style.
    ApplyCaptionPreset {
        /// The style made from the preset.
        style_id: CaptionStyleId,
    },
    /// Answer to [`crate::Command::GetVisionStatus`].
    GetVisionStatus {
        /// Whether faces can be detected here, and why not.
        status: VisionStatus,
    },
    /// Answer to [`crate::Command::AutoReframeItems`].
    AutoReframeItems(AutoReframeReport),
    /// Answer to [`crate::Command::GetSubjectDetection`].
    GetSubjectDetection {
        /// The faces of the sampled pictures; `null` while not analysed.
        detection: Option<SubjectDetection>,
    },
}

/// Facts about the running core, shown in the UI and useful in bug reports.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct CoreInfo {
    /// Version of the running `se-core` binary (semver).
    pub version: String,
    /// Operating system, as Rust's `std::env::consts::OS` (`windows`, `macos`, `linux`).
    pub os: String,
    /// CPU architecture, as Rust's `std::env::consts::ARCH` (`x86_64`, `aarch64`).
    pub arch: String,
    /// Absolute path of the app data folder (`SE_DATA_DIR`, set by the Electron app).
    pub app_data_dir: String,
    /// Absolute path of the folder holding the core's daily log files (`<app data>/logs`).
    pub logs_dir: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn results_carry_their_payload_next_to_the_kind() {
        let result = CommandResult::GetCoreInfo(CoreInfo {
            version: "0.1.0".to_owned(),
            os: "linux".to_owned(),
            arch: "x86_64".to_owned(),
            app_data_dir: "/data".to_owned(),
            logs_dir: "/data/logs".to_owned(),
        });
        let json = serde_json::to_string(&result).unwrap();
        assert_eq!(
            json,
            r#"{"kind":"getCoreInfo","version":"0.1.0","os":"linux","arch":"x86_64","appDataDir":"/data","logsDir":"/data/logs"}"#
        );
        assert_eq!(
            serde_json::from_str::<CommandResult>(&json).unwrap(),
            result
        );
    }

    #[test]
    fn struct_result_fields_are_camel_case() {
        let result = CommandResult::CreateSequence {
            sequence_id: SequenceId::from_test_index(2),
        };
        assert_eq!(
            serde_json::to_string(&result).unwrap(),
            r#"{"kind":"createSequence","sequenceId":"00000000-0002-7000-8000-000000000000"}"#
        );
    }

    #[test]
    fn task_changes_list_their_items() {
        let result = CommandResult::GetTaskChanges {
            changeset_id: None,
            touched_items: vec![ItemId::from_test_index(3)],
        };
        assert_eq!(
            serde_json::to_string(&result).unwrap(),
            r#"{"kind":"getTaskChanges","changesetId":null,"touchedItems":["00000000-0003-7000-8000-000000000000"]}"#
        );
    }

    #[test]
    fn caption_reports_sit_next_to_the_kind() {
        let result = CommandResult::GenerateCaptions(CaptionsReport {
            item_ids: vec![ItemId::from_test_index(4)],
            track_id: Some(TrackId::from_test_index(5)),
            style_id: None,
            replaced: 2,
            untranscribed_assets: Vec::new(),
        });
        assert_eq!(
            serde_json::to_string(&result).unwrap(),
            r#"{"kind":"generateCaptions","itemIds":["00000000-0004-7000-8000-000000000000"],"trackId":"00000000-0005-7000-8000-000000000000","styleId":null,"replaced":2,"untranscribedAssets":[]}"#
        );
    }

    #[test]
    fn unit_results_are_just_the_kind() {
        assert_eq!(
            serde_json::to_string(&CommandResult::Ping).unwrap(),
            r#"{"kind":"ping"}"#
        );
    }
}
