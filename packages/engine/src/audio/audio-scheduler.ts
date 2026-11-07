import type { AssetId, Flicks, TrackId } from '@se/protocol';

import { interpolateNumber, valueAt } from '../animation/interpolate';
import { playbackSegments, type PlaybackBounds, type Transport } from '../playback/transport';
import { FLICKS_PER_SECOND } from '../time/flicks';
import type { AudioChunk } from './audio-chunk';
import { sameClips, type AudioClip } from './audio-clips';
import {
  CHUNK_SECONDS,
  planPieces,
  sourceSecondsAt,
  trimPiece,
  type AudioPiece,
} from './audio-plan';
import { dbToGain } from './mix';

/** How far ahead of the render edge sound is scheduled (the anticipation). */
export const SCHEDULE_AHEAD_SECONDS = 0.2;
/** How far ahead chunks are decoded, so scheduling rarely waits. */
export const LOAD_AHEAD_SECONDS = 2;
/** Length of a scrub snippet. */
export const SCRUB_SECONDS = 0.08;
/** Minimum time between two scrub snippets (a drag would otherwise stack them). */
export const SCRUB_INTERVAL_SECONDS = 0.05;
/** A snippet decoded later than this after its request is dropped (the playhead moved on). */
const SCRUB_MAX_DELAY_SECONDS = 0.15;
const SCRUB_FADE_SECONDS = 0.005;
/** Sound decoded late starts this long after the render edge. */
const START_MARGIN_SECONDS = 0.005;

/** One `AudioBufferSourceNode` to start. */
export interface AudioVoiceRequest {
  readonly buffer: AudioBuffer;
  /** Context time of the first sample played. */
  readonly when: number;
  /** Seconds into the buffer. */
  readonly offset: number;
  /** Seconds of buffer to play. */
  readonly duration: number;
  /** The clip's speed (pitch follows). */
  readonly playbackRate: number;
  readonly trackId: TrackId;
  /** Linear clip gain. */
  readonly gain: number;
  /** Fade in and out length in seconds (scrub snippets), 0 for none. */
  readonly fade: number;
}

/** A started voice. */
export interface AudioVoice {
  stop(): void;
}

/** Where voices play (the Web Audio graph; a recorder in tests). */
export interface AudioOutput {
  /** The context's render edge: the earliest time a sound can start. */
  currentTime(): number;
  play(request: AudioVoiceRequest, onEnded: () => void): AudioVoice;
}

/** Decoded chunks of assets (`null` from `load`: the asset has no sound to play there). */
export interface AudioChunkProvider {
  peek(assetId: AssetId, index: number): AudioChunk | null;
  load(assetId: AssetId, index: number): Promise<AudioChunk | null>;
}

/**
 * Schedules the sound of a sequence ahead of the audio clock (`docs/architecture.md` §8): every
 * tick it starts the pieces beginning within the next 200 ms (`planPieces`), decodes chunks up
 * to 2 s ahead, and restarts cleanly on any jump. Sound plays at real-time forward speed only:
 * other speeds (JKL 2×, 4×, reverse) are silent. Also plays the short snippets of audio scrub.
 */
export class AudioScheduler {
  private clips: readonly AudioClip[] = [];
  private transport: Transport | null = null;
  private bounds: PlaybackBounds | null = null;
  private cursor: number | null = null;
  private generation = 0;
  private readonly voices = new Set<AudioVoice>();
  private lastScrub = Number.NEGATIVE_INFINITY;
  private pendingScrub: Flicks | null = null;

  constructor(
    private readonly output: AudioOutput,
    private readonly chunks: AudioChunkProvider,
  ) {}

  /** Whether sound is playing or a scrub snippet waits: the caller keeps ticking meanwhile. */
  get busy(): boolean {
    return this.audible() || this.pendingScrub !== null;
  }

  get voiceCount(): number {
    return this.voices.size;
  }

  /** New clips: scheduled sound restarts only when they changed. */
  setClips(clips: readonly AudioClip[]): void {
    if (sameClips(clips, this.clips)) return;
    this.clips = clips;
    if (this.audible()) this.restart();
  }

  /** Playback started, stopped, jumped or changed speed. */
  setTransport(transport: Transport | null, bounds: PlaybackBounds): void {
    this.transport = transport;
    this.bounds = bounds;
    this.restart();
    this.tick();
  }

