// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { AiSettings } from "./AiSettings";
import type { AiStatus } from "./AiStatus";
import type { AnimationPresetInfo } from "./AnimationPresetInfo";
import type { AssetId } from "./AssetId";
import type { AutoReframeReport } from "./AutoReframeReport";
import type { CaptionStyleId } from "./CaptionStyleId";
import type { CaptionsReport } from "./CaptionsReport";
import type { ChangesetId } from "./ChangesetId";
import type { CoreInfo } from "./CoreInfo";
import type { CutSummary } from "./CutSummary";
import type { FillerOccurrence } from "./FillerOccurrence";
import type { FontFamily } from "./FontFamily";
import type { ImportReport } from "./ImportReport";
import type { InsertedLibraryTemplate } from "./InsertedLibraryTemplate";
import type { ItemId } from "./ItemId";
import type { JobId } from "./JobId";
import type { JobSnapshot } from "./JobSnapshot";
import type { LibraryBrandKit } from "./LibraryBrandKit";
import type { LibraryImportResult } from "./LibraryImportResult";
import type { LibraryTemplate } from "./LibraryTemplate";
import type { LinkGroupId } from "./LinkGroupId";
import type { MarkerId } from "./MarkerId";
import type { MediaSettings } from "./MediaSettings";
import type { MediaToolsStatus } from "./MediaToolsStatus";
import type { NewProjectDefaults } from "./NewProjectDefaults";
import type { PlacedGraphic } from "./PlacedGraphic";
import type { ProjectCacheInfo } from "./ProjectCacheInfo";
import type { ProjectSession } from "./ProjectSession";
import type { RecentProject } from "./RecentProject";
import type { RelinkOutcome } from "./RelinkOutcome";
import type { SequenceId } from "./SequenceId";
import type { SmartCutSettings } from "./SmartCutSettings";
import type { SubjectDetection } from "./SubjectDetection";
import type { Task } from "./Task";
import type { TaskEventRecord } from "./TaskEventRecord";
import type { TaskRevertReport } from "./TaskRevertReport";
import type { TemplateUpdateReport } from "./TemplateUpdateReport";
import type { TimeRange } from "./TimeRange";
import type { TrackId } from "./TrackId";
import type { Transcript } from "./Transcript";
import type { TranscriptSearchHit } from "./TranscriptSearchHit";
import type { TranscriptionStatus } from "./TranscriptionStatus";
import type { TransitionId } from "./TransitionId";
import type { VisionStatus } from "./VisionStatus";

/**
 * The result of a [`crate::Command`], tagged by the **same `kind` as the command** it answers.
 */
