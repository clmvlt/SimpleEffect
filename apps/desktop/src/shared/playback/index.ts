/**
 * Playback status shared across features: the rate of the running playback (the viewer owns
 * playback; the timeline follows it).
 */
export {
  playbackStatusStore,
  setPlaybackRate,
  usePlaybackRate,
  type PlaybackStatus,
} from './playback-status-store';
