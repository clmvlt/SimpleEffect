import {
  Button,
  cn,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@se/ui';
import { useTranslation } from 'react-i18next';

import type { KeyValueStorage } from '../../../shared/storage';
import { openSettings } from '../../settings';
import { useFirstRun } from '../hooks/use-first-run';
import {
  ONBOARDING_STEPS,
  markOnboardingDone,
  nextStep,
  previousStep,
  type OnboardingStep,
} from '../services/onboarding-steps';
import { updateOnboarding, useOnboarding } from '../store/onboarding-store';
import { AiStep } from './AiStep';
import { MediaStep } from './MediaStep';
import { ModelsStep } from './ModelsStep';
import { ProjectsStep } from './ProjectsStep';
import { WelcomeStep } from './WelcomeStep';

/** Props of {@link OnboardingDialog}. */
export interface OnboardingDialogProps {
  /** Where « finished or skipped » is remembered (`uiPreferencesStorage()` in the app). */
  readonly preferences: KeyValueStorage;
}

/**
 * The first-run wizard (prompt 30): Bienvenue → Assistant IA → Binaires & GPU → Modèles
 * (facultatif) → Dossier des projets. It opens by itself on the first launch of this machine and
 * again from « À propos »; closing it at any step counts as done.
 */
export function OnboardingDialog({ preferences }: OnboardingDialogProps) {
  const { t } = useTranslation();
  const { open, step } = useOnboarding();
  useFirstRun(preferences);
  const close = () => {
    markOnboardingDone(preferences, new Date());
    updateOnboarding({ open: false });
  };
  const next = nextStep(step);
  const previous = previousStep(step);
  const index = ONBOARDING_STEPS.indexOf(step);
  return (
    <Dialog
      open={open}
      onOpenChange={(value) => {
        if (!value) close();
      }}
    >
      <DialogContent size="lg" closeLabel={t('common.close')}>
        <DialogHeader>
          <DialogTitle>{t(`onboarding.${step}.title`)}</DialogTitle>
          <DialogDescription>
            {t('onboarding.progress', { step: index + 1, count: ONBOARDING_STEPS.length })}
          </DialogDescription>
        </DialogHeader>
        <ol aria-label={t('onboarding.stepsLabel')} className="flex gap-1">
          {ONBOARDING_STEPS.map((item, position) => (
            <li
              key={item}
              aria-current={item === step ? 'step' : undefined}
              title={t(`onboarding.${item}.title`)}
              className={cn(
                'h-1 flex-1 rounded-full',
                position <= index ? 'bg-accent' : 'bg-control',
              )}
            />
          ))}
        </ol>
        <div className="min-h-48 py-2">
          <StepContent
            step={step}
            onOpenAiSettings={() => {
              close();
              openSettings('ai');
            }}
          />
        </div>
        <div className="flex items-center justify-between gap-2">
          <Button size="sm" variant="ghost" onClick={close}>
            {t('onboarding.skip')}
          </Button>
          <div className="flex gap-2">
            {previous !== null && (
              <Button
                size="sm"
                variant="outline"
                onClick={() => {
                  updateOnboarding({ step: previous });
                }}
              >
                {t('onboarding.previous')}
              </Button>
            )}
            <Button
              size="sm"
              variant="primary"
              onClick={() => {
                if (next === null) close();
                else updateOnboarding({ step: next });
              }}
            >
              {next === null ? t('onboarding.finish') : t('onboarding.next')}
            </Button>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}

function StepContent({
  step,
  onOpenAiSettings,
}: {
  readonly step: OnboardingStep;
  readonly onOpenAiSettings: () => void;
}) {
  switch (step) {
    case 'welcome':
      return <WelcomeStep />;
    case 'ai':
      return <AiStep onOpenAiSettings={onOpenAiSettings} />;
    case 'media':
      return <MediaStep active />;
    case 'models':
      return <ModelsStep active />;
    case 'projects':
      return <ProjectsStep active />;
  }
}
