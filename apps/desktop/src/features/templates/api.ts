import type {
  CaptionStyleId,
  CoreClient,
  Flicks,
  InsertedLibraryTemplate,
  LibraryImportResult,
  LibraryTemplate,
  LibraryTemplateInfo,
  PlacedGraphic,
  SequenceId,
  TemplateId,
  TemplateUpdateReport,
  TrackId,
} from '@se/protocol';

/** Protocol calls of the templates feature on one project, each one transaction (one undo step). */
export interface TemplatesApi {
  /**
   * Places an instance of a project template with its default values (brand kit values for
   * brand-bound parameters): on `trackId` when free there, else the highest free video track.
   */
  readonly insert: (
    sequenceId: SequenceId,
    templateId: TemplateId,
    at: Flicks,
    duration: Flicks,
    trackId?: TrackId | null,
  ) => Promise<PlacedGraphic>;
  /** Removes a template no item uses. */
  readonly remove: (templateId: TemplateId) => Promise<void>;
  /** Saves a project template in the library (a new entry or a new version of its entry). */
  readonly saveToLibrary: (
    templateId: TemplateId,
    info: LibraryTemplateInfo,
  ) => Promise<LibraryTemplate>;
  /** Saves a project caption style in the library. */
  readonly saveCaptionStyle: (
    styleId: CaptionStyleId,
    info: LibraryTemplateInfo,
  ) => Promise<LibraryTemplate>;
  /** Copies a library template into the project and places an instance. */
  readonly insertFromLibrary: (
    sequenceId: SequenceId,
    libraryId: string,
    start: Flicks,
    trackId?: TrackId | null,
  ) => Promise<InsertedLibraryTemplate>;
  /** Replaces a project template by the newer version of its library entry. */
  readonly updateFromLibrary: (templateId: TemplateId) => Promise<TemplateUpdateReport>;
  /** Imports `.setemplate` files (checked by this window). */
  readonly importFiles: (paths: readonly string[]) => Promise<readonly LibraryImportResult[]>;
}

/** The template calls for project `projectId` over `client`. */
export function templatesApi(client: Pick<CoreClient, 'send'>, projectId: string): TemplatesApi {
  return {
    insert: async (sequenceId, templateId, at, duration, trackId) => {
      const result = await client.send({
        kind: 'addGraphicItem',
        projectId,
        sequenceId,
        at,
        duration,
        graphic: { kind: 'template', templateId },
        ...(trackId === null || trackId === undefined ? {} : { trackId }),
      });
      return { itemId: result.itemId, trackId: result.trackId };
    },
    remove: async (templateId) => {
      await client.send({ kind: 'removeTemplate', projectId, templateId });
    },
    saveToLibrary: async (templateId, info) =>
      (await client.send({ kind: 'saveTemplateToLibrary', projectId, templateId, info })).template,
    saveCaptionStyle: async (styleId, info) =>
      (await client.send({ kind: 'saveCaptionStyleToLibrary', projectId, styleId, info })).template,
    insertFromLibrary: async (sequenceId, libraryId, start, trackId) => {
      const result = await client.send({
        kind: 'insertLibraryTemplate',
        projectId,
        templateId: libraryId,
        sequenceId,
        trackId: trackId ?? null,
        start,
      });
      return { templateId: result.templateId, itemId: result.itemId, trackId: result.trackId };
    },
    updateFromLibrary: async (templateId) => {
      const result = await client.send({
        kind: 'updateTemplateFromLibrary',
        projectId,
        templateId,
      });
      return {
        version: result.version,
        libraryVersion: result.libraryVersion,
        reset: result.reset,
      };
    },
    importFiles: async (paths) =>
      (await client.send({ kind: 'importLibraryTemplates', projectId, paths: [...paths] })).results,
  };
}

/** Protocol calls on the library itself (no project). */
export interface LibraryApi {
  /** Every entry, by name. */
  readonly list: () => Promise<readonly LibraryTemplate[]>;
  /** Renames, describes, tags or refiles an entry. */
  readonly updateInfo: (libraryId: string, info: LibraryTemplateInfo) => Promise<LibraryTemplate>;
  /** Moves an entry to the system trash. */
  readonly remove: (libraryId: string) => Promise<void>;
  /** Writes an entry as a `.setemplate` file. */
  readonly exportFile: (libraryId: string, path: string) => Promise<void>;
}

/** The library calls over `client`. */
export function libraryApi(client: Pick<CoreClient, 'send'>): LibraryApi {
  return {
    list: async () => (await client.send({ kind: 'listLibraryTemplates' })).templates,
    updateInfo: async (templateId, info) =>
      (await client.send({ kind: 'updateLibraryTemplateInfo', templateId, info })).template,
    remove: async (templateId) => {
      await client.send({ kind: 'removeLibraryTemplate', templateId });
    },
    exportFile: async (templateId, path) => {
      await client.send({ kind: 'exportLibraryTemplate', templateId, path });
    },
  };
}
