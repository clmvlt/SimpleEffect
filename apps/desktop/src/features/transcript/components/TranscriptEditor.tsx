import type { Flicks, ProjectId, SequenceId, TimeRange } from '@se/protocol';
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
  ContextMenuTrigger,
  toast,
} from '@se/ui';
import type { RefObject } from 'react';
import { useTranslation } from 'react-i18next';

import { useCoreClient } from '../../../shared/core';
import { errorMessageKey } from '../../../shared/errors';
import { ShortcutRegion, useShortcutAction } from '../../../shared/shortcuts';
import type { SequenceTranscript } from '../../../shared/transcript';
import { findFillers } from '../api';
import type { SmartCutActions } from '../hooks/use-smart-cuts';
import { useSmartCutSettings } from '../hooks/use-smart-cut-settings';
import type { WordSelection } from '../hooks/use-word-selection';
import type { CutPassage } from '../services/cut-passages';
import type { WordMatch } from '../services/word-search';
import { TranscriptText } from './TranscriptText';

/** Props of {@link TranscriptEditor}. */
export interface TranscriptEditorProps {
  readonly projectId: ProjectId;
  readonly sequenceId: SequenceId;
  readonly transcript: SequenceTranscript;
  readonly containerRef: RefObject<HTMLDivElement | null>;
  readonly selection: WordSelection;
  readonly matches: readonly WordMatch[];
  readonly currentMatch: number | null;
  /** Whether words can be cut (sequence mode). */
  readonly editable: boolean;
  /** Passages cut out, when shown. */
  readonly passages: readonly CutPassage[];
  readonly cuts: SmartCutActions;
  /** Called once the selected words were cut (the in/out marked by selecting them is stale). */
  readonly onSelectionCut: () => void;
}

/**
 * The transcript as an editing surface (prompt 22): select words then Suppr (or « Couper » in the
 * context menu) to cut them out of the sequence — one undoable ripple cut on their clips and the
 * linked picture —; the menu also removes the hesitations or the silences of the selection.
 */
export function TranscriptEditor({
  projectId,
  sequenceId,
  transcript,
  containerRef,
  selection,
  matches,
  currentMatch,
  editable,
  passages,
  cuts,
  onSelectionCut,
}: TranscriptEditorProps) {
  const { t } = useTranslation();
  const client = useCoreClient();
  const { settings } = useSmartCutSettings();
  const range = selection.range;
  const first = range === null ? undefined : transcript.words[range.first];
  const last = range === null ? undefined : transcript.words[range.last];
  const span: TimeRange | null =
    first === undefined || last === undefined
      ? null
      : { start: first.start, duration: (last.end - first.start) as Flicks };
  const canCut = editable && span !== null && !cuts.busy;

  const cutSelection = () => {
    if (!canCut) return;
    void cuts.cutWords([span], 'words').then((done) => {
      if (!done) return;
      selection.clear();
      onSelectionCut();
    });
  };
  const cutFillers = async () => {
    if (!canCut) return;
    try {
      const { occurrences } = await findFillers(client, projectId, sequenceId, span);
      const ranges = occurrences.map((found) => ({
        start: found.start,
        duration: (found.end - found.start) as Flicks,
      }));
      if (ranges.length === 0) {
        toast.info(t('editor.transcript.cuts.noFillers'));
        return;
      }
      await cuts.cutWords(ranges, 'fillers');
    } catch (error: unknown) {
      toast.error(t(errorMessageKey(error)));
    }
  };
  const cutSilences = () => {
    if (!canCut || settings === null) return;
    void cuts.cutSilences({
      range: span,
      params: settings.silence,
      trackIds: null,
      allTracks: false,
    });
  };
  useShortcutAction('deleteWords', cutSelection, canCut);

  return (
    <ShortcutRegion context="transcript" className="outline-none" tabIndex={-1}>
      <ContextMenu>
        <ContextMenuTrigger asChild disabled={!editable}>
          <div>
            <TranscriptText
              transcript={transcript}
              containerRef={containerRef}
              selection={selection}
              matches={matches}
              currentMatch={currentMatch}
              passages={passages}
              onRestore={cuts.restore}
            />
          </div>
        </ContextMenuTrigger>
        <ContextMenuContent>
          <ContextMenuItem disabled={!canCut} onSelect={cutSelection}>
            {t('editor.transcript.menu.cutWords', {
              count: range === null ? 0 : range.last - range.first + 1,
            })}
          </ContextMenuItem>
          <ContextMenuSeparator />
          <ContextMenuItem
            disabled={!canCut}
            onSelect={() => {
              void cutFillers();
            }}
          >
            {t('editor.transcript.menu.cutFillers')}
          </ContextMenuItem>
          <ContextMenuItem disabled={!canCut || settings === null} onSelect={cutSilences}>
            {t('editor.transcript.menu.cutSilences')}
          </ContextMenuItem>
        </ContextMenuContent>
      </ContextMenu>
    </ShortcutRegion>
  );
}
