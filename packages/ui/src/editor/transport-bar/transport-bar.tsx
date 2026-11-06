import {
  Pause,
  Play,
  Repeat,
  SkipBack,
  SkipForward,
  Square,
  StepBack,
  StepForward,
} from 'lucide-react';
import type { ReactNode } from 'react';

import { cn } from '../../lib/cn';
import { Toolbar, ToolbarButton, ToolbarGroup, ToolbarSeparator } from '../toolbar';

/** Accessible names of the transport buttons; the app passes localised ones. */
export interface TransportBarLabels {
  toolbar: string;
  goToStart: string;
  previousFrame: string;
  play: string;
  pause: string;
  stop: string;
  nextFrame: string;
  goToEnd: string;
  loop: string;
}

/** Default (English) names. */
export const DEFAULT_TRANSPORT_LABELS: TransportBarLabels = {
  toolbar: 'Transport',
  goToStart: 'Go to start',
  previousFrame: 'Previous frame',
  play: 'Play',
  pause: 'Pause',
  stop: 'Stop',
  nextFrame: 'Next frame',
  goToEnd: 'Go to end',
  loop: 'Loop playback',
};

/** Props of {@link TransportBar}. */
export interface TransportBarProps {
  /** Playback is running: the main button shows Pause. */
  playing: boolean;
  onPlayPause: () => void;
  onStop?: (() => void) | undefined;
  onPreviousFrame?: (() => void) | undefined;
  onNextFrame?: (() => void) | undefined;
  onGoToStart?: (() => void) | undefined;
  onGoToEnd?: (() => void) | undefined;
  /** Loop state; the loop toggle is shown when `onLoopChange` is given. */
  looping?: boolean | undefined;
  onLoopChange?: ((looping: boolean) => void) | undefined;
  /** Timecode slot, at the left of the controls (usually a `TimecodeInput`). */
  timecode?: ReactNode | undefined;
  /** Extra slot at the right (zoom, quality…). */
  trailing?: ReactNode | undefined;
  labels?: Partial<TransportBarLabels> | undefined;
  disabled?: boolean | undefined;
  className?: string | undefined;
}

/**
 * Playback controls of the viewer: go to start, previous frame, play/pause, stop, next frame, go
 * to end and loop, with timecode and trailing slots. The controls form one toolbar (one tab stop,
 * arrow keys inside). Buttons whose callback is missing are disabled. Keyboard shortcuts (J/K/L,
 * Space) belong to the app, which calls the same callbacks.
 */
export function TransportBar({
  playing,
  onPlayPause,
  onStop,
  onPreviousFrame,
  onNextFrame,
  onGoToStart,
  onGoToEnd,
  looping = false,
  onLoopChange,
  timecode,
  trailing,
  labels: labelOverrides,
  disabled = false,
  className,
}: TransportBarProps) {
  const labels = { ...DEFAULT_TRANSPORT_LABELS, ...labelOverrides };
  return (
    <div className={cn('flex h-control-md items-center gap-3 px-2', className)}>
      {timecode !== undefined && <div className="flex shrink-0 items-center">{timecode}</div>}
      <Toolbar label={labels.toolbar} className="mx-auto">
        <ToolbarGroup>
          <ToolbarButton
            icon={SkipBack}
            label={labels.goToStart}
            disabled={disabled || onGoToStart === undefined}
            onClick={onGoToStart}
          />
          <ToolbarButton
            icon={StepBack}
            label={labels.previousFrame}
            disabled={disabled || onPreviousFrame === undefined}
            onClick={onPreviousFrame}
          />
          <ToolbarButton
            icon={playing ? Pause : Play}
            label={playing ? labels.pause : labels.play}
            variant="secondary"
            disabled={disabled}
            onClick={onPlayPause}
          />
          <ToolbarButton
            icon={Square}
            label={labels.stop}
            disabled={disabled || onStop === undefined}
            onClick={onStop}
          />
          <ToolbarButton
            icon={StepForward}
            label={labels.nextFrame}
            disabled={disabled || onNextFrame === undefined}
            onClick={onNextFrame}
          />
          <ToolbarButton
            icon={SkipForward}
            label={labels.goToEnd}
            disabled={disabled || onGoToEnd === undefined}
            onClick={onGoToEnd}
          />
        </ToolbarGroup>
        {onLoopChange !== undefined && (
          <>
            <ToolbarSeparator />
            <ToolbarButton
              icon={Repeat}
              label={labels.loop}
              pressed={looping}
              disabled={disabled}
              onClick={() => {
                onLoopChange(!looping);
              }}
            />
          </>
        )}
      </Toolbar>
      {trailing !== undefined && <div className="flex shrink-0 items-center gap-1">{trailing}</div>}
    </div>
  );
}
