import { EmptyState, Panel, PanelBody, PanelHeader, PanelTitle } from '@se/ui';
import { MonitorPlay } from '@se/ui/icons';
import { useTranslation } from 'react-i18next';

import { ShortcutRegion } from '../../../shared/shortcuts';
import { useActiveSequence } from '../../timeline';
import { useViewerPreferences } from '../hooks/use-viewer-preferences';
import { createPreviewEngine, type PreviewEngineFactory } from '../services/preview-engine-factory';
import { ViewerFooter } from './ViewerFooter';
import { ViewerStage } from './ViewerStage';
import { ViewerToolbar } from './ViewerToolbar';

/** Props of {@link ViewerPanel}. */
export interface ViewerPanelProps {
  /** Starts the engine (tests inject a fake; the app uses `@se/engine/preview`). */
  readonly createEngine?: PreviewEngineFactory | undefined;
}

/**
 * The program viewer: the exact frame of the active sequence at the playhead, drawn by
 * `@se/engine` (the code path export uses). Time shortcuts work here too (context `viewer`).
 * TODO(prompt-13): playback, audio and the transport bar.
 */
export function ViewerPanel({ createEngine = createPreviewEngine }: ViewerPanelProps) {
  const { t } = useTranslation();
  const sequence = useActiveSequence();
  const preferences = useViewerPreferences();
  return (
    <Panel>
      <PanelHeader>
        <PanelTitle>{t('editor.viewer.title')}</PanelTitle>
        {sequence !== null && <ViewerToolbar preferences={preferences} />}
      </PanelHeader>
      <PanelBody scrollable={false} className="bg-bg">
        {sequence === null ? (
          <EmptyState
            icon={MonitorPlay}
            title={t('editor.viewer.empty.title')}
            description={t('editor.viewer.empty.description')}
          />
        ) : (
          <ShortcutRegion context="viewer" className="flex size-full flex-col">
            <ViewerStage
              sequence={sequence}
              preferences={preferences}
              createEngine={createEngine}
            />
            <ViewerFooter sequence={sequence} />
          </ShortcutRegion>
        )}
      </PanelBody>
    </Panel>
  );
}
