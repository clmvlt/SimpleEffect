import type { AssetId, CoreEndpoint, ExportId } from '@se/protocol';
import type { StreamTargetChunk } from 'mediabunny';

import type { MediaLocator } from '../decode/media-variant';

/** The two byte streams of an export: the muxed file (or picture) and the WAV ffmpeg encodes. */
export type ExportStreamName = 'data' | 'audio';

/** Where an export's bytes go (the core's upload routes; memory in tests). */
export interface ExportSink {
  /** Writes `bytes` at `position` of `stream` (muxers go back to patch headers). */
  write(stream: ExportStreamName, position: number, bytes: Uint8Array): Promise<void>;
  /** Sends one RGBA frame for ffmpeg (masters). */
  frame(rgba: Uint8Array): Promise<void>;
  /** Sends the export's thumbnail. */
  thumbnail(jpeg: Blob): Promise<void>;
}

/** The core refused an upload because the export is over (cancelled, or failed elsewhere). */
export class ExportStoppedError extends Error {
  constructor() {
    super('the export was stopped');
    this.name = 'ExportStoppedError';
  }
}

/** A writable stream for Mediabunny's `StreamTarget` writing to `stream` of `sink`. */
export function sinkWritable(
  sink: ExportSink,
  stream: ExportStreamName,
): WritableStream<StreamTargetChunk> {
  return new WritableStream<StreamTargetChunk>({
    write: (chunk) => sink.write(stream, chunk.position, chunk.data),
  });
}

/** `path` on the core with the session token (and extra query parameters). */
function coreUrl(endpoint: CoreEndpoint, path: string, query: Record<string, string> = {}): string {
  const url = new URL(path, endpoint.url);
  url.searchParams.set('t', endpoint.token);
  for (const [name, value] of Object.entries(query)) url.searchParams.set(name, value);
  return url.toString();
}

function exportPath(exportId: ExportId, rest: string): string {
  return `/media/exports/${encodeURIComponent(exportId)}/${rest}`;
}

/**
 * Where an export reads assets' files: the core serves them from the export's snapshot
 * (`/media/exports/{id}/assets/{asset}/{original|mezzanine}`), never the proxy (ADR-0038). URLs
 * carry the session token: never log them.
 */
export function exportMediaLocator(endpoint: CoreEndpoint, exportId: ExportId): MediaLocator {
  return (assetId: AssetId, variant) =>
    variant === 'proxy'
      ? null
      : coreUrl(endpoint, exportPath(exportId, `assets/${encodeURIComponent(assetId)}/${variant}`));
}

/** Why the core refused an upload, without its URL (it carries the session token). */
async function refusal(response: Response, what: string): Promise<Error> {
  if (response.status === 409) return new ExportStoppedError();
  const detail = (await response.text().catch(() => '')).trim();
  const reason = detail === '' ? `status ${String(response.status)}` : detail;
  return new Error(`the core refused the export's ${what}: ${reason}`);
}

/** An {@link ExportSink} uploading to the core (`PUT /media/exports/{id}/…`, `export_routes.rs`). */
export function httpExportSink(
  endpoint: CoreEndpoint,
  exportId: ExportId,
  send: typeof fetch = fetch,
): ExportSink {
  const put = async (url: string, body: BodyInit, what: string, type: string) => {
    const response = await send(url, {
      method: 'PUT',
      headers: { 'Content-Type': type },
      body,
    });
    if (!response.ok) throw await refusal(response, what);
  };
  return {
    write: (stream, position, bytes) =>
      put(
        coreUrl(endpoint, exportPath(exportId, stream), { position: String(position) }),
        bytes as Uint8Array<ArrayBuffer>,
        stream === 'data' ? 'file' : 'sound',
        'application/octet-stream',
      ),
    frame: (rgba) =>
      put(
        coreUrl(endpoint, exportPath(exportId, 'frames')),
        rgba as Uint8Array<ArrayBuffer>,
        'frame',
        'application/octet-stream',
      ),
    thumbnail: (jpeg) =>
      put(coreUrl(endpoint, exportPath(exportId, 'thumbnail')), jpeg, 'thumbnail', 'image/jpeg'),
  };
}
