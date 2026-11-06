import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { Meter } from './meter';
import { levelToFraction, updatePeakHold } from './meter-math';

describe('levelToFraction', () => {
  it('maps dB linearly onto the scale and clamps the ends', () => {
    expect(levelToFraction(-30, -60, 0)).toBe(0.5);
    expect(levelToFraction(Number.NEGATIVE_INFINITY, -60, 0)).toBe(0);
    expect(levelToFraction(-80, -60, 0)).toBe(0);
    expect(levelToFraction(3, -60, 0)).toBe(1);
  });
});

describe('updatePeakHold', () => {
  it('raises peaks at once, holds them, then lets them fall', () => {
    let state = updatePeakHold(undefined, [-10, -20], 0);
    expect(state.peaks).toEqual([-10, -20]);

    state = updatePeakHold(state, [-30, -5], 1000);
    expect(state.peaks).toEqual([-10, -5]);

    // After the 1.5 s hold, the left peak falls at 20 dB/s for the elapsed 1 s.
    state = updatePeakHold(state, [-30, -5], 2000);
    expect(state.peaks[0]).toBeCloseTo(-30);
    state = updatePeakHold(undefined, [-10], 0);
    state = updatePeakHold(state, [-40], 1600);
    expect(state.peaks[0]).toBeCloseTo(-40);
  });

  it('never drops a peak below the current level', () => {
    let state = updatePeakHold(undefined, [-6], 0);
    state = updatePeakHold(state, [-8], 10_000);
    expect(state.peaks[0]).toBe(-8);
  });
});

describe('Meter', () => {
  it('exposes one named meter per channel with its level', () => {
    render(
      <Meter
        label="Niveau"
        channelLabels={['Gauche', 'Droite']}
        levels={[-12, Number.NEGATIVE_INFINITY]}
      />,
    );
    expect(screen.getByRole('group', { name: 'Niveau' })).toBeInTheDocument();
    const left = screen.getByRole('meter', { name: 'Gauche' });
    expect(left).toHaveAttribute('aria-valuenow', '-12');
    expect(left).toHaveAttribute('aria-valuetext', '-12.0 dB');
    const right = screen.getByRole('meter', { name: 'Droite' });
    expect(right).toHaveAttribute('aria-valuenow', '-60');
    expect(right).toHaveAttribute('aria-valuetext', '≤ -60 dB');
  });

  it('draws a peak line at the held level', () => {
    const { container } = render(
      <Meter levels={[-40, -40]} peaks={[-30, Number.NEGATIVE_INFINITY]} />,
    );
    const peaksDrawn = container.querySelectorAll('[data-peak]');
    expect(peaksDrawn).toHaveLength(1);
    expect(peaksDrawn[0]).toHaveStyle({ bottom: '50%' });
  });
});
