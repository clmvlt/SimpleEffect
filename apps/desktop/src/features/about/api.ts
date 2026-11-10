import type { ClientDiagnostics, CoreClient, DiagnosticsSummary } from '@se/protocol';

/** What the about feature needs from the core client. */
export type AboutClient = Pick<CoreClient, 'send'>;

/**
 * Writes a diagnostics archive at `path` (absolute, `.zip`): the core adds its own state and its
 * recent logs, masks personal data and resolves with what it wrote.
 */
export async function exportDiagnostics(
  client: AboutClient,
  path: string,
  diagnostics: ClientDiagnostics,
): Promise<DiagnosticsSummary> {
  const result = await client.send({ kind: 'exportDiagnostics', path, client: diagnostics });
  return { path: result.path, files: result.files, bytes: result.bytes };
}
