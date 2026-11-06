import { EmptyState, Panel, PanelBody, PanelHeader, PanelTitle } from '@se/ui';
import { GalleryHorizontal } from '@se/ui/icons';
import { useTranslation } from 'react-i18next';

/**
 * The timeline of the open sequence, empty until sequences exist.
 * TODO(prompt-10): tracks, items, ruler and playhead.
 */
export function TimelinePanel() {
  const { t } = useTranslation();
  return (
    <Panel>
      <PanelHeader>
        <PanelTitle>{t('editor.timeline.title')}</PanelTitle>
      </PanelHeader>
      <PanelBody scrollable={false}>
        <EmptyState
          icon={GalleryHorizontal}
          title={t('editor.timeline.empty.title')}
          description={t('editor.timeline.empty.description')}
        />
      </PanelBody>
    </Panel>
  );
}
