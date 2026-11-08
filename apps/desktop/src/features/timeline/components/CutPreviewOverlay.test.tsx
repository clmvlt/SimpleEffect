/**
 * @vitest-environment jsdom
 */
import type { Flicks } from '@se/protocol';
import { act, render } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';

import { clearCutPreview, showCutPreview } from '../store/cut-preview-store';
import { CutPreviewOverlay } from './CutPreviewOverlay';

const seconds = (value: number) => Math.round(value * 705_600_000) as Flicks;
const viewport = { pixelsPerSecond: 50, scrollTime: 0 as Flicks, width: 1000 };

afterEach(() => {
  clearCutPreview();
});

describe('CutPreviewOverlay', () => {
  it('draws the previewed cuts of its sequence as bands, and nothing for another one', () => {
    const { container } = render(<CutPreviewOverlay sequenceId="sequence-1" viewport={viewport} />);
    expect(container.querySelectorAll('[data-cut-preview]')).toHaveLength(0);
    act(() => {
      showCutPreview('sequence-1', [
        { start: seconds(2), duration: seconds(1) },
        { start: seconds(40), duration: seconds(0.5) },
      ]);
    });
    const bands = container.querySelectorAll<HTMLElement>('[data-cut-preview]');
    expect(bands).toHaveLength(1);
    expect(bands[0]?.style.left).toBe('100px');
    expect(bands[0]?.style.width).toBe('50px');
    act(() => {
      showCutPreview('sequence-2', [{ start: seconds(2), duration: seconds(1) }]);
    });
    expect(container.querySelectorAll('[data-cut-preview]')).toHaveLength(0);
  });
});
