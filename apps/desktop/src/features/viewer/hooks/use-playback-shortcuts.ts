import { useShortcutAction, useShortcutRegistry } from '../../../shared/shortcuts';
import type { PlaybackSession } from '../services/playback-session';
import { setViewerPreferences, viewerStore } from '../store/viewer-store';

/**
 * Binds the playback shortcuts (live in the timeline and the viewer): Space plays or pauses,
 * JKL shuttles — K pauses, holding K while pressing J or L steps one frame — and Ctrl/⌘+L toggles
 * looping.
 */
export function usePlaybackShortcuts(session: PlaybackSession): void {
  const registry = useShortcutRegistry();
  const holdingK = () => registry.isKeyHeld('k');
  useShortcutAction('togglePlay', () => {
    session.togglePlay();
  });
  useShortcutAction('shuttlePause', () => {
    session.pause();
  });
  useShortcutAction('shuttleForward', () => {
    if (holdingK()) session.step(1);
    else session.shuttle('forward');
  });
  useShortcutAction('shuttleReverse', () => {
    if (holdingK()) session.step(-1);
    else session.shuttle('reverse');
  });
  useShortcutAction('toggleLoop', () => {
    setViewerPreferences({ loop: !viewerStore.getState().loop });
  });
}
