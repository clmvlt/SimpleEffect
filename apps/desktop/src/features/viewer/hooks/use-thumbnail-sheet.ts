import type { Asset, ThumbnailSheet } from '@se/protocol';
import { useEffect, useState } from 'react';

import { useAssetMediaUrl } from '../../media-bin';

/** The thumbnail sheet of a video asset and its image URL; `null` until both are known. */
export function useThumbnailSheet(
  asset: Asset,
): { readonly sheet: ThumbnailSheet; readonly imageUrl: string } | null {
  const layoutUrl = useAssetMediaUrl(asset, 'thumbnails', 'thumbnails');
  const imageUrl = useAssetMediaUrl(asset, 'thumbnailSheet', 'thumbnails');
  const [loaded, setLoaded] = useState<{ url: string; sheet: ThumbnailSheet } | null>(null);
  useEffect(() => {
    if (layoutUrl === null) return undefined;
    const controller = new AbortController();
    fetch(layoutUrl, { signal: controller.signal })
      .then(async (response) => (response.ok ? ((await response.json()) as ThumbnailSheet) : null))
      .then((sheet) => {
        if (sheet !== null) setLoaded({ url: layoutUrl, sheet });
      })
      .catch(() => {
        // Not generated yet, or aborted: the backdrop stays an outline.
      });
    return () => {
      controller.abort();
    };
  }, [layoutUrl]);
  if (loaded === null || loaded.url !== layoutUrl || imageUrl === null) return null;
  return { sheet: loaded.sheet, imageUrl };
}
