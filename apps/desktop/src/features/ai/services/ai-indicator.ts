import type { AiStatus } from '@se/protocol';
import type { StatusDotProps } from '@se/ui';

/** Whether AI tasks can run, as the UI shows it. */
export type AiAvailability = 'checking' | 'ready' | 'loggedOut' | 'notInstalled' | 'unknown';

/** How the status bar shows the AI provider. */
export interface AiIndicator {
  readonly availability: AiAvailability;
  readonly tone: NonNullable<StatusDotProps['tone']>;
  readonly labelKey: `statusBar.ai.${AiAvailability}`;
  readonly hintKey: `statusBar.ai.hint.${AiAvailability}`;
}

const TONES: Record<AiAvailability, AiIndicator['tone']> = {
  checking: 'neutral',
  ready: 'success',
  loggedOut: 'warning',
  notInstalled: 'neutral',
  unknown: 'warning',
};

/** Whether AI tasks can run with this status (`null`: not known yet). */
export function aiAvailability(status: AiStatus | null): AiAvailability {
  if (status === null) return 'checking';
  if (status.installation.kind !== 'installed') return 'notInstalled';
  switch (status.auth.kind) {
    case 'loggedIn':
      return 'ready';
    case 'loggedOut':
      return 'loggedOut';
    case 'unknown':
      return 'unknown';
  }
}

/** Dot colour, label and hint of a status: green logged in, orange logged out, grey missing. */
export function aiIndicator(status: AiStatus | null): AiIndicator {
  const availability = aiAvailability(status);
  return {
    availability,
    tone: TONES[availability],
    labelKey: `statusBar.ai.${availability}`,
    hintKey: `statusBar.ai.hint.${availability}`,
  };
}

/** Whether new tasks may be submitted (an unknown login state is let through: runs will tell). */
export function canSubmitTasks(status: AiStatus | null): boolean {
  const availability = aiAvailability(status);
  return availability === 'ready' || availability === 'unknown';
}
