import type { Track } from '@se/protocol';

/** The mixing state of a track (a domain `Track` or a mirror `TrackEntity` fits). */
export type MixTrack = Pick<Track, 'id' | 'kind' | 'muted' | 'solo' | 'volumeDb' | 'pan'>;

/** What a track strip applies: linear gain and stereo balance. */
export interface StripSettings {
  readonly gain: number;
  /** −1 (left) to 1 (right). */
  readonly pan: number;
}

/** Linear gain of `db` decibels (0 dB = 1). */
export function dbToGain(db: number): number {
  return 10 ** (db / 20);
}

/** Decibels of a linear level (`-Infinity` for silence). */
export function gainToDb(gain: number): number {
  return gain > 0 ? 20 * Math.log10(gain) : Number.NEGATIVE_INFINITY;
}

/**
 * Strip settings of every audio track (pure): a muted track is silent, and when at least one
 * track is solo only solo tracks are heard; otherwise the track's volume. Pan is clamped to
 * [−1, 1].
 */
export function stripSettings(tracks: readonly MixTrack[]): Map<string, StripSettings> {
  const audio = tracks.filter((track) => track.kind === 'audio');
  const anySolo = audio.some((track) => track.solo);
  const settings = new Map<string, StripSettings>();
  for (const track of audio) {
    const silent = track.muted || (anySolo && !track.solo);
    settings.set(track.id, {
      gain: silent ? 0 : dbToGain(track.volumeDb),
      pan: Math.max(-1, Math.min(1, track.pan)),
    });
  }
  return settings;
}
