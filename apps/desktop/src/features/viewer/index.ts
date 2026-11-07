/**
 * Viewer feature: the frame of the active sequence at the playhead, drawn by `@se/engine`
 * (prompt 12); real-time playback with sound, transport bar, meter and HUD (prompt 13).
 */
export { ViewerPanel, type ViewerPanelProps } from './components/ViewerPanel';
export type { AudioEngineFactory, AudioEngineRequest } from './services/audio-engine-factory';
export type { PreviewEngineFactory, PreviewEngineRequest } from './services/preview-engine-factory';
