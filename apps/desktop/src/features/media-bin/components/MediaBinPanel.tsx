import { EmptyState, Panel, PanelBody, PanelHeader, PanelTitle } from '@se/ui';
import { Film } from '@se/ui/icons';
import { useTranslation } from 'react-i18next';

/**
 * The project's media bin, empty until media can be imported.
 * TODO(prompt-08): list imported media with thumbnails, import actions and drag to the timeline.
 */
export function MediaBinPanel() {
  const { t } = useTranslation();
  return (
    <Panel>
      <PanelHeader>
        <PanelTitle>{t('editor.media.title')}</PanelTitle>
      </PanelHeader>
      <PanelBody scrollable={false}>
        <EmptyState
          icon={Film}
          title={t('editor.media.empty.title')}
          description={t('editor.media.empty.description')}
        />
      </PanelBody>
    </Panel>
  );
}
