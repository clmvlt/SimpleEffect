import { useStore } from 'zustand';
import { createStore } from 'zustand/vanilla';

/**
 * Whether a sequence is playing, published by the viewer's playback session and read by other
 * features (the timeline follows the playhead by pages while playing).
 */
export interface PlaybackStatus {
  /** 0 when paused; 1 real time; negative in reverse; 2 and 4 fast. */
  readonly rate: number;
}

/** The playback status store (one playback at a time). */
export const playbackStatusStore = createStore<PlaybackStatus>()(() => ({ rate: 0 }));

/** Publishes the playback rate. */
export function setPlaybackRate(rate: number): void {
  if (playbackStatusStore.getState().rate !== rate) playbackStatusStore.setState({ rate });
}

/** The playback rate, re-rendering when it changes (not on every frame). */
export function usePlaybackRate(): number {
  return useStore(playbackStatusStore, (state) => state.rate);
}
