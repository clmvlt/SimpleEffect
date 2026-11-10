import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from '@se/ui';
import { useTranslation } from 'react-i18next';

import { openOnboarding } from '../../onboarding';
import { PerformanceSection } from '../../performance';
import { aboutTabOf, updateAboutDialog, useAboutDialog } from '../store/about-dialog-store';
import { ApplicationTab } from './ApplicationTab';
import { LicencesTab } from './LicencesTab';

/**
 * « À propos de SimpleEffect »: versions and diagnostics, licences, and the in-app performance
 * measure (prompt 29, moved here from « Réglages » by prompt 30). Mounted once by the app layout;
 * opened by {@link openAbout} (the status bar's version).
 */
export function AboutDialog() {
  const { t } = useTranslation();
  const { open, tab } = useAboutDialog();
  return (
    <Dialog
      open={open}
      onOpenChange={(next) => {
        updateAboutDialog({ open: next });
      }}
    >
      <DialogContent size="lg" closeLabel={t('common.close')}>
        <DialogHeader>
          <DialogTitle>{t('about.title')}</DialogTitle>
          <DialogDescription>{t('about.description')}</DialogDescription>
        </DialogHeader>
        <Tabs
          value={tab}
          onValueChange={(value) => {
            updateAboutDialog({ tab: aboutTabOf(value) });
          }}
        >
          <TabsList variant="underline">
            <TabsTrigger value="application">{t('about.tabs.application')}</TabsTrigger>
            <TabsTrigger value="licences">{t('about.tabs.licences')}</TabsTrigger>
            <TabsTrigger value="performance">{t('about.tabs.performance')}</TabsTrigger>
          </TabsList>
          <TabsContent value="application" className="max-h-[65vh] overflow-auto pt-3">
            <ApplicationTab
              onOpenWelcome={() => {
                updateAboutDialog({ open: false });
                openOnboarding();
              }}
            />
          </TabsContent>
          <TabsContent value="licences" className="pt-3">
            <LicencesTab active={open && tab === 'licences'} />
          </TabsContent>
          <TabsContent value="performance" className="max-h-[65vh] overflow-auto pt-3">
            <PerformanceSection />
          </TabsContent>
        </Tabs>
      </DialogContent>
    </Dialog>
  );
}
