import type { AiStatus } from '@se/protocol';

/** Interpolated names of the AI status texts (`statusBar.ai.*`, `settings.ai.*`). */
export type AiTextKey = 'name' | 'command' | 'login' | 'version' | 'account' | 'reason';

/** Interpolation values of the AI status texts (a record, as i18next options must be). */
export type AiTextValues = Readonly<Record<AiTextKey, string>>;

/**
 * The values AI status texts interpolate, from what the provider says about itself (the UI never
 * names a CLI: golden rule 3). `fallbackName` stands for the provider while it is unknown.
 */
export function aiTextValues(status: AiStatus | null, fallbackName: string): AiTextValues {
  const provider = status?.provider;
  const installation = status?.installation;
  const auth = status?.auth;
  return {
    name: provider?.name ?? fallbackName,
    command: provider?.command ?? '',
    login: provider?.loginCommand ?? '',
    version: installation?.kind === 'installed' ? installation.version : '',
    account: auth?.kind === 'loggedIn' ? (auth.accountKind ?? '') : '',
    reason:
      auth?.kind === 'unknown'
        ? auth.reason
        : installation !== undefined && installation.kind !== 'installed'
          ? installation.message
          : '',
  };
}
