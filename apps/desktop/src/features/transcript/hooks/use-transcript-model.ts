import type { Asset, AssetId, ProjectId, SequenceEntity, Transcript } from '@se/protocol';

import { useProjectSelector } from '../../../shared/project-store';
import {
  EMPTY_SEQUENCE_TRANSCRIPT,
  assetsOf,
  sequenceTranscript,
  soundClipsOf,
  type SequenceTranscript,
  type SoundClip,
} from '../../../shared/transcript';
import { mediaTranscript } from '../services/media-words';
import { useAssetTranscripts } from './use-asset-transcripts';

/** What the panel shows: the active sequence's words, or one media file's. */
export type TranscriptMode = 'sequence' | 'media';

/** The panel's data. */
export interface TranscriptModel {
  /** Words in sequence time (sequence mode) or source time (media mode). */
  readonly transcript: SequenceTranscript;
  /** The words of the active sequence (media mode maps clicked words through it). */
  readonly sequenceWords: SequenceTranscript;
  /** Media with sound in view that have no transcript yet. */
  readonly untranscribed: readonly Asset[];
  /** Media with sound in view (transcribed or not). */
  readonly media: readonly Asset[];
  /** Whether transcripts are still being fetched. */
  readonly loading: boolean;
  /** The sound clips of the active sequence (cut passages are found between them). */
  readonly clips: readonly SoundClip[];
  /** The fetched transcript of an asset. */
  readonly transcriptOf: (assetId: AssetId) => Transcript | undefined;
}

const NO_ASSETS: readonly Asset[] = [];

/** Whether an asset has sound to transcribe. */
function hasSound(asset: Asset): boolean {
  return asset.kind !== 'image' && (asset.mediaInfo?.audioStreams.length ?? 0) > 0;
}

/**
 * The transcript panel's model for `mode`: the words of `sequence` through its sound clips, or
 * those of the media bin's selected asset (`assetId`) in its own time.
 */
export function useTranscriptModel(
  projectId: ProjectId | null,
  mode: TranscriptMode,
  sequence: SequenceEntity | null,
  assetId: string | null,
): TranscriptModel {
  const mirror = useProjectSelector((state) => state.mirror);
  const clips = mirror === null || sequence === null ? [] : soundClipsOf(mirror, sequence.id);
  const sequenceAssets =
    mirror === null
      ? NO_ASSETS
      : assetsOf(clips).flatMap((id) => {
          const asset = mirror.assets[id];
          return asset === undefined || !hasSound(asset) ? [] : [asset];
        });
  const selected = mirror === null || assetId === null ? undefined : mirror.assets[assetId];
  const mediaAssets = selected !== undefined && hasSound(selected) ? [selected] : NO_ASSETS;
  const { transcripts, loading } = useAssetTranscripts(projectId, [
    ...sequenceAssets,
    ...mediaAssets,
  ]);
  const sequenceWords =
    clips.length === 0
      ? EMPTY_SEQUENCE_TRANSCRIPT
      : sequenceTranscript(clips, (id) => transcripts.get(id));
  const media = mode === 'sequence' ? sequenceAssets : mediaAssets;
  const mediaWords =
    selected === undefined
      ? EMPTY_SEQUENCE_TRANSCRIPT
      : mediaTranscript(selected.id, transcripts.get(selected.id));
  return {
    transcript: mode === 'sequence' ? sequenceWords : mediaWords,
    sequenceWords,
    untranscribed: media.filter((asset) => asset.analysis.transcript === null),
    media,
    loading,
    clips,
    transcriptOf: (id) => transcripts.get(id),
  };
}
