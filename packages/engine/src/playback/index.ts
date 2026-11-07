/**
 * `@se/engine/playback`: real-time playback logic, pure and dependency-free — the transport
 * (positions derived from the master clock, loops), JKL shuttle rates, frame pacing (late frames
 * skipped and counted) and the `PlaybackController` that drives them on each display refresh.
 */
export {
  PlaybackController,
  type FrameTicker,
  type PlaybackClock,
  type PlaybackControllerOptions,
  type PlaybackState,
} from './playback-controller';
export { PlaybackStatsCounter, type PlaybackStats } from './playback-stats';
export { frameDurationFlicks, frameIndexAt, frameStart, skippedFrames } from './frame-pacing';
export { nextShuttleRate, SHUTTLE_SPEEDS, type ShuttleDirection } from './shuttle';
export {
  playbackSegments,
  transportPosition,
  type PlaybackBounds,
  type PlaybackSegment,
  type PlaybackSpan,
  type Transport,
  type TransportPosition,
} from './transport';
