import { StatusDot } from '@se/ui';
import { useTranslation } from 'react-i18next';

import { useConnectionStatus } from '../../shared/core';
import { connectionIndicator } from './connection-indicator';

/**
 * The 24 px bar at the bottom of the window: core connection (announced politely when it
 * changes), AI availability and app version.
 * TODO(prompt-17): show the configured AI provider and its state.
 */
export function StatusBar() {
  const { t } = useTranslation();
  const indicator = connectionIndicator(useConnectionStatus());
  return (
    <footer
      aria-label={t('statusBar.label')}
      className="flex h-6 shrink-0 items-center gap-4 border-t border-border bg-panel px-2 text-2xs text-text-muted select-none"
    >
      <p role="status" className="flex items-center gap-1.5">
        <StatusDot tone={indicator.tone} pulse={indicator.pulse} size="xs" />
        {t(indicator.labelKey)}
      </p>
      <p className="flex items-center gap-1.5">
        <StatusDot tone="neutral" size="xs" />
        {t('statusBar.ai.notConfigured')}
      </p>
      <p className="ml-auto tabular-nums">{t('statusBar.version', { version: __APP_VERSION__ })}</p>
    </footer>
  );
}
