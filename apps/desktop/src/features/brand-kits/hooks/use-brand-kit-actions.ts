import type { LibraryBrandKit } from '@se/protocol';
import { toast } from '@se/ui';
import { useTranslation } from 'react-i18next';

import { useCoreClient } from '../../../shared/core';
import { useDesktopBridge } from '../../../shared/desktop';
import { errorMessageKey } from '../../../shared/errors';
import { brandKitsApi } from '../api';

/** Extensions offered when adding logos. */
export const LOGO_EXTENSIONS = ['png', 'jpg', 'jpeg', 'webp', 'gif', 'bmp', 'tif', 'tiff'];

/** What the brand kit menu and editor do (each failure shown as a toast). */
export interface BrandKitActions {
  /** Gives the project a copy of `kit`, or no kit. */
  readonly setProjectKit: (projectId: string, kit: LibraryBrandKit | null) => Promise<void>;
  /** Adds the project's own kit to the library. */
  readonly addProjectKit: (projectId: string) => Promise<void>;
  /** Creates a kit (a copy of `copyOf` when given); resolves with it, or `null` on failure. */
  readonly create: (
    name: string,
    copyOf: LibraryBrandKit | null,
  ) => Promise<LibraryBrandKit | null>;
  /** Saves a kit; resolves with the saved version, or `null` on failure. */
  readonly save: (kit: LibraryBrandKit) => Promise<LibraryBrandKit | null>;
  readonly remove: (kit: LibraryBrandKit) => Promise<void>;
  /** Asks for images and adds them as logos; resolves with the kit as saved, or `null`. */
  readonly addLogos: (kit: LibraryBrandKit) => Promise<LibraryBrandKit | null>;
}

/** The brand kit actions over the core. */
export function useBrandKitActions(): BrandKitActions {
  const { t } = useTranslation();
  const client = useCoreClient();
  const bridge = useDesktopBridge();
  const api = brandKitsApi(client);
  const attempt = async <T>(run: () => Promise<T>): Promise<T | null> => {
    try {
      return await run();
    } catch (error) {
      toast.error(t(errorMessageKey(error)));
      return null;
    }
  };
  return {
    setProjectKit: async (projectId, kit) => {
      const done = await attempt(() => api.setProjectKit(projectId, kit?.id ?? null));
      if (done === null) return;
      toast.success(
        kit === null ? t('brandKit.menu.removed') : t('brandKit.menu.chosen', { name: kit.name }),
      );
    },
    addProjectKit: async (projectId) => {
      const kit = await attempt(() => api.addProjectKitToLibrary(projectId));
      if (kit !== null) toast.success(t('brandKit.menu.added', { name: kit.name }));
    },
    create: (name, copyOf) => attempt(() => api.create(name, copyOf?.id ?? null)),
    save: async (kit) => {
      const saved = await attempt(() => api.save(kit));
      if (saved !== null) {
        toast.success(t('brandKit.settings.saved', { name: saved.name, version: saved.version }));
      }
      return saved;
    },
    remove: async (kit) => {
      const done = await attempt(() => api.remove(kit.id));
      if (done !== null) toast.success(t('brandKit.settings.removed', { name: kit.name }));
    },
    addLogos: async (kit) => {
      const paths = await bridge.chooseFiles({
        title: t('brandKit.settings.logoTitle'),
        buttonLabel: t('brandKit.settings.logoButton'),
        filterName: t('brandKit.settings.imagesFilter'),
        extensions: LOGO_EXTENSIONS,
        allFilesName: t('brandKit.settings.allFiles'),
      });
      let current: LibraryBrandKit | null = null;
      for (const path of paths) {
        const saved = await attempt(() => api.addLogo(kit.id, path));
        if (saved !== null) current = saved;
      }
      return current;
    },
  };
}
