import type { KeyValueStorage } from '../../../shared/storage';

/** The steps of the first-run wizard, in order. */
export const ONBOARDING_STEPS = ['welcome', 'ai', 'media', 'models', 'projects'] as const;

/** A step of the first-run wizard. */
export type OnboardingStep = (typeof ONBOARDING_STEPS)[number];

/**
 * Preference key (in `uiPreferencesStorage()`) set once the wizard was finished or skipped. The
 * version suffix lets a later wizard with new steps show once again.
 */
export const ONBOARDING_DONE_KEY = 'onboarding.v1.done';

/** The step after `step`, or `null` after the last one. */
export function nextStep(step: OnboardingStep): OnboardingStep | null {
  return ONBOARDING_STEPS[ONBOARDING_STEPS.indexOf(step) + 1] ?? null;
}

/** The step before `step`, or `null` before the first one. */
export function previousStep(step: OnboardingStep): OnboardingStep | null {
  const index = ONBOARDING_STEPS.indexOf(step);
  return index <= 0 ? null : (ONBOARDING_STEPS[index - 1] ?? null);
}

/** Whether the wizard should open by itself (first launch on this machine). */
export function isFirstRun(storage: KeyValueStorage): boolean {
  return storage.getItem(ONBOARDING_DONE_KEY) === null;
}

/** Remembers the wizard was finished or skipped. */
export function markOnboardingDone(storage: KeyValueStorage, now: Date): void {
  storage.setItem(ONBOARDING_DONE_KEY, now.toISOString());
}
