import type { AiStatus } from '@se/protocol';

/** A tone of the wizard's check rows. */
export type CheckTone = 'success' | 'warning' | 'danger' | 'neutral';

/** What the « Assistant IA » step shows for an {@link AiStatus}. */
export interface AiStepState {
  readonly installation: CheckTone;
  /** i18n key under `onboarding.ai.installation.`. */
  readonly installationKey: 'checking' | 'ready' | 'notInstalled' | 'unusable';
  readonly login: CheckTone;
  /** i18n key under `onboarding.ai.login.`. */
  readonly loginKey: 'checking' | 'ready' | 'loggedOut' | 'unknown' | 'needsCli';
}

/** The installation and login rows of the AI step (`null`: the core has not answered yet). */
export function aiStepState(status: AiStatus | null): AiStepState {
  if (status === null) {
    return {
      installation: 'neutral',
      installationKey: 'checking',
      login: 'neutral',
      loginKey: 'checking',
    };
  }
  const kind = status.installation.kind;
  if (kind !== 'installed') {
    return {
      installation: 'danger',
      installationKey: kind,
      login: 'neutral',
      loginKey: 'needsCli',
    };
  }
  const auth = status.auth.kind;
  return {
    installation: 'success',
    installationKey: 'ready',
    login: auth === 'loggedIn' ? 'success' : auth === 'loggedOut' ? 'danger' : 'warning',
    loginKey: auth === 'loggedIn' ? 'ready' : auth,
  };
}
