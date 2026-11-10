import { Button } from '@se/ui';
import { useTranslation } from 'react-i18next';

import { useDesktopBridge } from '../../../shared/desktop';
import { aiTextValues, useAiRefreshing, useAiStatus, useRefreshAiStatus } from '../../ai';
import { aiStepState } from '../services/ai-step';
import { CheckRow } from './CheckRow';

/** Props of {@link AiStep}. */
export interface AiStepProps {
  /** Opens « Réglages → IA » (a custom CLI path). */
  readonly onOpenAiSettings: () => void;
}

/**
 * « Assistant IA »: whether the AI CLI is installed and logged in, with what to do otherwise
 * (installation guide, the login command to type) and « Vérifier à nouveau ».
 */
export function AiStep({ onOpenAiSettings }: AiStepProps) {
  const { t } = useTranslation();
  const bridge = useDesktopBridge();
  const status = useAiStatus();
  const refreshing = useAiRefreshing();
  const refresh = useRefreshAiStatus();
  const values = aiTextValues(status, t('statusBar.ai.fallbackName'));
  const state = aiStepState(status);
  return (
    <div className="flex flex-col gap-3">
      <p className="text-sm text-text-muted">{t('onboarding.ai.intro', values)}</p>
      <ul className="flex flex-col">
        <CheckRow tone={state.installation} label={t('onboarding.ai.installed', values)}>
          {t(`onboarding.ai.installation.${state.installationKey}`, values)}
        </CheckRow>
        <CheckRow tone={state.login} label={t('onboarding.ai.loggedIn', values)}>
          {t(`onboarding.ai.login.${state.loginKey}`, values)}
        </CheckRow>
      </ul>
      <div className="flex flex-wrap gap-2">
        <Button size="sm" onClick={refresh} disabled={refreshing}>
          {refreshing ? t('onboarding.ai.checking') : t('onboarding.ai.check')}
        </Button>
        {status !== null && state.installationKey !== 'ready' && (
          <Button
            size="sm"
            variant="outline"
            onClick={() => {
              void bridge.openExternal(status.provider.helpUrl);
            }}
          >
            {t('onboarding.ai.howToInstall', values)}
          </Button>
        )}
        <Button size="sm" variant="ghost" onClick={onOpenAiSettings}>
          {t('onboarding.ai.settings')}
        </Button>
      </div>
      <p className="text-2xs text-text-muted">{t('onboarding.ai.later')}</p>
    </div>
  );
}