export type CommandResult = { "kind": "ping" } | { "kind": "getCoreInfo" } & CoreInfo | { "kind": "getNewProjectDefaults" } & NewProjectDefaults | { "kind": "createProject" } & ProjectSession | { "kind": "openProject" } & ProjectSession | { "kind": "closeProject" } | { "kind": "listRecentProjects", 
/**
 * Most recently opened first.
 */
projects: Array<RecentProject>, } | { "kind": "renameProject" } & RecentProject | { "kind": "duplicateProject" } & RecentProject | { "kind": "trashProject" } | { "kind": "getProjectSnapshot" } & ProjectSession | { "kind": "createSequence", 
/**
 * Id of the new sequence.
 */
sequenceId: SequenceId, } | { "kind": "renameSequence" } | { "kind": "setSequenceFormat" } | { "kind": "duplicateSequence", 
/**
 * Id of the copy.
 */
sequenceId: SequenceId, 
/**
 * What automatic reframing did, when it was asked (prompt 26).
 */
reframe?: AutoReframeReport, } | { "kind": "removeSequence" } | { "kind": "undo" } | { "kind": "redo" } | { "kind": "addTrack", 
/**
 * Id of the new track.
 */
trackId: TrackId, } | { "kind": "removeTrack" } | { "kind": "updateTrack" } | { "kind": "reorderTracks" } | { "kind": "addMarker", 
/**
 * Id of the new marker.
 */
markerId: MarkerId, } | { "kind": "updateMarker" } | { "kind": "removeMarker" } | { "kind": "placeMedia", 
/**
 * Ids of the new items, in placement order (picture before sound for each asset).
 */
itemIds: Array<ItemId>, } | { "kind": "moveItems" } | { "kind": "duplicateItems", 
/**
 * Ids of the copies, in the order of the request.
 */
itemIds: Array<ItemId>, } | { "kind": "pasteItems", 
/**
 * Ids of the copies, in the order of the request.
 */
itemIds: Array<ItemId>, } | { "kind": "trimItems" } | { "kind": "splitItems", 
/**
 * Ids of the new right parts, in the order of the request.
 */
itemIds: Array<ItemId>, } | { "kind": "deleteItems" } | { "kind": "linkItems", 
/**
 * The new link group.
 */
linkGroup: LinkGroupId, } | { "kind": "unlinkItems" } | { "kind": "resizeGap" } | { "kind": "updateItems" } | { "kind": "setItemAnimation" } | { "kind": "addTransitions", 
/**
 * Ids of the transitions added, in the order of the request (cuts without room left
 * out).
 */
transitionIds: Array<TransitionId>, } | { "kind": "updateTransition" } | { "kind": "removeTransitions" } | { "kind": "setItemAnimations" } | { "kind": "addGraphicItem" } & PlacedGraphic | { "kind": "applyAnimationPreset" } | { "kind": "listAnimationPresets", 
/**
 * Every preset, in display order.
 */
presets: Array<AnimationPresetInfo>, } | { "kind": "listFonts", 
/**
 * Bundled families first, then installed ones, each sorted by name.
 */
families: Array<FontFamily>, } | { "kind": "importMedia" } & ImportReport | { "kind": "relinkAsset", 
/**
 * Whether the asset was relinked or the user must confirm first.
 */
outcome: RelinkOutcome, } | { "kind": "removeAsset" } | { "kind": "listJobs", 
/**
 * Oldest first.
 */
jobs: Array<JobSnapshot>, } | { "kind": "cancelJob" } | { "kind": "reportDecodability" } | { "kind": "reportDecoderCapabilities" } | { "kind": "generateDerivedMedia" } | { "kind": "getMediaSettings", 
/**
 * The current settings.
 */
settings: MediaSettings, } | { "kind": "updateMediaSettings", 
/**
 * The settings as saved.
 */
settings: MediaSettings, } | { "kind": "getMediaToolsStatus", 
/**
 * Tools found and hardware encoders.
 */
status: MediaToolsStatus, } | { "kind": "getAudioSilences", 
/**
 * The silences in source time, in order; `null` while the sound is not analysed.
 */
silences: Array<TimeRange> | null, } | { "kind": "getProjectCache", 
/**
 * The cache folder and its size.
 */
cache: ProjectCacheInfo, } | { "kind": "clearProjectCache", 
/**
 * The cache folder and what is left in it (files in use cannot be deleted).
 */
cache: ProjectCacheInfo, } | { "kind": "submitTask", 
/**
 * The new task.
 */
task: Task, } | { "kind": "cancelTask" } | { "kind": "retryTask", 
/**
 * The task.
 */
task: Task, } | { "kind": "followUpTask", 
/**
 * The task.
 */
task: Task, } | { "kind": "listTasks", 
/**
 * Oldest first.
 */
tasks: Array<Task>, } | { "kind": "revertTask", 
/**
 * What was undone and what was left.
 */
report: TaskRevertReport, } | { "kind": "previewTaskRevert", 
/**
 * What would be undone and what would be left.
 */
report: TaskRevertReport, } | { "kind": "getTaskChanges", 
/**
 * The task's open changeset (`null` while it changed nothing or once reverted).
 */
changesetId: ChangesetId | null, 
/**
 * The items it added or changed that still exist, in id order.
 */
touchedItems: Array<ItemId>, } | { "kind": "getTaskEvents", 
/**
 * The task's log, in order.
 */
events: Array<TaskEventRecord>, } | { "kind": "getAiStatus", 
/**
 * The provider in use.
 */
status: AiStatus, } | { "kind": "refreshAiStatus", 
/**
 * The provider in use, checked now.
 */
status: AiStatus, } | { "kind": "getAiSettings", 
/**
 * The current settings.
 */
settings: AiSettings, } | { "kind": "updateAiSettings", 
/**
 * The settings as saved.
 */
settings: AiSettings, } | { "kind": "reportFrameRenderFailure" } | { "kind": "reportTemplateCheck" } | { "kind": "removeTemplate" } | { "kind": "listLibraryTemplates", 
/**
 * The entries, by name.
 */
templates: Array<LibraryTemplate>, } | { "kind": "saveTemplateToLibrary", 
/**
 * The entry as saved.
 */
template: LibraryTemplate, } | { "kind": "saveCaptionStyleToLibrary", 
/**
 * The new entry.
 */
template: LibraryTemplate, } | { "kind": "updateLibraryTemplateInfo", 
/**
 * The entry as saved.
 */
template: LibraryTemplate, } | { "kind": "removeLibraryTemplate" } | { "kind": "insertLibraryTemplate" } & InsertedLibraryTemplate | { "kind": "updateTemplateFromLibrary" } & TemplateUpdateReport | { "kind": "exportLibraryTemplate" } | { "kind": "importLibraryTemplates", 
/**
 * One result per file, in order.
 */
results: Array<LibraryImportResult>, } | { "kind": "listBrandKits", 
/**
 * The kits, by name.
 */
kits: Array<LibraryBrandKit>, } | { "kind": "createBrandKit", 
/**
 * The new kit.
 */
kit: LibraryBrandKit, } | { "kind": "saveBrandKit", 
/**
 * The kit as saved (new version).
 */
kit: LibraryBrandKit, } | { "kind": "removeBrandKit" } | { "kind": "addBrandKitLogo", 
/**
 * The kit with its new logo (saved).
 */
kit: LibraryBrandKit, } | { "kind": "setProjectBrandKit" } | { "kind": "addProjectBrandKitToLibrary", 
/**
 * The new library kit.
 */
kit: LibraryBrandKit, } | { "kind": "getTranscriptionStatus", 
/**
 * The status.
 */
status: TranscriptionStatus, } | { "kind": "updateTranscriptionPreferences", 
/**
 * The status with the saved preferences.
 */
status: TranscriptionStatus, } | { "kind": "startTranscriptionDownload" } | { "kind": "cancelTranscriptionDownload" } | { "kind": "importTranscriptionModel", 
/**
 * The id of the imported model.
 */
modelId: string, 
/**
 * The status with the model installed.
 */
status: TranscriptionStatus, } | { "kind": "removeTranscriptionFile", 
/**
 * The status without the file.
 */
status: TranscriptionStatus, } | { "kind": "transcribeAssets", 
/**
 * One transcription job per asset, in order.
 */
jobIds: Array<JobId>, } | { "kind": "getTranscript", 
/**
 * The transcript of the asset's current content; `null` while it is not transcribed.
 */
transcript: Transcript | null, } | { "kind": "searchTranscripts", 
/**
 * Best matches first (at most 50).
 */
hits: Array<TranscriptSearchHit>, } | { "kind": "setTranscriptionSettings" } | { "kind": "getSmartCutSettings" } & SmartCutSettings | { "kind": "updateSmartCutSettings" } & SmartCutSettings | { "kind": "removeWords" } & CutSummary | { "kind": "findFillers", 
/**
 * In sequence order.
 */
occurrences: Array<FillerOccurrence>, 
/**
 * Media of the range not transcribed yet.
 */
untranscribedAssets: Array<AssetId>, } | { "kind": "previewSilenceCuts" } & CutSummary | { "kind": "removeSilences" } & CutSummary | { "kind": "generateCaptions" } & CaptionsReport | { "kind": "importSubtitles" } & CaptionsReport | { "kind": "exportSubtitles", 
/**
 * Subtitles written.
 */
cueCount: number, } | { "kind": "editCaptionText" } | { "kind": "mergeCaptions", 
/**
 * The first caption, now spanning the others.
 */
itemId: ItemId, } | { "kind": "splitCaption", 
/**
 * The new caption.
 */
itemId: ItemId, } | { "kind": "shiftCaptions" } | { "kind": "setCaptionEmphasis" } | { "kind": "updateCaptionStyle", 
/**
 * Styles changed in place or created for the captions.
 */
styleIds: Array<CaptionStyleId>, } | { "kind": "applyCaptionPreset", 
/**
 * The style made from the preset.
 */
styleId: CaptionStyleId, } | { "kind": "getVisionStatus", 
/**
 * Whether faces can be detected here, and why not.
 */
status: VisionStatus, } | { "kind": "autoReframeItems" } & AutoReframeReport | { "kind": "getSubjectDetection", 
/**
 * The faces of the sampled pictures; `null` while not analysed.
 */
detection: SubjectDetection | null, };
