import type { Flicks } from '@se/protocol';

import { AudioChunkPool } from '../audio/audio-chunk-pool';
import { audioClipsOf, type AudioClip, type PlayableSequence } from '../audio/audio-clips';
import { AudioGraph } from '../audio/audio-graph';
import { CHUNK_SECONDS, planPieces, sequenceTimeOf, type AudioPiece } from '../audio/audio-plan';
import { createMediabunnyAudioSource, type AudioReaderFactory } from '../audio/audio-source';
import { gainEnvelope } from '../audio/clip-gain';
import { soundFileOf } from '../audio/sound-file';
import type { MediaLocator } from '../decode/media-variant';
import type { AssetTable } from '../evaluate/item-layer';
import { FLICKS_PER_SECOND } from '../time/flicks';
import {
  AUDIO_BLOCK_SECONDS,
  EXPORT_CHANNELS,
  EXPORT_SAMPLE_RATE,
  audioBlocks,
  type AudioBlock,
} from './export-timing';

/** Creates the offline context a block renders in (`new OfflineAudioContext` in browsers). */
export type OfflineContextFactory = (
  channels: number,
  frames: number,
  sampleRate: number,
) => OfflineAudioContext;

/** What an {@link ExportAudioMixer} mixes. */
export interface ExportAudioMixerOptions {
  /** The exported sequence's tracks (with their mixing state) and the project's assets. */
  readonly sequence: PlayableSequence;
  readonly assets: AssetTable;
  /** Where the export reads assets' files (original or mezzanine). */
  readonly locator: MediaLocator;
  readonly start: Flicks;
  readonly end: Flicks;
  /** Opens assets' files (Mediabunny by default). */
  readonly createReader?: AudioReaderFactory;
  readonly sampleRate?: number;
  readonly blockSeconds?: number;
  readonly createContext?: OfflineContextFactory;
}

/** One mixed block: a buffer of samples per channel. */
export interface MixedBlock {
  readonly block: AudioBlock;
  readonly channels: Float32Array[];
}

/** Longest lead-in rendered before a block (very slow clips). */
const MAX_LEAD_IN_SECONDS = 10;

const defaultContext: OfflineContextFactory = (channels, frames, sampleRate) =>
  new OfflineAudioContext({ numberOfChannels: channels, length: frames, sampleRate });

/**
 * Mixes the sound of an export block by block (ADR-0038): each block renders in an
 * `OfflineAudioContext` through the preview's {@link AudioGraph} (clip gain → track gain/pan →
 * master) from the pieces {@link planPieces} gives for that stretch, with the preview's gain
 * envelopes. Each block renders from a short lead-in (see `leadInFrames`), and pieces running
 * past its end are cut by the context, so blocks join on the sample: the blocks equal a one-pass
 * mix. Decoded chunks are shared by all blocks (one chunk pool). Runs on the main thread: Workers have no `OfflineAudioContext`.
 */
export class ExportAudioMixer {
  readonly blocks: readonly AudioBlock[];
  private readonly clips: readonly AudioClip[];
  private readonly pool: AudioChunkPool;
  private readonly sampleRate: number;
  private readonly createContext: OfflineContextFactory;

  constructor(private readonly options: ExportAudioMixerOptions) {
    this.sampleRate = options.sampleRate ?? EXPORT_SAMPLE_RATE;
    this.createContext = options.createContext ?? defaultContext;
    this.blocks = audioBlocks(
      options.start,
      options.end,
      this.sampleRate,
      options.blockSeconds ?? AUDIO_BLOCK_SECONDS,
    );
    this.clips = audioClipsOf(options.sequence);
    this.pool = new AudioChunkPool({
      createReader: options.createReader ?? createMediabunnyAudioSource,
    });
    this.pool.setLocator((assetId) => soundFileOf(options.assets, options.locator, assetId));
  }

  /** Mixes block `index`; rejects for an index out of range. */
  async mix(index: number): Promise<MixedBlock> {
    const block = this.blocks[index];
    if (block === undefined) throw new Error(`the export has no sound block ${String(index)}`);
    const leadIn = this.leadInFrames(block);
    const frames = leadIn + block.frames;
    const context = this.createContext(EXPORT_CHANNELS, frames, this.sampleRate);
    const graph = new AudioGraph(context);
    graph.setMix(this.options.sequence.tracks, { immediate: true });
    const toFlicks = (count: number) => (count / this.sampleRate) * FLICKS_PER_SECOND;
    const timeStart = block.start - toFlicks(leadIn);
    const pieces = planPieces(
      this.clips,
      [
        {
          clockStart: 0,
          clockEnd: frames / this.sampleRate,
          timeStart,
          timeEnd: timeStart + toFlicks(frames),
        },
      ],
      { from: 0, fresh: true, spanEnd: this.options.end },
    );
    const chunks = await Promise.all(
      pieces.map((piece) => this.pool.load(piece.clip.assetId, piece.chunkIndex)),
    );
    pieces.forEach((piece, at) => {
      const chunk = chunks[at];
      if (chunk !== null && chunk !== undefined) play(graph, piece, chunk.buffer, chunk.start);
    });
    const rendered = await context.startRendering();
    graph.dispose();
    const channels: Float32Array[] = [];
    for (let channel = 0; channel < EXPORT_CHANNELS; channel++) {
      // A copy without the lead-in: the block is transferred to the export Worker.
      const data = rendered.getChannelData(Math.min(channel, rendered.numberOfChannels - 1));
      channels.push(data.slice(leadIn));
    }
    return { block, channels };
  }

  /**
   * Samples rendered before `block` and dropped. Chrome rounds a source's start offset to a whole
   * sample, so sound resuming mid-chunk at a block's first sample would be shifted by up to half a
   * sample until the next chunk boundary (a measurable seam). Starting the render one chunk of
   * the slowest clip earlier puts that resumed piece entirely in the lead-in; never before the
   * export's start, where a one-pass render starts too.
   */
  private leadInFrames(block: AudioBlock): number {
    if (block.index === 0) return 0;
    const slowest = Math.min(1, ...this.clips.map((clip) => clip.speed));
    const seconds = Math.min(MAX_LEAD_IN_SECONDS, CHUNK_SECONDS / slowest);
    const before = Math.round(block.seconds * this.sampleRate);
    return Math.min(before, Math.ceil(seconds * this.sampleRate) + 1);
  }

  /** Frees decoded sound and open files. */
  destroy(): void {
    this.pool.destroy();
  }
}

/** Starts the voice of `piece` from `buffer` (whose first sample is media second `bufferStart`). */
function play(
  graph: AudioGraph,
  piece: AudioPiece,
  buffer: AudioBuffer,
  bufferStart: number,
): void {
  const { clip } = piece;
  graph.play(
    {
      buffer,
      // Rounding may put a piece starting with the render a hair before 0.
      when: Math.max(0, piece.when),
      offset: piece.sourceStart - bufferStart,
      duration: piece.sourceEnd - piece.sourceStart,
      playbackRate: clip.speed,
      trackId: clip.trackId,
      gain: gainEnvelope(
        clip,
        sequenceTimeOf(clip, piece.sourceStart),
        sequenceTimeOf(clip, piece.sourceEnd),
      ),
      fade: 0,
    },
    () => undefined,
  );
}
