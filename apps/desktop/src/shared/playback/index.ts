/**
 * Playback shared across features: the rate of the running playback (the viewer owns playback;
 * the timeline follows it) and requests to hear a passage (`requestPlayRange`, prompt 22).
 */
export {
  playbackStatusStore,
  setPlaybackRate,
  usePlaybackRate,
  type PlaybackStatus,
} from './playback-status-store';
export { onPlayRequest, requestPlayRange, type PlayRequest } from './play-requests';
