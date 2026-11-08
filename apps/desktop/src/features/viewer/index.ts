/**
 * Viewer feature: the frame of the active sequence at the playhead, drawn by `@se/engine`
 * (prompt 12); real-time playback with sound, transport bar, meter and HUD (prompt 13); frames
 * drawn offscreen for AI agents (prompt 18, `useAgentFrameRequests`).
 */
export { ViewerPanel, type ViewerPanelProps } from './components/ViewerPanel';
export { useAgentFrameRequests } from './hooks/use-agent-frame-requests';
export type { AudioEngineFactory, AudioEngineRequest } from './services/audio-engine-factory';
export type { PreviewEngineFactory, PreviewEngineRequest } from './services/preview-engine-factory';
