import type {
  CaptionStyleChoice,
  CaptionStyleId,
  CaptionStylePatch,
  CaptionsReport,
  CoreClient,
  Flicks,
  ItemId,
  ProjectId,
  SequenceId,
  TimeRange,
} from '@se/protocol';

/** What the captions feature needs from the core client. */
export type CaptionsClient = Pick<CoreClient, 'send'>;

/** The caption commands of one project (each edit one undo step). */
export interface CaptionsApi {
  generate(
    sequenceId: SequenceId,
    range: TimeRange | null,
    style: CaptionStyleChoice | null,
    trackName: string,
  ): Promise<CaptionsReport>;
  importSubtitles(
    sequenceId: SequenceId,
    path: string,
    style: CaptionStyleChoice | null,
    trackName: string,
  ): Promise<CaptionsReport>;
  exportSubtitles(sequenceId: SequenceId, path: string): Promise<number>;
  editText(itemId: ItemId, text: string): Promise<void>;
  setEmphasis(itemId: ItemId, wordIndices: readonly number[], emphasis: boolean): Promise<void>;
  merge(itemIds: readonly ItemId[]): Promise<ItemId>;
  split(itemId: ItemId, wordIndex: number): Promise<ItemId>;
  shift(itemIds: readonly ItemId[], delta: Flicks): Promise<void>;
  updateStyle(itemIds: readonly ItemId[], patch: CaptionStylePatch): Promise<CaptionStyleId[]>;
  applyPreset(itemIds: readonly ItemId[], preset: string): Promise<CaptionStyleId>;
}

/** Report fields of a caption result (its `kind` tag removed). */
function report(result: CaptionsReport & { readonly kind: string }): CaptionsReport {
  return {
    itemIds: result.itemIds,
    trackId: result.trackId,
    styleId: result.styleId,
    replaced: result.replaced,
    untranscribedAssets: result.untranscribedAssets,
  };
}

/** The caption commands of `projectId` through `client`. */
export function captionsApi(client: CaptionsClient, projectId: ProjectId): CaptionsApi {
  return {
    generate: async (sequenceId, range, style, trackName) =>
      report(
        await client.send({
          kind: 'generateCaptions',
          projectId,
          sequenceId,
          trackName,
          ...(range === null ? {} : { range }),
          ...(style === null ? {} : { style }),
        }),
      ),
    importSubtitles: async (sequenceId, path, style, trackName) =>
      report(
        await client.send({
          kind: 'importSubtitles',
          projectId,
          sequenceId,
          path,
          trackName,
          ...(style === null ? {} : { style }),
        }),
      ),
    exportSubtitles: async (sequenceId, path) =>
      (await client.send({ kind: 'exportSubtitles', projectId, sequenceId, path })).cueCount,
    editText: async (itemId, text) => {
      await client.send({ kind: 'editCaptionText', projectId, itemId, text });
    },
    setEmphasis: async (itemId, wordIndices, emphasis) => {
      await client.send({
        kind: 'setCaptionEmphasis',
        projectId,
        itemId,
        wordIndices: [...wordIndices],
        emphasis,
      });
    },
    merge: async (itemIds) =>
      (await client.send({ kind: 'mergeCaptions', projectId, itemIds: [...itemIds] })).itemId,
    split: async (itemId, wordIndex) =>
      (await client.send({ kind: 'splitCaption', projectId, itemId, wordIndex })).itemId,
    shift: async (itemIds, delta) => {
      await client.send({ kind: 'shiftCaptions', projectId, itemIds: [...itemIds], delta });
    },
    updateStyle: async (itemIds, patch) =>
      (
        await client.send({
          kind: 'updateCaptionStyle',
          projectId,
          itemIds: [...itemIds],
          patch,
        })
      ).styleIds,
    applyPreset: async (itemIds, preset) =>
      (
        await client.send({
          kind: 'applyCaptionPreset',
          projectId,
          itemIds: [...itemIds],
          preset,
        })
      ).styleId,
  };
}
