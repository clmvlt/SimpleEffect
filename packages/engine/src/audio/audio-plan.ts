import type { PlaybackSegment } from '../playback/transport';
import { FLICKS_PER_SECOND } from '../time/flicks';
import type { AudioClip } from './audio-clips';

/** Decoded sound is cut into chunks of this many seconds of media time (chunk `k` = `[k, k+1)`). */
export const CHUNK_SECONDS = 1;

/** One sound to start: part of a chunk of a clip's asset, at a context time. */
export interface AudioPiece {
  readonly clip: AudioClip;
  readonly chunkIndex: number;
  /** Media seconds of the asset played: `[sourceStart, sourceEnd)`. */
  readonly sourceStart: number;
  readonly sourceEnd: number;
  /** Context time (seconds) at which `sourceStart` is heard. */
  readonly when: number;
}

/** The clock window being scheduled. */
export interface PlanWindow {
  readonly from: number;
  /** First window after a (re)start: sound already under way at `from` starts mid-chunk. */
  readonly fresh: boolean;
  /** Sequence time where playback stops or wraps (pieces never play past it). */
  readonly spanEnd: number;
}

/** Media seconds of `clip` at sequence `time` (flicks, not rounded). */
export function sourceSecondsAt(clip: AudioClip, time: number): number {
  return (clip.sourceStart + (time - clip.start) * clip.speed) / FLICKS_PER_SECOND;
}

/** Sequence time (flicks, not rounded) at which `clip` plays media second `source`. */
export function sequenceTimeOf(clip: AudioClip, source: number): number {
  return clip.start + (source * FLICKS_PER_SECOND - clip.sourceStart) / clip.speed;
}

/**
 * The pieces of sound starting inside a scheduling window (pure, real-time forward playback).
 *
 * Each clip's sound (its item extended through its cross-fades, `playStart`–`playEnd`) is cut at
 * chunk boundaries, clip edges and where playback stops or wraps;
 * a piece is scheduled in the window where it starts and plays to its end, even past the window,
 * so consecutive windows never cut sound mid-chunk. The only pieces starting before their chunk
 * are those where a run of sound begins: the first window after a (re)start, a clip's start,
 * and the start of a pass after a loop wrap. Start times derive from the segment's anchor, never
 * from previous pieces, so they stay exact to the sample over any duration.
 */
export function planPieces(
  clips: readonly AudioClip[],
  segments: readonly PlaybackSegment[],
  window: PlanWindow,
): AudioPiece[] {
  const pieces: AudioPiece[] = [];
  for (const segment of segments) {
    const passStarts = segment.clockStart > window.from || window.fresh;
    for (const clip of clips) {
      const sliceStart = Math.max(segment.timeStart, clip.playStart);
      const sliceEnd = Math.min(segment.timeEnd, clip.playEnd);
      if (sliceEnd <= sliceStart) continue;
      const runStarts =
        sliceStart === clip.playStart || (passStarts && sliceStart === segment.timeStart);
      const slice = {
        from: sourceSecondsAt(clip, sliceStart),
        to: sourceSecondsAt(clip, sliceEnd),
        limit: sourceSecondsAt(clip, Math.min(clip.playEnd, window.spanEnd)),
      };
      for (
        let chunk = Math.floor(slice.from / CHUNK_SECONDS);
        chunk * CHUNK_SECONDS < slice.to;
        chunk++
      ) {
        const chunkStart = chunk * CHUNK_SECONDS;
        if (chunkStart < slice.from && !runStarts) continue;
        const sourceStart = Math.max(chunkStart, slice.from);
        const sourceEnd = Math.min(chunkStart + CHUNK_SECONDS, slice.limit);
        if (sourceEnd <= sourceStart) continue;
        const offset = (sequenceTimeOf(clip, sourceStart) - segment.timeStart) / FLICKS_PER_SECOND;
        pieces.push({
          clip,
          chunkIndex: chunk,
          sourceStart,
          sourceEnd,
          when: segment.clockStart + offset,
        });
      }
    }
  }
  return pieces;
}

/**
 * `piece` trimmed so it starts no earlier than context time `earliest` (sound decoded late joins
 * playback in time); `null` when nothing is left.
 */
export function trimPiece(piece: AudioPiece, earliest: number): AudioPiece | null {
  if (piece.when >= earliest) return piece;
  const sourceStart = piece.sourceStart + (earliest - piece.when) * piece.clip.speed;
  if (sourceStart >= piece.sourceEnd) return null;
  return { ...piece, sourceStart, when: earliest };
}
