import type { CSSProperties } from 'react';

import { cn } from '../../lib/cn';
import { DEFAULT_METER_ZONES, levelToFraction, type MeterZones } from './meter-math';

/** Props of {@link Meter}. */
export interface MeterProps {
  /** Current level of each channel in dBFS (`-Infinity` for silence). Usually two (L, R). */
  levels: readonly number[];
  /** Held peak of each channel in dBFS (see `updatePeakHold`); no peak line when omitted. */
  peaks?: readonly number[] | undefined;
  /** Bottom of the scale (default −60 dBFS). */
  minDb?: number | undefined;
  /** Top of the scale (default 0 dBFS). */
  maxDb?: number | undefined;
  zones?: MeterZones | undefined;
  orientation?: 'vertical' | 'horizontal' | undefined;
  /** Scale marks in dB, drawn next to the bars (default 0, −6, −12, −24, −48). */
  scaleMarks?: readonly number[] | undefined;
  showScale?: boolean | undefined;
  /** Accessible name of the meter group (default English « Audio level »). */
  label?: string | undefined;
  /** Accessible names of the channels (default English « Left », « Right »). */
  channelLabels?: readonly string[] | undefined;
  className?: string | undefined;
}

const DEFAULT_CHANNEL_LABELS = ['Left', 'Right'];
const DEFAULT_SCALE_MARKS = [0, -6, -12, -24, -48];

/** Text of a level for screen readers and scale marks. */
function formatDb(db: number, minDb: number): string {
  return db <= minDb ? `≤ ${String(minDb)} dB` : `${db.toFixed(1)} dB`;
}

/**
 * Hard-stop gradient painting the zones at their absolute positions on the track, so a bar shows
 * green, then orange, then red as it rises. Built from tokens only.
 */
function zoneGradient(zones: MeterZones, minDb: number, maxDb: number, vertical: boolean): string {
  const warning = `${String(levelToFraction(zones.warningDb, minDb, maxDb) * 100)}%`;
  const danger = `${String(levelToFraction(zones.dangerDb, minDb, maxDb) * 100)}%`;
  return [
    `linear-gradient(to ${vertical ? 'top' : 'right'}`,
    `var(--color-success) 0%, var(--color-success) ${warning}`,
    `var(--color-warning) ${warning}, var(--color-warning) ${danger}`,
    `var(--color-danger) ${danger}, var(--color-danger) 100%)`,
  ].join(', ');
}

/** Places an element at `fraction` along the bar (from the bottom, or the left). */
function alongBar(fraction: number, vertical: boolean): CSSProperties {
  const offset = `${String(fraction * 100)}%`;
  return vertical ? { bottom: offset } : { left: offset };
}

/**
 * A stereo (or n-channel) level meter in dBFS with green/orange/red zones, optional peak-hold
 * lines and a dB scale. Purely visual: levels and peaks come from props, updated by the audio
 * engine. Each channel is exposed as an ARIA `meter`.
 */
export function Meter({
  levels,
  peaks,
  minDb = -60,
  maxDb = 0,
  zones = DEFAULT_METER_ZONES,
  orientation = 'vertical',
  scaleMarks = DEFAULT_SCALE_MARKS,
  showScale = true,
  label = 'Audio level',
  channelLabels = DEFAULT_CHANNEL_LABELS,
  className,
}: MeterProps) {
  const vertical = orientation === 'vertical';
  const gradient = zoneGradient(zones, minDb, maxDb, vertical);
  return (
    <div
      role="group"
      aria-label={label}
      className={cn(
        'flex gap-1',
        vertical ? 'h-full min-h-24 flex-row' : 'w-full min-w-24 flex-col',
        className,
      )}
    >
      {levels.map((level, channel) => {
        const fraction = levelToFraction(level, minDb, maxDb);
        const peak = peaks?.[channel];
        const peakFraction = peak === undefined ? 0 : levelToFraction(peak, minDb, maxDb);
        const unlit = `${String((1 - fraction) * 100)}%`;
        return (
          <div
            // Channels are positional (L, R…): the index is their identity.
            key={channel}
            role="meter"
            aria-label={channelLabels[channel] ?? String(channel + 1)}
            aria-valuemin={minDb}
            aria-valuemax={maxDb}
            aria-valuenow={Math.max(minDb, Math.min(maxDb, Number.isFinite(level) ? level : minDb))}
            aria-valuetext={formatDb(level, minDb)}
            className={cn('relative overflow-hidden rounded-xs', vertical ? 'w-1.5' : 'h-1.5')}
            style={{ backgroundImage: gradient }}
          >
            <div
              className={cn(
                'absolute bg-control',
                vertical ? 'inset-x-0 top-0' : 'inset-y-0 right-0',
              )}
              style={vertical ? { height: unlit } : { width: unlit }}
            />
            {peakFraction > 0 && (
              <div
                data-peak
                className={cn('absolute bg-text', vertical ? 'inset-x-0 h-px' : 'inset-y-0 w-px')}
                style={alongBar(peakFraction, vertical)}
              />
            )}
          </div>
        );
      })}
      {showScale && (
        <div
          aria-hidden
          className={cn(
            'relative text-2xs text-text-subtle',
            vertical ? 'ml-0.5 w-6' : 'mt-0.5 h-3',
          )}
        >
          {scaleMarks.map((mark) => (
            <span
              key={mark}
              className={cn(
                'absolute leading-none tabular-nums',
                vertical ? 'translate-y-1/2' : '-translate-x-1/2',
              )}
              style={alongBar(levelToFraction(mark, minDb, maxDb), vertical)}
            >
              {mark}
            </span>
          ))}
        </div>
      )}
    </div>
  );
}
