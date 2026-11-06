import type { CoreStatus } from '../shared/core-connection';

/**
 * French label of a core connection state.
 * TODO(prompt-04): move these strings to i18n keys in `i18n/locales/fr.json`.
 */
export function describeCoreStatus(status: CoreStatus): string {
  switch (status.kind) {
    case 'connecting':
      return 'Connexion au cœur…';
    case 'connected':
      return `Cœur connecté — se-core ${status.version}`;
    case 'reconnecting':
      return 'Reconnexion…';
  }
}
