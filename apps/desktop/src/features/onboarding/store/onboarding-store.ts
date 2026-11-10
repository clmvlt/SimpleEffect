import { useStore } from 'zustand';
import { createStore } from 'zustand/vanilla';

import type { OnboardingStep } from '../services/onboarding-steps';

/** Whether the first-run wizard is open and on which step. */
export interface OnboardingState {
  readonly open: boolean;
  readonly step: OnboardingStep;
}

/** The wizard's state, so « À propos » can open it again. */
export const onboardingStore = createStore<OnboardingState>()(() => ({
  open: false,
  step: 'welcome',
}));

/** Opens the wizard on its first step. */
export function openOnboarding(): void {
  onboardingStore.setState({ open: true, step: 'welcome' });
}

/** Closes the wizard or shows another step. */
export function updateOnboarding(changes: Partial<OnboardingState>): void {
  onboardingStore.setState(changes);
}

/** The wizard's state, re-rendering when it changes. */
export function useOnboarding(): OnboardingState {
  return useStore(onboardingStore);
}
