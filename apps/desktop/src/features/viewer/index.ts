/**
 * Viewer feature: the frame of the active sequence at the playhead, drawn by `@se/engine`
 * (prompt 12); real-time playback with sound, transport bar, meter and HUD (prompt 13); frames
 * drawn offscreen for AI agents (prompt 18, `useAgentFrameRequests`). Exports (prompt 27) build
 * their scene with `sceneOf`, like the viewer, so they draw and mix what it plays.
 */
export { ViewerPanel, type ViewerPanelProps } from './components/ViewerPanel';
export { useAgentFrameRequests } from './hooks/use-agent-frame-requests';
export type { AudioEngineFactory, AudioEngineRequest } from './services/audio-engine-factory';
export type { PreviewEngineFactory, PreviewEngineRequest } from './services/preview-engine-factory';
export { sceneOf, type PlaybackScene, type SceneSource } from './services/viewer-scene';
