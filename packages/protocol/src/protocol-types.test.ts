import { describe, expectTypeOf, it } from 'vitest';

import type { CoreInfo, JobSnapshot, ProjectPatch, ProjectSession } from './generated';
import type {
  CommandKind,
  CommandsWithoutResult,
  CoreEventKind,
  CoreEventOf,
  ResultFor,
} from './protocol-types';

// These checks run at compile time (`tsc -b` in `pnpm check`): a Rust command added without a
// result variant of the same kind breaks the build here.
describe('derived protocol types', () => {
  it('pairs every command with a result of the same kind', () => {
    expectTypeOf<CommandsWithoutResult>().toBeNever();
    expectTypeOf<CommandKind>().toEqualTypeOf<
      | 'ping'
      | 'getCoreInfo'
      | 'getNewProjectDefaults'
      | 'createProject'
      | 'openProject'
      | 'closeProject'
      | 'listRecentProjects'
      | 'renameProject'
      | 'duplicateProject'
      | 'trashProject'
      | 'getProjectSnapshot'
      | 'createSequence'
      | 'renameSequence'
      | 'setSequenceFormat'
      | 'duplicateSequence'
      | 'removeSequence'
      | 'undo'
      | 'redo'
      | 'addTrack'
      | 'removeTrack'
      | 'updateTrack'
      | 'reorderTracks'
      | 'addMarker'
      | 'updateMarker'
      | 'removeMarker'
      | 'placeMedia'
      | 'setItemAnimations'
      | 'addGraphicItem'
      | 'applyAnimationPreset'
      | 'listAnimationPresets'
      | 'listFonts'
      | 'moveItems'
      | 'duplicateItems'
      | 'pasteItems'
      | 'trimItems'
      | 'splitItems'
      | 'deleteItems'
      | 'linkItems'
      | 'unlinkItems'
      | 'resizeGap'
      | 'updateItems'
      | 'setItemAnimation'
      | 'addTransitions'
      | 'updateTransition'
      | 'removeTransitions'
      | 'importMedia'
      | 'relinkAsset'
      | 'removeAsset'
      | 'listJobs'
      | 'cancelJob'
      | 'reportDecodability'
      | 'reportDecoderCapabilities'
      | 'generateDerivedMedia'
      | 'getMediaSettings'
      | 'updateMediaSettings'
      | 'getMediaToolsStatus'
      | 'getAudioSilences'
      | 'getProjectCache'
      | 'clearProjectCache'
      | 'submitTask'
      | 'cancelTask'
      | 'retryTask'
      | 'followUpTask'
      | 'listTasks'
      | 'revertTask'
      | 'previewTaskRevert'
      | 'getTaskChanges'
      | 'getTaskEvents'
      | 'getAiStatus'
      | 'refreshAiStatus'
      | 'getAiSettings'
      | 'updateAiSettings'
      | 'reportFrameRenderFailure'
      | 'reportTemplateCheck'
      | 'removeTemplate'
      | 'getTranscriptionStatus'
      | 'updateTranscriptionPreferences'
      | 'startTranscriptionDownload'
      | 'cancelTranscriptionDownload'
      | 'importTranscriptionModel'
      | 'removeTranscriptionFile'
      | 'transcribeAssets'
      | 'getTranscript'
      | 'searchTranscripts'
      | 'setTranscriptionSettings'
      | 'getSmartCutSettings'
      | 'updateSmartCutSettings'
      | 'removeWords'
      | 'findFillers'
      | 'previewSilenceCuts'
      | 'removeSilences'
      | 'generateCaptions'
      | 'importSubtitles'
      | 'exportSubtitles'
      | 'editCaptionText'
      | 'mergeCaptions'
      | 'splitCaption'
      | 'shiftCaptions'
      | 'setCaptionEmphasis'
      | 'updateCaptionStyle'
      | 'applyCaptionPreset'
      | 'listLibraryTemplates'
      | 'saveTemplateToLibrary'
      | 'saveCaptionStyleToLibrary'
      | 'updateLibraryTemplateInfo'
      | 'removeLibraryTemplate'
      | 'insertLibraryTemplate'
      | 'updateTemplateFromLibrary'
      | 'exportLibraryTemplate'
      | 'importLibraryTemplates'
      | 'listBrandKits'
      | 'createBrandKit'
      | 'saveBrandKit'
      | 'removeBrandKit'
      | 'addBrandKitLogo'
      | 'setProjectBrandKit'
      | 'addProjectBrandKitToLibrary'
    >();
  });

  it('derives the result type from the command kind', () => {
    expectTypeOf<ResultFor<'ping'>>().toEqualTypeOf<{ kind: 'ping' }>();
    expectTypeOf<ResultFor<'getCoreInfo'>>().toExtend<CoreInfo>();
    expectTypeOf<ResultFor<'openProject'>>().toExtend<ProjectSession>();
    expectTypeOf<ResultFor<'relinkAsset'>['outcome']>().toEqualTypeOf<
      'relinked' | 'contentDiffers'
    >();
    expectTypeOf<ResultFor<'getMediaSettings'>['settings']['proxyResolution']>().toEqualTypeOf<
      '540p' | '720p'
    >();
  });

  it('derives the event type from the event kind', () => {
    expectTypeOf<CoreEventKind>().toEqualTypeOf<
      | 'coreReady'
      | 'coreShuttingDown'
      | 'resyncRequired'
      | 'projectOpened'
      | 'projectPatched'
      | 'projectClosed'
      | 'historyChanged'
      | 'jobQueued'
      | 'jobProgress'
      | 'jobCompleted'
      | 'jobFailed'
      | 'jobCancelled'
      | 'taskUpdated'
      | 'taskEventAppended'
      | 'taskChangesetUpdated'
      | 'aiStatusChanged'
      | 'renderFramesRequested'
      | 'templateCheckRequested'
      | 'transcriptionStatusChanged'
      | 'libraryChanged'
    >();
    expectTypeOf<CoreEventOf<'jobCompleted'>['job']>().toEqualTypeOf<JobSnapshot>();
    expectTypeOf<CoreEventOf<'projectPatched'>>().toExtend<ProjectPatch>();
    expectTypeOf<CoreEventOf<'resyncRequired'>['missedEvents']>().toEqualTypeOf<number>();
  });
});
