import type { WaveformPeaks } from '@se/protocol';

import { waveformPath } from '../services/waveform-path';

/** Props of {@link WaveformPreview}. */
export interface WaveformPreviewProps {
  readonly peaks: WaveformPeaks;
  readonly width: number;
  readonly height: number;
}

/** The waveform of a sound file, drawn with the audio item colour. Decorative. */
export function WaveformPreview({ peaks, width, height }: WaveformPreviewProps) {
  return (
    <svg
      aria-hidden
      width={width}
      height={height}
      viewBox={`0 0 ${String(width)} ${String(height)}`}
      className="text-item-audio"
    >
      <path d={waveformPath(peaks, width, height)} stroke="currentColor" strokeWidth={1} />
    </svg>
  );
}
