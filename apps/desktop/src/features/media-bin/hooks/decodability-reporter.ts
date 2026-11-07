import { checkDecodability, checkDecoderCapabilities } from '@se/engine/capabilities';
import type { Asset } from '@se/protocol';

import { reportDecodability, reportDecoderCapabilities, type MediaClient } from '../api';

/** Checks what this machine decodes among the codecs of mezzanines and proxies, and tells the core. */
export async function reportCapabilitiesOf(
  client: MediaClient,
  check: typeof checkDecoderCapabilities = checkDecoderCapabilities,
): Promise<void> {
  await reportDecoderCapabilities(client, await check());
}

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
