import type { CoreClient, Flicks, PlacedGraphic, SequenceId, TemplateId } from '@se/protocol';

/** Protocol calls of the templates feature, each one transaction (one undo step). */
export interface TemplatesApi {
  /** Places an instance with the template's default values on the highest free video track. */
  readonly insert: (
    sequenceId: SequenceId,
    templateId: TemplateId,
    at: Flicks,
    duration: Flicks,
  ) => Promise<PlacedGraphic>;
  /** Removes a template no item uses. */
  readonly remove: (templateId: TemplateId) => Promise<void>;
}

/** The template calls for project `projectId` over `client`. */
export function templatesApi(client: Pick<CoreClient, 'send'>, projectId: string): TemplatesApi {
  return {
    insert: async (sequenceId, templateId, at, duration) => {
      const result = await client.send({
        kind: 'addGraphicItem',
        projectId,
        sequenceId,
        at,
        duration,
        graphic: { kind: 'template', templateId },
      });
      return { itemId: result.itemId, trackId: result.trackId };
    },
    remove: async (templateId) => {
      await client.send({ kind: 'removeTemplate', projectId, templateId });
    },
  };
}
