import type { Flicks, TranscriptionSettings } from '@se/protocol';
import { EmptyState, ScrollArea, Spinner, toast } from '@se/ui';
import { Captions } from '@se/ui/icons';
import { useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { useCoreClient } from '../../../shared/core';
import { errorMessageKey } from '../../../shared/errors';
import { selectProjectId, useProjectSelector } from '../../../shared/project-store';
import { useSelectedAssetId } from '../../media-bin';
import { inOutControls, setPlayheadTime, useActiveSequence } from '../../timeline';
import { setTranscriptionSettings } from '../api';
import { useTranscribe } from '../hooks/use-transcribe';
import { useTranscriptModel, type TranscriptMode } from '../hooks/use-transcript-model';
import { useWordHighlight } from '../hooks/use-word-highlight';
import { useWordSelection, type WordRange } from '../hooks/use-word-selection';
import { sequenceTimeOf } from '../services/media-words';
import { findMatches, stepMatch } from '../services/word-search';
import { TranscriptSettingsPopover } from './TranscriptSettingsPopover';
import { UntranscribedMedia } from './TranscriptStatus';
import { TranscriptText } from './TranscriptText';
import { TranscriptToolbar } from './TranscriptToolbar';

/**
 * The « Transcription » tab: the active sequence's words in sequence time (only what its sound
 * clips play) or the selected media's own words; the word under the playhead is highlighted,
 * a click moves the playhead, a drag marks in/out, a search highlights and steps through matches,
 * unsure words are underlined. Media without transcript can be transcribed from here.
 */
export function TranscriptView() {
  const { t } = useTranslation();
  const client = useCoreClient();
  const projectId = useProjectSelector(selectProjectId);
  const settings = useProjectSelector((state) => state.mirror?.project.settings.transcription);
  const sequence = useActiveSequence();
  const assetId = useSelectedAssetId();
  const [mode, setMode] = useState<TranscriptMode>('sequence');
  const [query, setQuery] = useState('');
  const [currentMatch, setCurrentMatch] = useState<number | null>(null);
  const containerRef = useRef<HTMLDivElement | null>(null);
  const model = useTranscriptModel(projectId, mode, sequence, assetId);
  const transcribe = useTranscribe(projectId);
  const words = model.transcript.words;
  const matches = findMatches(words, query);
  useWordHighlight(
    containerRef,
    words,
    mode === 'sequence' && sequence !== null ? sequence.id : null,
  );

  /** The sequence time of word `index` of the view (media mode goes through the sequence). */
  const sequenceTime = (index: number): Flicks | null => {
    const word = words[index];
    if (word === undefined) return null;
    if (mode === 'sequence') return word.start;
    return sequenceTimeOf(model.sequenceWords, word.assetId, word.wordIndex);
  };
  const seek = (index: number) => {
    const time = sequenceTime(index);
    if (sequence === null) return;
    if (time === null) toast.info(t('editor.transcript.notInSequence'));
    else setPlayheadTime(sequence.id, time);
  };
  const markRange = (range: WordRange) => {
    const last = words[range.last];
    if (sequence === null || mode !== 'sequence' || last === undefined) {
      seek(range.first);
      return;
    }
    const start = words[range.first]?.start ?? last.start;
    inOutControls(sequence).markSpan(start, last.end);
    setPlayheadTime(sequence.id, start);
  };
  const selection = useWordSelection(seek, markRange);
  const step = (direction: 1 | -1) => {
    const next = stepMatch(matches.length, currentMatch, direction);
    setCurrentMatch(next);
    const match = next === null ? undefined : matches[next];
    if (match === undefined) return;
    containerRef.current
      ?.querySelector(`[data-word-index="${String(match.first)}"]`)
      ?.scrollIntoView({ block: 'center' });
    seek(match.first);
  };
  const saveSettings = (next: TranscriptionSettings) => {
    if (projectId === null) return;
    setTranscriptionSettings(client, projectId, next).catch((error: unknown) => {
      toast.error(t(errorMessageKey(error)));
    });
  };

  if (sequence === null || settings === undefined) {
    return <EmptyState icon={Captions} title={t('editor.transcript.noProject')} />;
  }
  return (
    <div className="flex size-full min-h-0 flex-col">
      <TranscriptToolbar
        mode={mode}
        onModeChange={(next) => {
          setMode(next);
          selection.clear();
          setCurrentMatch(null);
        }}
        query={query}
        onQueryChange={(next) => {
          setQuery(next);
          setCurrentMatch(null);
        }}
        matchCount={matches.length}
        currentMatch={currentMatch}
        onStep={step}
        settings={<TranscriptSettingsPopover settings={settings} onSave={saveSettings} />}
      />
      <UntranscribedMedia
        assets={model.untranscribed}
        onTranscribe={(ids) => {
          void transcribe(ids);
        }}
      />
      {words.length > 0 ? (
        <ScrollArea className="min-h-0 flex-1">
          <TranscriptText
            transcript={model.transcript}
            containerRef={containerRef}
            selection={selection}
            matches={matches}
            currentMatch={currentMatch}
          />
        </ScrollArea>
      ) : model.loading ? (
        <div className="flex flex-1 items-center justify-center">
          <Spinner label={t('editor.transcript.loading')} />
        </div>
      ) : (
        <EmptyState
          icon={Captions}
          title={t(
            mode === 'sequence'
              ? 'editor.transcript.emptySequence'
              : 'editor.transcript.emptyMedia',
          )}
          description={t(
            model.media.length === 0
              ? mode === 'sequence'
                ? 'editor.transcript.noSoundInSequence'
                : 'editor.transcript.selectMedia'
              : 'editor.transcript.transcribeHint',
          )}
        />
      )}
    </div>
  );
}