  /** Schedules what enters the anticipation window; plays a waiting scrub snippet. */
  tick(): void {
    const now = this.output.currentTime();
    if (this.pendingScrub !== null && now - this.lastScrub >= SCRUB_INTERVAL_SECONDS) {
      const time = this.pendingScrub;
      this.pendingScrub = null;
      this.scrub(time);
    }
    const { transport, bounds } = this;
    if (transport === null || bounds === null || !this.audible()) return;
    const fresh = this.cursor === null;
    const from = this.cursor ?? Math.max(now, transport.anchorClock);
    const to = Math.max(from, now + SCHEDULE_AHEAD_SECONDS);
    if (to > from) {
      const segments = playbackSegments(transport, bounds, from, to);
      const window = { from, fresh, spanEnd: bounds.span.end };
      for (const piece of planPieces(this.clips, segments, window)) this.schedule(piece);
      this.cursor = to;
    }
    this.preload(transport, bounds, to, now + LOAD_AHEAD_SECONDS);
  }

  /** Plays a short snippet of the sound at `time` (paused playhead moves). */
  scrub(time: Flicks): void {
    if (this.audible()) return;
    const now = this.output.currentTime();
    if (now - this.lastScrub < SCRUB_INTERVAL_SECONDS) {
      this.pendingScrub = time;
      return;
    }
    this.lastScrub = now;
    const generation = this.generation;
    for (const clip of this.clips) {
      if (time < clip.start || time >= clip.end) continue;
      const sourceStart = sourceSecondsAt(clip, time);
      const limit = sourceSecondsAt(clip, clip.end);
      const piece: AudioPiece = {
        clip,
        chunkIndex: Math.floor(sourceStart / CHUNK_SECONDS),
        sourceStart,
        sourceEnd: Math.min(limit, sourceStart + SCRUB_SECONDS * clip.speed),
        when: now + START_MARGIN_SECONDS,
      };
      this.withChunk(piece, generation, (chunk) => {
        if (this.output.currentTime() - now > SCRUB_MAX_DELAY_SECONDS) return;
        this.start(chunk, piece, SCRUB_FADE_SECONDS);
      });
    }
  }

  /** Stops every voice (destroy). */
  stop(): void {
    this.generation += 1;
    for (const voice of this.voices) voice.stop();
    this.voices.clear();
    this.cursor = null;
  }

  private audible(): boolean {
    return this.transport?.rate === 1;
  }

  private restart(): void {
    this.stop();
    this.pendingScrub = null;
  }

  private schedule(piece: AudioPiece): void {
    this.withChunk(piece, this.generation, (chunk) => {
      this.start(chunk, piece, 0);
    });
  }

  /** Runs `play` with the chunk of `piece` now if decoded, else once decoded (same run only). */
  private withChunk(
    piece: AudioPiece,
    generation: number,
    play: (chunk: AudioChunk) => void,
  ): void {
    const cached = this.chunks.peek(piece.clip.assetId, piece.chunkIndex);
    if (cached !== null) {
      play(cached);
      return;
    }
    void this.chunks.load(piece.clip.assetId, piece.chunkIndex).then((chunk) => {
      if (chunk !== null && generation === this.generation) play(chunk);
    });
  }

  private start(chunk: AudioChunk, planned: AudioPiece, fade: number): void {
    const piece = trimPiece(planned, this.output.currentTime() + START_MARGIN_SECONDS);
    if (piece === null) return;
    const { clip } = piece;
    const voice = this.output.play(
      {
        buffer: chunk.buffer,
        when: piece.when,
        offset: piece.sourceStart - chunk.start,
        duration: piece.sourceEnd - piece.sourceStart,
        playbackRate: clip.speed,
        trackId: clip.trackId,
        gain: clipGain(clip, piece),
        fade,
      },
      () => this.voices.delete(voice),
    );
    this.voices.add(voice);
  }

  /** Decodes the chunks the next seconds of playback will need. */
  private preload(transport: Transport, bounds: PlaybackBounds, from: number, to: number): void {
    for (const segment of playbackSegments(transport, bounds, from, to)) {
      for (const clip of this.clips) {
        const sliceStart = Math.max(segment.timeStart, clip.start);
        const sliceEnd = Math.min(segment.timeEnd, clip.end);
        if (sliceEnd <= sliceStart) continue;
        const first = Math.floor(sourceSecondsAt(clip, sliceStart) / CHUNK_SECONDS);
        const last = Math.floor(sourceSecondsAt(clip, sliceEnd) / CHUNK_SECONDS);
        for (let index = first; index <= last; index++) {
          if (this.chunks.peek(clip.assetId, index) === null)
            void this.chunks.load(clip.assetId, index);
        }
      }
    }
  }
}

/**
 * The clip's gain where `piece` starts. TODO(prompt-14): volume keyframes become gain ramps
 * (today a keyframed volume is sampled once per piece).
 */
function clipGain(clip: AudioClip, piece: AudioPiece): number {
  const local = Math.round((piece.sourceStart * FLICKS_PER_SECOND - clip.sourceStart) / clip.speed);
  return dbToGain(valueAt(clip.volumeDb, local as Flicks, interpolateNumber, 0));
}
