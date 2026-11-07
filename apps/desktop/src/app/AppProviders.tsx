import type { CoreClient } from '@se/protocol';
import { Toaster, TooltipProvider } from '@se/ui';
import type { i18n } from 'i18next';
import type { ReactNode } from 'react';
import { I18nextProvider, useTranslation } from 'react-i18next';

import { CoreClientProvider } from '../shared/core';
import { DesktopBridgeProvider, type SimpleEffectBridge } from '../shared/desktop';
import { ProjectStoreProvider, type ProjectStore, type ProjectSync } from '../shared/project-store';
import { ShortcutProvider, type ShortcutRegistry } from '../shared/shortcuts';

/** Props of {@link AppProviders}: the app-wide services created by the composition root. */
export interface AppProvidersProps {
  readonly coreClient: CoreClient;
  readonly i18n: i18n;
  readonly bridge: SimpleEffectBridge;
  readonly projectStore: ProjectStore;
  readonly projectSync: ProjectSync;
  readonly shortcuts: ShortcutRegistry;
  readonly children: ReactNode;
}

/**
 * App-wide contexts: translations, core connection, Electron bridge, open-project mirror,
 * keyboard shortcuts, tooltip timing, and the toast area.
 */
export function AppProviders({
  coreClient,
  i18n,
  bridge,
  projectStore,
  projectSync,
  shortcuts,
  children,
}: AppProvidersProps) {
  return (
    <I18nextProvider i18n={i18n}>
      <CoreClientProvider client={coreClient}>
        <DesktopBridgeProvider bridge={bridge}>
          <ProjectStoreProvider store={projectStore} sync={projectSync}>
            <ShortcutProvider registry={shortcuts}>
              <TooltipProvider>
                {children}
                <AppToaster />
              </TooltipProvider>
            </ShortcutProvider>
          </ProjectStoreProvider>
        </DesktopBridgeProvider>
      </CoreClientProvider>
    </I18nextProvider>
  );
}

/** The toast area, named in the interface language. */
function AppToaster() {
  const { t } = useTranslation();
  return <Toaster label={t('common.notifications')} closeLabel={t('common.close')} />;
}
