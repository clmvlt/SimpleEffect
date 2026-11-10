import type {
  CoreClient,
  ExportId,
  ExportInfo,
  ExportPreferences,
  StartExportRequest,
} from '@se/protocol';

/** Protocol calls of the export feature. */
export interface ExportsApi {
  /** Queues an export (the core snapshots the sequence at once). */
  readonly start: (request: StartExportRequest) => Promise<ExportInfo>;
  /** Cancels a queued or running export (its partial files are removed). */
  readonly cancel: (exportId: ExportId) => Promise<void>;
  /** Exports of this session, oldest first. */
  readonly list: () => Promise<readonly ExportInfo[]>;
  /** Forgets finished, failed and cancelled exports. */
  readonly clearFinished: () => Promise<void>;
  /** What the dialog remembers: folder, file name template, saved presets. */
  readonly preferences: () => Promise<ExportPreferences>;
  readonly savePreferences: (preferences: ExportPreferences) => Promise<void>;
  /** Reports of the renderer drawing an export. */
  readonly reportProgress: (exportId: ExportId, done: number, total: number) => Promise<void>;
  readonly finishRender: (exportId: ExportId) => Promise<void>;
  readonly failRender: (exportId: ExportId, message: string) => Promise<void>;
}

/** The export calls over `client`. */
export function exportsApi(client: Pick<CoreClient, 'send'>): ExportsApi {
  return {
    start: async (request) => (await client.send({ kind: 'startExport', ...request })).export,
    cancel: async (exportId) => {
      await client.send({ kind: 'cancelExport', exportId });
    },
    list: async () => (await client.send({ kind: 'listExports' })).exports,
    clearFinished: async () => {
      await client.send({ kind: 'clearFinishedExports' });
    },
    preferences: async () => {
      const { directory, fileNameTemplate, presets } = await client.send({
        kind: 'getExportPreferences',
      });
      return { directory, fileNameTemplate, presets };
    },
    savePreferences: async (preferences) => {
      await client.send({ kind: 'updateExportPreferences', ...preferences });
    },
    reportProgress: async (exportId, done, total) => {
      await client.send({ kind: 'reportExportProgress', exportId, done, total });
    },
    finishRender: async (exportId) => {
      await client.send({ kind: 'finishExportRender', exportId });
    },
    failRender: async (exportId, message) => {
      await client.send({ kind: 'failExportRender', exportId, message });
    },
  };
}
