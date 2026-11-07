import type { Asset, AssetKind } from '@se/protocol';

import { searchable } from '../../../shared/text';

/** Kind filter of the media bin. */
export type KindFilter = 'all' | AssetKind;

/** The last segment of a path, `/` or `\` separated (paths come from any OS). */
export function fileName(path: string): string {
  return path.split(/[\\/]/).at(-1) ?? path;
}

/** File name of an asset's original. */
export function assetName(asset: Asset): string {
  return fileName(asset.originalPath);
}

/**
 * The assets in import order. Asset ids are UUIDv7, created in import order, so sorting by id
 * gives the project's order even after a patch re-adds an asset at the end of the mirror.
 */
export function sortAssets(assets: Readonly<Record<string, Asset>>): readonly Asset[] {
  return Object.values(assets).sort((left, right) =>
    left.id < right.id ? -1 : left.id > right.id ? 1 : 0,
  );
}

/** The assets of `kind` whose file name contains `query` (accents and case ignored). */
export function filterAssets(
  assets: readonly Asset[],
  kind: KindFilter,
  query: string,
): readonly Asset[] {
  const needle = searchable(query.trim());
  return assets.filter(
    (asset) =>
      (kind === 'all' || asset.kind === kind) &&
      (needle === '' || searchable(assetName(asset)).includes(needle)),
  );
}

/** Whether the renderer still has to check if WebCodecs decodes the asset's streams. */
export function needsDecodabilityCheck(asset: Asset): boolean {
  const { video, audio } = asset.decodability;
  return !asset.missing && asset.mediaInfo !== null && (video === 'unknown' || audio === 'unknown');
}
