import { cn, EmptyState, Panel, PanelBody, PanelHeader, PanelTitle } from '@se/ui';
import { GalleryHorizontal } from '@se/ui/icons';
import { useTranslation } from 'react-i18next';

import { ShortcutRegion, useActiveShortcutContext } from '../../../shared/shortcuts';
import { inOutControls } from '../hooks/use-in-out';
import { usePlayheadControls } from '../hooks/use-playhead-controls';
import { useTimelineCommands } from '../hooks/use-timeline-commands';
import { useTimelineModel, type TimelineModel } from '../hooks/use-timeline-model';
import { useTimelinePersistence } from '../hooks/use-timeline-persistence';
import { useTimelineShortcuts } from '../hooks/use-timeline-shortcuts';
import { useTimelineViewport } from '../hooks/use-timeline-viewport';
import { TRACK_HEIGHT_PRESETS } from '../services/track-height';
import { clearSelection } from '../store/selection-store';
import { setActiveSequence, setTrackHeights } from '../store/timeline-view-store';
import { SequenceTabs } from './SequenceTabs';
import { TimelineToolbar } from './TimelineToolbar';
import { TimelineView } from './TimelineView';

/** The timeline of the open project: its active sequence, or an empty state without one. */
export function TimelinePanel() {
  const { t } = useTranslation();
  const model = useTimelineModel();
  useTimelinePersistence(model?.projectId ?? null);
  if (model === null) {
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
  return <TimelineEditor key={model.sequence.id} model={model} />;
}

/** The timeline of one sequence: tabs, tools, ruler, tracks and shortcuts. */
function TimelineEditor({ model }: { readonly model: TimelineModel }) {
  const { t } = useTranslation();
  const commands = useTimelineCommands(model.projectId);
  const api = useTimelineViewport(model.sequence, model.contentEnd);
  const playhead = usePlayheadControls(
    model.sequence,
    model.contentEnd,
    model.editPoints,
    api.reveal,
  );
  const inOut = inOutControls(model.sequence);
  useTimelineShortcuts({ model, viewport: api, playhead, inOut, commands });
  const active = useActiveShortcutContext() === 'timeline';
  return (
    <ShortcutRegion context="timeline" claimOnMount className="size-full">
      <Panel
        data-active={active || undefined}
        className={cn(active && 'shadow-[inset_0_1px_0_var(--color-accent)]')}
      >
        <PanelHeader className="gap-3">
          <PanelTitle>{t('editor.timeline.title')}</PanelTitle>
          <SequenceTabs
            sequences={model.sequences}
            activeId={model.sequence.id}
            onSelect={(sequenceId) => {
              clearSelection();
              setActiveSequence(sequenceId);
            }}
            onCreate={() => {
              const name = t('editor.timeline.newSequenceName', {
                number: model.sequences.length + 1,
              });
              void commands.createSequence(name).then((sequenceId) => {
                if (sequenceId !== null) setActiveSequence(sequenceId);
              });
            }}
          />
          <TimelineToolbar
            sequence={model.sequence}
            api={api}
            playhead={playhead}
            onAddTrack={(kind) => {
              void commands.addTrack(model.sequence.id, kind, null);
            }}
            onTrackHeights={(preset) => {
              setTrackHeights(
                model.tracks.map((track) => track.id),
                TRACK_HEIGHT_PRESETS[preset],
              );
            }}
          />
        </PanelHeader>
        <PanelBody scrollable={false}>
          <TimelineView
            model={model}
            api={api}
            playhead={playhead}
            inOut={inOut}
            commands={commands}
          />
        </PanelBody>
      </Panel>
    </ShortcutRegion>
  );
}
