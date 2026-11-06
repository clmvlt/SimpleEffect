import type { CoreClient } from '@se/protocol';
import { Toaster, TooltipProvider } from '@se/ui';
import type { i18n } from 'i18next';
import type { ReactNode } from 'react';
import { I18nextProvider, useTranslation } from 'react-i18next';

import { CoreClientProvider } from '../shared/core';

/** Props of {@link AppProviders}. */
export interface AppProvidersProps {
  readonly coreClient: CoreClient;
  readonly i18n: i18n;
  readonly children: ReactNode;
}

/** App-wide contexts: translations, core connection, tooltip timing, and the toast area. */
export function AppProviders({ coreClient, i18n, children }: AppProvidersProps) {
  return (
    <I18nextProvider i18n={i18n}>
      <CoreClientProvider client={coreClient}>
        <TooltipProvider>
          {children}
          <AppToaster />
        </TooltipProvider>
      </CoreClientProvider>
    </I18nextProvider>
  );
}

/** The toast area, named in the interface language. */
function AppToaster() {
  const { t } = useTranslation();
  return <Toaster label={t('common.notifications')} closeLabel={t('common.close')} />;
}
