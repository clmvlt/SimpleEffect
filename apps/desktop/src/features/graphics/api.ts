import type {
  AnimationPhase,
  AnimationPresetInfo,
  CoreClient,
  Flicks,
  ItemAnimation,
  ItemId,
  ItemUpdate,
  NewGraphic,
  PlacedGraphic,
  SequenceId,
} from '@se/protocol';

/** Protocol calls of the graphics tools, each edit one transaction (one undo step) in the core. */
export interface GraphicsApi {
  readonly addGraphicItem: (
    sequenceId: SequenceId,
    at: Flicks,
    duration: Flicks,
    graphic: NewGraphic,
  ) => Promise<PlacedGraphic>;
  readonly applyAnimationPreset: (
    itemIds: readonly ItemId[],
    preset: string,
    phase: AnimationPhase,
    duration: Flicks,
    start: Flicks | null,
  ) => Promise<void>;
  readonly setItemAnimations: (changes: readonly ItemAnimation[]) => Promise<void>;
  readonly updateItems: (updates: readonly ItemUpdate[]) => Promise<void>;
}

/** The graphics calls for project `projectId` over `client`. */
export function graphicsApi(client: Pick<CoreClient, 'send'>, projectId: string): GraphicsApi {
  return {
    addGraphicItem: async (sequenceId, at, duration, graphic) => {
      const result = await client.send({
        kind: 'addGraphicItem',
        projectId,
        sequenceId,
        at,
        duration,
        graphic,
      });
      return { itemId: result.itemId, trackId: result.trackId };
    },
    applyAnimationPreset: async (itemIds, preset, phase, duration, start) => {
      await client.send({
        kind: 'applyAnimationPreset',
        projectId,
        itemIds: [...itemIds],
        preset,
        phase,
        duration,
        start,
      });
    },
    setItemAnimations: async (changes) => {
      await client.send({ kind: 'setItemAnimations', projectId, animations: [...changes] });
    },
    updateItems: async (updates) => {
      await client.send({ kind: 'updateItems', projectId, updates: [...updates] });
    },
  };
}

/** The animation preset library (the same for every project). */
export async function listAnimationPresets(
  client: Pick<CoreClient, 'send'>,
): Promise<readonly AnimationPresetInfo[]> {
  const result = await client.send({ kind: 'listAnimationPresets' });
  return result.presets;
}
