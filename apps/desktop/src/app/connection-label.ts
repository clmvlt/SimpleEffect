import type { ConnectionState, CoreInfo } from '@se/protocol';

/** One labelled line of the core information list. */
export interface InfoRow {
  readonly label: string;
  readonly value: string;
}

/**
 * French label of the core connection state, with the core version once known.
 * TODO(prompt-04): move these strings to i18n keys in `i18n/locales/fr.json`.
 */
export function describeConnection(state: ConnectionState, info: CoreInfo | null): string {
  switch (state) {
    case 'connecting':
      return 'Connexion au cœur…';
    case 'open':
      return info === null ? 'Cœur connecté' : `Cœur connecté — se-core ${info.version}`;
    case 'reconnecting':
      return 'Reconnexion…';
    case 'closed':
      return 'Connexion au cœur fermée';
  }
}

/**
 * Lines describing the connected core.
 * TODO(prompt-04): move these strings to i18n keys in `i18n/locales/fr.json`.
 */
export function describeCoreInfo(info: CoreInfo): InfoRow[] {
  return [
    { label: 'Plateforme', value: `${info.os} (${info.arch})` },
    { label: 'Données', value: info.appDataDir },
    { label: 'Journaux', value: info.logsDir },
  ];
}
