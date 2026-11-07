import { StatusDot, Tooltip } from '@se/ui';
import { useTranslation } from 'react-i18next';

import { useAiStatus } from '../hooks/use-ai-status';
import { aiIndicator } from '../services/ai-indicator';
import { aiTextValues } from '../services/ai-texts';

/** Props of {@link AiStatusIndicator}. */
export interface AiStatusIndicatorProps {
  /** Opens the AI settings (where the CLI, the login and the help are). */
  readonly onOpenSettings: () => void;
}

/**
 * The AI provider in the status bar: green when logged in, orange when logged out (the tooltip
 * says how to log in from a terminal), grey when not installed. A click opens the AI settings.
 */
export function AiStatusIndicator({ onOpenSettings }: AiStatusIndicatorProps) {
  const { t } = useTranslation();
  const status = useAiStatus();
  const indicator = aiIndicator(status);
  const values = aiTextValues(status, t('statusBar.ai.fallbackName'));
  return (
    <Tooltip content={t(indicator.hintKey, values)} side="top" align="start">
      <button
        type="button"
        data-availability={indicator.availability}
        className="flex h-5 items-center gap-1.5 rounded-sm px-1 hover:bg-hover hover:text-text focus-visible:outline-1 focus-visible:outline-accent"
        aria-label={`${t(indicator.labelKey, values)} — ${t('statusBar.ai.open')}`}
        onClick={onOpenSettings}
      >
        <StatusDot tone={indicator.tone} pulse={indicator.availability === 'checking'} size="xs" />
        {t(indicator.labelKey, values)}
      </button>
    </Tooltip>
  );
}
