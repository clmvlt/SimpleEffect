import type { Flicks, SequenceEntity } from '@se/protocol';
import { TimecodeDisplay, Toolbar, ToolbarButton, Tooltip, TransportBar } from '@se/ui';
import { AudioWaveform } from '@se/ui/icons';
import { useTranslation } from 'react-i18next';

import { formatTimecode } from '../../../shared/time';
import { usePlayheadTime } from '../../timeline';
import { usePlaybackState } from '../hooks/use-playback-session';
import type { PlaybackSession } from '../services/playback-session';
import { setViewerPreferences, type ViewerPreferences } from '../store/viewer-store';
import { PlaybackMeter } from './PlaybackMeter';

/** Props of {@link ViewerTransport}. */
export interface ViewerTransportProps {
  readonly session: PlaybackSession;
  readonly sequence: SequenceEntity;
  readonly contentEnd: Flicks;
  readonly preferences: ViewerPreferences;
}

/**
 * The transport bar under the picture: current timecode and duration, go to start, previous
 * frame, play/pause, stop, next frame, go to end, loop, the master meter and the audio scrub
 * toggle. Buttons call the playback session, like the shortcuts do.
 */
export function ViewerTransport({
  session,
  sequence,
  contentEnd,
  preferences,
}: ViewerTransportProps) {
  const { t } = useTranslation();
  const { rate } = usePlaybackState(session);
  return (
    <TransportBar
      className="shrink-0 border-t border-border"
      playing={rate !== 0}
      onPlayPause={() => {
        session.togglePlay();
      }}
      onStop={() => {
        session.stopPlayback();
      }}
      onPreviousFrame={() => {
        session.step(-1);
      }}
      onNextFrame={() => {
        session.step(1);
      }}
      onGoToStart={() => {
        session.goToStart();
      }}
      onGoToEnd={() => {
        session.goToEnd();
      }}
      looping={preferences.loop}
      onLoopChange={(loop) => {
        setViewerPreferences({ loop });
      }}
      labels={{
        toolbar: t('editor.viewer.transport.toolbar'),
        goToStart: t('editor.viewer.transport.goToStart'),
        previousFrame: t('editor.viewer.transport.previousFrame'),
        play: t('editor.viewer.transport.play'),
        pause: t('editor.viewer.transport.pause'),
        stop: t('editor.viewer.transport.stop'),
        nextFrame: t('editor.viewer.transport.nextFrame'),
        goToEnd: t('editor.viewer.transport.goToEnd'),
        loop: t('editor.viewer.transport.loop'),
      }}
      timecode={<TransportTimecodes sequence={sequence} contentEnd={contentEnd} />}
      trailing={
        <>
          <PlaybackMeter session={session} />
          <Toolbar label={t('editor.viewer.transport.options')}>
            <Tooltip content={t('editor.viewer.transport.scrubAudio')}>
              <ToolbarButton
                icon={AudioWaveform}
                label={t('editor.viewer.transport.scrubAudio')}
                pressed={preferences.scrubAudio}
                onClick={() => {
                  setViewerPreferences({ scrubAudio: !preferences.scrubAudio });
                }}
              />
            </Tooltip>
          </Toolbar>
        </>
      }
    />
  );
}

/**
 * The playhead timecode and the sequence duration. Alone in its component: it re-renders on
 * every played frame, the transport bar does not.
 */
function TransportTimecodes({
  sequence,
  contentEnd,
}: Pick<ViewerTransportProps, 'sequence' | 'contentEnd'>) {
  const { t } = useTranslation();
  const rate = sequence.format.frameRate;
  const time = usePlayheadTime(sequence.id);
  const format = (value: number) => formatTimecode(value as Flicks, rate);
  return (
    <div className="flex items-center gap-1 text-xs text-text-muted">
      <TimecodeDisplay
        className="text-text"
        label={t('editor.viewer.playhead')}
        value={time}
        format={format}
      />
      <span aria-hidden>/</span>
      <TimecodeDisplay
        label={t('editor.viewer.transport.duration')}
        value={contentEnd}
        format={format}
      />
    </div>
  );
}
