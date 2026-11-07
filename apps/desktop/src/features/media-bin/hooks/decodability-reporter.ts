import { checkDecodability } from '@se/engine/capabilities';
import type { Asset } from '@se/protocol';

import { reportDecodability, type MediaClient } from '../api';

/** Checks with WebCodecs whether this machine decodes `asset` and tells the core. */
export async function reportDecodabilityOf(
  client: MediaClient,
  projectId: string,
  asset: Asset,
  check: typeof checkDecodability = checkDecodability,
): Promise<void> {
  if (asset.mediaInfo === null) {
    return;
  }
  const result = await check(asset.kind, asset.mediaInfo);
  await reportDecodability(client, { projectId, assetId: asset.id, ...result });
}
