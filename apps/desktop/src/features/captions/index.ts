/**
 * Captions feature (prompt 23): the « Sous-titres » tab of the editor's right panel —
 * generation from the transcript, SRT import and export, the list of captions following playback
 * with inline editing (split, merge, emphasis, global shift), and the style panel (predefined
 * styles previewed in the viewer, fields applied to the selection or the whole track). The
 * viewer moves a caption's style when a caption is dragged (`captionsApi`).
 */
export { CaptionsView } from './components/CaptionsView';
export { captionsApi, type CaptionsApi } from './api';
export { captionsStore, useCaptionsPreferences } from './store/captions-store';
export { styleTargets, type StyleScope } from './services/caption-rows';
