import type { CoreEndpoint, ExportId } from '@se/protocol';

import type { CatalogFace } from '../fonts/font-registry';
import type { RenderScene } from '../preview/frame-renderer';
import type { ExportRenderJob } from './render-export';

/** Everything the export Worker needs to render an export on its own. */
export interface ExportWorkerJob extends ExportRenderJob {
  readonly exportId: ExportId;
  /** The core: upload routes and the export's media (`/media/exports/{id}/…`). */
  readonly endpoint: CoreEndpoint;
  /** The snapshot's sequence and assets, as the preview draws them. */
  readonly scene: RenderScene;
  /** The font catalogue's faces (texts and motion templates). */
  readonly fonts: readonly CatalogFace[];
}

/** Messages from the page to the export Worker. */
export type ToExportWorker =
  | { readonly kind: 'start'; readonly job: ExportWorkerJob }
  /** Mixed sound of a block the Worker asked for (buffers transferred). */
  | { readonly kind: 'sound'; readonly index: number; readonly channels: Float32Array[] }
  /** The page could not mix a block. */
  | { readonly kind: 'soundFailed'; readonly index: number; readonly message: string };

/** Messages from the export Worker to the page. */
export type FromExportWorker =
  /** Mix block `index` (the page has `OfflineAudioContext`, the Worker does not). */
  | { readonly kind: 'needSound'; readonly index: number }
  | { readonly kind: 'progress'; readonly done: number; readonly total: number }
  | {
      readonly kind: 'finished';
      /** Assets, font families and templates that could not be drawn as asked. */
      readonly missingAssets: readonly string[];
      readonly missingFonts: readonly string[];
      readonly failedTemplates: readonly string[];
    }
  | { readonly kind: 'failed'; readonly message: string; readonly stopped: boolean };
