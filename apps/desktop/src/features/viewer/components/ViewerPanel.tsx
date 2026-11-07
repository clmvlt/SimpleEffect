import { EmptyState, Panel, PanelBody, PanelHeader, PanelTitle } from '@se/ui';
import { MonitorPlay } from '@se/ui/icons';
import { useTranslation } from 'react-i18next';

import { useActiveSequence } from '../../timeline';
import { useViewerPreferences } from '../hooks/use-viewer-preferences';
import { createAudioEngine, type AudioEngineFactory } from '../services/audio-engine-factory';
import { createPreviewEngine, type PreviewEngineFactory } from '../services/preview-engine-factory';
import { ViewerToolbar } from './ViewerToolbar';
import { ViewerWorkspace } from './ViewerWorkspace';

/** Props of {@link ViewerPanel}. */
export interface ViewerPanelProps {
  /** Starts the engine (tests inject a fake; the app uses `@se/engine/preview`). */
  readonly createEngine?: PreviewEngineFactory | undefined;
  /** Starts the audio engine (tests inject a fake; the app uses `@se/engine/audio`). */
  readonly createAudio?: AudioEngineFactory | undefined;
}

/**
 * The program viewer: the exact frame of the active sequence at the playhead, drawn by
 * `@se/engine` (the code path export uses), real-time playback with sound, the transport bar and
 * the performance HUD. Time and playback shortcuts work here too (context `viewer`).
 */
export function ViewerPanel({
  createEngine = createPreviewEngine,
  createAudio = createAudioEngine,
}: ViewerPanelProps) {
  const { t } = useTranslation();
  const sequence = useActiveSequence();
  const preferences = useViewerPreferences();
  return (
    <Panel>
      <PanelHeader>
        <PanelTitle>{t('editor.viewer.title')}</PanelTitle>
        {sequence !== null && <ViewerToolbar preferences={preferences} sequenceId={sequence.id} />}
      </PanelHeader>
      <PanelBody scrollable={false} className="bg-bg">
        {sequence === null ? (
          <EmptyState
            icon={MonitorPlay}
            title={t('editor.viewer.empty.title')}
            description={t('editor.viewer.empty.description')}
          />
        ) : (
          <ViewerWorkspace
            sequence={sequence}
            preferences={preferences}
            createEngine={createEngine}
            createAudio={createAudio}
          />
        )}
      </PanelBody>
    </Panel>
  );
}
