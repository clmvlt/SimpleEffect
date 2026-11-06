import { EmptyState, Panel, PanelBody, PanelHeader, PanelTitle } from '@se/ui';
import { MonitorPlay } from '@se/ui/icons';
import { useTranslation } from 'react-i18next';

/**
 * The program viewer, empty until sequences can be rendered.
 * TODO(prompt-12): render the open sequence with @se/engine and add the transport bar.
 */
export function ViewerPanel() {
  const { t } = useTranslation();
  return (
    <Panel>
      <PanelHeader>
        <PanelTitle>{t('editor.viewer.title')}</PanelTitle>
      </PanelHeader>
      <PanelBody scrollable={false} className="bg-bg">
        <EmptyState
          icon={MonitorPlay}
          title={t('editor.viewer.empty.title')}
          description={t('editor.viewer.empty.description')}
        />
      </PanelBody>
    </Panel>
  );
}
