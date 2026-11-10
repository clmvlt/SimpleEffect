import type { AiStatus } from '@se/protocol';

import type { RuntimeInfo } from '../../../shared/desktop';

/** One line of the versions table of « À propos ». */
export interface VersionRow {
  /** Stable id, also the i18n key under `about.versions.` (except `ai`, labelled by name). */
  readonly id: 'app' | 'core' | 'engine' | 'electron' | 'chromium' | 'ai';
  /** The version, or `null` while unknown (core disconnected, CLI not found…). */
  readonly version: string | null;
}

/** What the versions table is made of. */
export interface VersionSources {
  readonly app: string;
  readonly engine: string;
  /** The connected core's version, `null` while disconnected. */
  readonly core: string | null;
  readonly runtime: RuntimeInfo | null;
  readonly ai: AiStatus | null;
}

/** The versions table: the app, se-core, the engine, Electron, Chromium and the AI CLI. */
export function versionRows(sources: VersionSources): VersionRow[] {
  const installation = sources.ai?.installation;
  return [
    { id: 'app', version: sources.app },
    { id: 'core', version: sources.core },
    { id: 'engine', version: sources.engine },
    { id: 'electron', version: sources.runtime?.electron ?? null },
    { id: 'chromium', version: sources.runtime?.chrome ?? null },
    { id: 'ai', version: installation?.kind === 'installed' ? installation.version : null },
  ];
}

/** The versions as plain text, for « Copier » (bug reports). */
export function versionsText(
  rows: readonly VersionRow[],
  labels: (row: VersionRow) => string,
): string {
  return rows.map((row) => `${labels(row)}: ${row.version ?? '—'}`).join('\n');
}
