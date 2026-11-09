import type { CoreClient, LibraryBrandKit } from '@se/protocol';

/** Protocol calls of the brand kits feature. */
export interface BrandKitsApi {
  /** Every kit of the library, by name. */
  readonly list: () => Promise<readonly LibraryBrandKit[]>;
  /** A new kit with default values, or a copy of `copyOf`. */
  readonly create: (name: string, copyOf: string | null) => Promise<LibraryBrandKit>;
  /** Saves a kit (a new version); open projects using it follow at once. */
  readonly save: (kit: LibraryBrandKit) => Promise<LibraryBrandKit>;
  /** Moves a kit to the system trash. */
  readonly remove: (kitId: string) => Promise<void>;
  /** Copies an image into a kit as a new logo. */
  readonly addLogo: (kitId: string, path: string) => Promise<LibraryBrandKit>;
  /** Gives a project a copy of a kit, or none (one undoable step). */
  readonly setProjectKit: (projectId: string, kitId: string | null) => Promise<void>;
  /** Adds a project's own kit to the library and links the project to it. */
  readonly addProjectKitToLibrary: (projectId: string) => Promise<LibraryBrandKit>;
}

/** The brand kit calls over `client`. */
export function brandKitsApi(client: Pick<CoreClient, 'send'>): BrandKitsApi {
  return {
    list: async () => (await client.send({ kind: 'listBrandKits' })).kits,
    create: async (name, copyOf) =>
      (await client.send({ kind: 'createBrandKit', name, copyOf })).kit,
    save: async (kit) => (await client.send({ kind: 'saveBrandKit', kit })).kit,
    remove: async (kitId) => {
      await client.send({ kind: 'removeBrandKit', kitId });
    },
    addLogo: async (kitId, path) =>
      (await client.send({ kind: 'addBrandKitLogo', kitId, path })).kit,
    setProjectKit: async (projectId, kitId) => {
      await client.send({ kind: 'setProjectBrandKit', projectId, kitId });
    },
    addProjectKitToLibrary: async (projectId) =>
      (await client.send({ kind: 'addProjectBrandKitToLibrary', projectId })).kit,
  };
}
