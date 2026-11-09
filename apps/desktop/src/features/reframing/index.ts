/**
 * Automatic reframing (prompt 26, ADR-0037): clips follow their subject from the faces found in
 * their media. The inspector's « Cadrage » shows {@link AutoReframeControls}; the version dialog
 * asks {@link useVisionStatus}; the viewer draws the trajectory and the faces of the frame
 * ({@link useSubjectDetection}, {@link facesAt}, {@link focusTrajectory}); the editor mounts
 * {@link useAutoReframeEvents}.
 */
export { AutoReframeControls } from './components/AutoReframeControls';
export { useAutoReframe, useAutoReframeEvents, type AutoReframe } from './hooks/use-auto-reframe';
export { useSubjectDetection, useVisionStatus } from './hooks/use-vision';
export { resetVisionStore } from './store/vision-store';
export { facesAt, focusKeyframes, focusTrajectory } from './services/subject-path';
export { DEFAULT_TRACKING_SETTINGS, isTracked, trackingOf } from './services/tracking';
