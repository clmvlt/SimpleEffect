import type {
  CoreClient,
  ItemAnimation,
  ItemId,
  ItemUpdate,
  PropertyAnimation,
} from '@se/protocol';

/** Protocol calls of the inspector, each one transaction (one undo step) in the core. */
export interface InspectorApi {
  /** Patches, then `animations` in the same step (a reframe mode with its scale). */
  readonly updateItems: (
    updates: readonly ItemUpdate[],
    animations?: readonly ItemAnimation[],
  ) => Promise<void>;
  readonly setItemAnimation: (itemId: ItemId, animation: PropertyAnimation) => Promise<void>;
  /** Several properties of several items in one step. */
  readonly setItemAnimations: (animations: readonly ItemAnimation[]) => Promise<void>;
}

/** The inspector calls for project `projectId` over `client`. */
export function inspectorApi(client: Pick<CoreClient, 'send'>, projectId: string): InspectorApi {
  return {
    updateItems: async (updates, animations = []) => {
      await client.send({
        kind: 'updateItems',
        projectId,
        updates: [...updates],
        ...(animations.length > 0 ? { animations: [...animations] } : {}),
      });
    },
    setItemAnimation: async (itemId, animation) => {
      await client.send({ kind: 'setItemAnimation', projectId, itemId, animation });
    },
    setItemAnimations: async (animations) => {
      await client.send({ kind: 'setItemAnimations', projectId, animations: [...animations] });
    },
  };
}
