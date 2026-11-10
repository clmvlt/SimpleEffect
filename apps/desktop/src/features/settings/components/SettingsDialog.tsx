import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  Separator,
  Spinner,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from '@se/ui';
import { useTranslation } from 'react-i18next';

import { useAiSettings } from '../hooks/use-ai-settings';
import { useMediaSettings } from '../hooks/use-media-settings';
import { useProjectCache } from '../hooks/use-project-cache';
import { useTranscriptionSettings } from '../hooks/use-transcription-settings';
import { BrandKitsSection } from '../../brand-kits';
import { PerformanceSection } from '../../performance';
import { settingsSectionOf, type SettingsSection } from '../store/settings-dialog-store';
import { AiSettingsSection } from './AiSettingsSection';
import { MediaSettingsSection } from './MediaSettingsSection';
import { ProjectCacheSection } from './ProjectCacheSection';
import { ToolsSection } from './ToolsSection';
import { TranscriptionSettingsSection } from './TranscriptionSettingsSection';

/** Props of {@link SettingsDialog}. */
export interface SettingsDialogProps {
  readonly open: boolean;
  readonly onOpenChange: (open: boolean) => void;
  /** The section shown. */
  readonly section: SettingsSection;
  readonly onSectionChange: (section: SettingsSection) => void;
  /** The open project (its cache section), or `null` on the home screen. */
  readonly projectId: string | null;
}

/**
 * « Réglages »: app preferences by section — Médias (derived copies, ffmpeg tools, the open
 * project's cache), IA (the AI CLI, its login, task defaults) and Transcription (whisper.cpp, GPU,
 * models, options; loaded only while shown) and Brand kits (prompt 25: the library's kits, each
 * saved by its own button). Other changes are saved as they are made.
 */
export function SettingsDialog({
  open,
  onOpenChange,
  projectId,
  section,
  onSectionChange,
}: SettingsDialogProps) {
  const { t } = useTranslation();
  const media = useMediaSettings(open);
  const ai = useAiSettings(open);
  const cache = useProjectCache(projectId, open);
  const transcription = useTranscriptionSettings(open && section === 'transcription');
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent size={section === 'brandKits' ? 'xl' : 'lg'} closeLabel={t('common.close')}>
        <DialogHeader>
          <DialogTitle>{t('settings.title')}</DialogTitle>
          <DialogDescription>{t('settings.description')}</DialogDescription>
        </DialogHeader>
        <Tabs
          value={section}
          onValueChange={(value) => {
            onSectionChange(settingsSectionOf(value));
          }}
        >
          <TabsList variant="underline">
            <TabsTrigger value="media">{t('settings.sections.media')}</TabsTrigger>
            <TabsTrigger value="ai">{t('settings.sections.ai')}</TabsTrigger>
            <TabsTrigger value="transcription">{t('settings.sections.transcription')}</TabsTrigger>
            <TabsTrigger value="brandKits">{t('settings.sections.brandKits')}</TabsTrigger>
            <TabsTrigger value="performance">{t('settings.sections.performance')}</TabsTrigger>
          </TabsList>
          <TabsContent
            value="media"
            className="flex max-h-[60vh] flex-col gap-4 overflow-auto pt-3"
          >
            {media.settings === null ? (
              <div className="flex justify-center py-8">
                <Spinner size="lg" label={t('settings.media.checking')} />
              </div>
            ) : (
              <>
                <MediaSettingsSection settings={media.settings} onSave={media.save} />
                <Separator />
                <ToolsSection settings={media.settings} tools={media.tools} onSave={media.save} />
              </>
            )}
            <Separator />
            <ProjectCacheSection projectId={projectId} state={cache} />
          </TabsContent>
          <TabsContent value="ai" className="flex max-h-[60vh] flex-col gap-4 overflow-auto pt-3">
            {ai.settings === null ? (
              <div className="flex justify-center py-8">
                <Spinner size="lg" label={t('settings.media.checking')} />
              </div>
            ) : (
              <AiSettingsSection settings={ai.settings} onSave={ai.save} />
            )}
          </TabsContent>
          <TabsContent
            value="transcription"
            className="flex max-h-[60vh] flex-col gap-4 overflow-auto pt-3"
          >
            {transcription.status === null ? (
              <div className="flex justify-center py-8">
                <Spinner size="lg" label={t('settings.media.checking')} />
              </div>
            ) : (
              <TranscriptionSettingsSection status={transcription.status} actions={transcription} />
            )}
          </TabsContent>
          <TabsContent
            value="brandKits"
            className="flex max-h-[70vh] flex-col gap-4 overflow-auto pt-3"
          >
            {section === 'brandKits' && <BrandKitsSection projectId={projectId} />}
          </TabsContent>
          <TabsContent
            value="performance"
            className="flex max-h-[60vh] flex-col gap-4 overflow-auto pt-3"
          >
            <PerformanceSection />
          </TabsContent>
        </Tabs>
      </DialogContent>
    </Dialog>
  );
}
